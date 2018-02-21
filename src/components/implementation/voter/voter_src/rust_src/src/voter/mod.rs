pub mod voter_lib;
pub mod voter_config;

use self::voter_lib::VoteStatus;
use self::voter_config::*;
use lib_composite::sl_lock::Lock;
use lib_composite::sl::{Sl, ThreadParameter};
use std::ops::{Deref, DerefMut};
use std::mem::replace;

pub struct Voter {
    application: voter_lib::Component,
}

impl Voter {
    pub fn new(app_reps: usize, app_entry: fn(sl: Sl, replica_id: usize), sl: Sl) -> Voter {
        Voter {
            application: voter_lib::Component::new(app_reps, sl, app_entry),
        }
    }

    pub fn monitor_application(voter_lock: &Lock<Voter>, sl: Sl) {
        let mut consecutive_inconclusive = 0;

        loop {
            sl.current_thread().set_param(ThreadParameter::Priority(voter_config::VOTE_PRIO));
            match Voter::monitor_vote(voter_lock, consecutive_inconclusive, sl) {
                VoteStatus::Success(consensus) => {
                    consecutive_inconclusive = 0;
                    let application_data = Voter::contact_server(consensus[0]);
                    Voter::transfer(voter_lock, application_data);
                },
                VoteStatus::Inconclusive(num_processing, _rep) => {
                    //track inconclusive for the case where only one replica is still processing
                    if num_processing == 1 {
                        consecutive_inconclusive += 1;
                    }
                },
                VoteStatus::Fail(_rep) => panic!("Replica Fault"), //TODO - handle faults
            }
            sl.current_thread().set_param(ThreadParameter::Priority(voter_config::REP_PRIO));
            sl.thd_yield();
        }
    }

    fn monitor_vote(voter_lock: &Lock<Voter>, consecutive_inconclusive: u8, sl: Sl) -> voter_lib::VoteStatus {
        let mut voter = voter_lock.lock();
        let vote = voter.deref_mut().application.collect_vote();
        match vote {
            VoteStatus::Success(consensus) => (),
            VoteStatus::Fail(replica_id) => voter.application.replicas[replica_id].recover(),
            VoteStatus::Inconclusive(_num_processing, replica_id) => {
                if consecutive_inconclusive > voter_config::MAX_INCONCLUSIVE {
                    println!("Inconclusive breach!");
                    voter.application.replicas[replica_id].recover();
                }
            }
        }

        vote
    }

    //TODO
    fn contact_server(sinv_id:u8) -> [u8; BUFF_SIZE] {
        println!("making sinv {}", sinv_id);

        [9;BUFF_SIZE]
    }

    fn transfer(voter_lock: &Lock<Voter>, data:[u8; BUFF_SIZE]) {
        let mut voter = voter_lock.lock();
        for replica in &mut voter.deref_mut().application.replicas {
            for i in 0..BUFF_SIZE {
                replica.data_buffer[i] = data[i];
            }
        }
        voter.deref_mut().application.wake_all();
    }

    pub fn request(
        voter_lock: &Lock<Voter>,
        data: [u8; BUFF_SIZE],
        replica_id: usize,
        sl: Sl,
    ) -> [u8; BUFF_SIZE]
    {
        println!("Making Request ....");
        {
            let mut voter = voter_lock.lock();
            voter.application.replicas[replica_id].write(data);
            voter.application.replicas[replica_id].state_transition(voter_lib::ReplicaState::Blocked);
        }

        sl.block();

        //get data returned from request.
        let mut voter = voter_lock.lock();
        replace(&mut voter.application.replicas[replica_id].data_buffer,[0; BUFF_SIZE])
    }
}
