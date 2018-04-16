pub mod voter_lib;
pub mod voter_config;
mod server_bindings;

use self::voter_lib::VoteStatus;
use self::voter_config::*;
use lib_composite::sl_lock::{Lock, LockGuard};

use lib_composite::sl::Sl;
use lib_composite::sys::sl::sl_thd_state;
use lib_composite::sys::types;

use lib_composite::memmgr_api::SharedMemoryReigon;

use std::ops::DerefMut;

extern {
    fn get_num_replicas() -> i32;
    fn voter_done_initalizing();
    fn cos_inv_token_rs() -> types::spdid_t;
}

pub struct Voter {
    application: voter_lib::Component,
    server_shrdmem: Lock<SharedMemoryReigon>,
}

lazy_static! {
    static ref VOTER:Lock<Voter> = unsafe {
        Lock::new(Sl::assert_scheduler_already_started(),Voter::new(get_num_replicas())
    )};
}

impl Voter {
    pub fn new(num_replicas: i32) -> Voter {
        let sl = unsafe {Sl::assert_scheduler_already_started()};

        Voter {
            application: voter_lib::Component::new(num_replicas),
            server_shrdmem: Lock::new(sl,SharedMemoryReigon::page_alloc()),
        }
    }

    pub fn initialize(sl:Sl) {
        loop {
            if {
                let voter = Voter::try_lock_and_wait(&*VOTER, sl);
                voter.application.replicas_initialized()
            } {
                break;
            }
            else {
                sl.thd_yield();
            }
        }

        unsafe {voter_done_initalizing()};
        Voter::monitor_application(sl);
    }

    pub fn replica_done_initializing(shdmem_id: i32) {
        let sl = unsafe {
            Sl::assert_scheduler_already_started()
        };

        let mut voter = Voter::try_lock_and_wait(&*VOTER, sl);


        for replica in &mut voter.application.replicas {
            if replica.id == 0 {
                replica.id = unsafe {
                    cos_inv_token_rs()
                };
                replica.shrdmem = Some(SharedMemoryReigon::page_map(shdmem_id));

                return;
            }
        }
        panic!("init replica not found");
    }

    pub fn monitor_application(sl: Sl) {
        let mut consecutive_inconclusive = 0;
        loop {
            match Voter::monitor_vote(&*VOTER, consecutive_inconclusive, sl) {
                VoteStatus::Success => {
                    println!("vote success");
                    consecutive_inconclusive = 0;

                    let mut voter_lock_guard = Voter::try_lock_and_wait(&*VOTER, sl);
                    let voter = voter_lock_guard.deref_mut();

                    let (ret,is_data_from_server) = server_bindings::handle_request(&voter.application.replicas[0].data_buffer,
                                                                                    &voter.server_shrdmem);
                    voter.transfer(is_data_from_server,ret);
                }
                VoteStatus::Inconclusive(num_processing, _rep) => {
                    //track inconclusive for the case where only one replica is still processing
                    if num_processing == 1 {
                        consecutive_inconclusive += 1;
                    }
                }
                VoteStatus::Fail(_rep) => panic!("Replica Fault"), //TODO - handle faults
            }
            sl.thd_yield();
        }
    }

    fn monitor_vote(voter_lock: &Lock<Voter>, consecutive_inconclusive: u8, sl: Sl) -> voter_lib::VoteStatus {
        println!("Getting Vote");
        let mut voter_lock_guard = Voter::try_lock_and_wait(voter_lock, sl);
        let voter = voter_lock_guard.deref_mut();
        let vote = voter.application.collect_vote();
        match vote {
            VoteStatus::Success => (),
            VoteStatus::Fail(replica_id) => voter.application.get_replica_by_spdid(replica_id).unwrap().recover(),
            VoteStatus::Inconclusive(_num_processing, replica_id) => {
                if consecutive_inconclusive > voter_config::MAX_INCONCLUSIVE {
                    println!("Inconclusive breach!");
                    voter.application.get_replica_by_spdid(replica_id).unwrap().recover();
                }
            }
        }

        vote
    }

    fn transfer(&mut self, is_data_from_server:bool, ret:i32) {
        for replica in &mut self.application.replicas {
            replica.ret = Some(ret);
            if is_data_from_server {
                let replica_shdmem = replica.shrdmem.as_mut().unwrap();
                replica_shdmem.mem[..].copy_from_slice(&self.server_shrdmem.lock().deref_mut().mem[..])
            }
        }

        self.application.wake_all();
    }

    pub fn request(replica_id: types::spdid_t, op:i32, data_size:usize, args:[u8;MAX_ARGS], sl:Sl) -> i32 {
        println!("Rep {} making request", replica_id);

        {
            let mut voter = Voter::try_lock_and_wait(&*VOTER, sl);
            voter.application.get_replica_by_spdid(replica_id).unwrap().request(op,data_size,args,sl);
        }

        sl.block();

       let mut voter = Voter::try_lock_and_wait(&*VOTER, sl);
       voter.application.get_replica_by_spdid(replica_id).unwrap().ret.take().unwrap()
    }

    //unsure if this is actually still necessary. now that we fixed the WOKE race
    fn try_lock_and_wait(voter_lock: &Lock<Voter>, sl: Sl) -> LockGuard<Voter> {
        let mut voter = voter_lock.try_lock();
        while voter.is_none() {
            sl.thd_yield();
            voter = voter_lock.try_lock();
        }
        voter.unwrap()
    }
}
