pub mod voter_lib;
pub mod voter_config;
mod server_bindings;

use self::voter_lib::VoteStatus;
use self::voter_config::*;
use lib_composite::sl_lock::{Lock, LockGuard};

use lib_composite::sl::Sl;
use lib_composite::sys::sl::sl_thd_state;
use lib_composite::sys::types;
use std::ops::DerefMut;
use std::mem::replace;
use std::slice;

extern {
    fn get_num_replicas() -> i32;
    fn voter_done_initalizing();
    fn cos_inv_token_rs() -> types::spdid_t;
}

pub struct Voter {
    application: voter_lib::Component,
}

lazy_static! {
    static ref VOTER:Lock<Voter> = unsafe {
        Lock::new(Sl::assert_scheduler_already_started(),Voter::new(get_num_replicas())
    )};
}

impl Voter {
    pub fn new(num_replicas: i32) -> Voter {
        Voter {
            application: voter_lib::Component::new(num_replicas),
        }
    }

    pub fn initialize(sl:Sl) {
        loop {
            let mut initialized = true;
            {
                let voter = Voter::try_lock_and_wait(&*VOTER, sl);
                initialized = voter.application.replicas_initialized()
            }
            if initialized {
                break;
            }
            else {
                sl.thd_yield();
            }
        }

        unsafe {voter_done_initalizing()};
        Voter::monitor_application(sl);
    }

    pub fn replica_done_initializing() {
        let sl;
        unsafe {
            sl = Sl::assert_scheduler_already_started();
        }

        let mut voter = Voter::try_lock_and_wait(&*VOTER, sl);

        for replica in &mut voter.application.replicas {
            if replica.id == 0 {
                unsafe {replica.id = cos_inv_token_rs();}
                return;
            }
        }
        panic!("init replica not found");
    }

    pub fn monitor_application(sl: Sl) {
        let mut consecutive_inconclusive = 0;
        loop {
            match Voter::monitor_vote(&*VOTER, consecutive_inconclusive, sl) {
                VoteStatus::Success(consensus) => {
                    consecutive_inconclusive = 0;
                    let application_data = Voter::contact_server(consensus);
                    Voter::transfer(&*VOTER, application_data, sl);
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
        let mut voter = Voter::try_lock_and_wait(voter_lock, sl);
        let vote = voter.deref_mut().application.collect_vote();
        match vote {
            VoteStatus::Success(_consensus) => (),
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

    fn contact_server(serialized_msg: [u8; BUFF_SIZE]) -> [u8; BUFF_SIZE] {
        server_bindings::handle_request(serialized_msg)
    }

    fn transfer(voter_lock: &Lock<Voter>, data: [u8; BUFF_SIZE], sl: Sl) {
        let mut voter = Voter::try_lock_and_wait(voter_lock, sl);
        for replica in &mut voter.deref_mut().application.replicas {
            for i in 0..BUFF_SIZE {
                replica.data_buffer[i] = data[i];
            }
        }
        voter.deref_mut().application.wake_all();
    }

    pub fn request(data: &mut [u8], op:i32, replica_id: types::spdid_t) -> [u8; BUFF_SIZE] {
        let sl:Sl;
        unsafe {
            sl = Sl::assert_scheduler_already_started();
        }

        println!("Rep {} making request", replica_id);
        {
            //is there a way to remove the need for this lock?
            let mut voter = Voter::try_lock_and_wait(&*VOTER, sl);
            voter.application.get_replica_by_spdid(replica_id).unwrap().write(op,data);
        }

        sl.block();

        //get data returned from request.
        let mut voter = Voter::try_lock_and_wait(&*VOTER, sl);
        replace(
            &mut voter.application.get_replica_by_spdid(replica_id).unwrap().data_buffer,
            [0; BUFF_SIZE],
        )
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
