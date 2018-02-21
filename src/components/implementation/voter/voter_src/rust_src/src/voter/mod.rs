pub mod voter_lib;
pub mod voter_config;

use self::voter_lib::VoteStatus;
use self::voter_config::*;
use lib_composite::sl_lock::Lock;
use lib_composite::sl::{Sl, ThreadParameter};
use std::ops::{Deref, DerefMut};

const APP_IDX: usize = 0;
const SRV_IDX: usize = 1;

pub struct Voter {
    /* application:0 Service_Provider:1 */
    components: [voter_lib::Component; voter_config::MAX_COMPS],
    active_component: usize,
    new_data: bool,
}

impl Voter {
    pub fn new(
        app_reps: usize,
        srv_reps: usize,
        app_entry: fn(sl: Sl, replica_id: usize),
        srv_entry: fn(sl: Sl, replica_id: usize),
        sl: Sl,
    ) -> Voter {
        Voter {
            components: [
                voter_lib::Component::new(app_reps, sl, app_entry),
                voter_lib::Component::new(srv_reps, sl, srv_entry),
            ],
            active_component: 0,
            new_data: false,
        }
    }

    pub fn monitor_components(voter_lock: &Lock<Voter>, sl: Sl) {
        loop {
            let mut consecutive_inconclusive = 0;
            let mut concensus = false;
            while !concensus {
                concensus = false;
                match Voter::monitor_vote(voter_lock, consecutive_inconclusive, sl) {
                    VoteStatus::Success => concensus = true,
                    VoteStatus::Inconclusive(num_processing, _rep) => {
                        //track inconclusive for the case where only one replica is still processing
                        if num_processing == 1 {
                            consecutive_inconclusive += 1;
                        }
                        sl.thd_yield();
                    }
                    VoteStatus::Fail(_rep) => concensus = true, //TODO - handle faults
                }
            }
            Voter::switch_active_component(voter_lock, sl);
        }
    }

    fn monitor_vote(voter_lock: &Lock<Voter>, consecutive_inconclusive: u8, sl: Sl) -> voter_lib::VoteStatus {
        sl.current_thread()
            .set_param(ThreadParameter::Priority(voter_config::VOTE_PRIO));

        let mut voter = voter_lock.lock();
        let current = voter.deref().active_component;
        let vote = voter.deref_mut().components[current].collect_vote();
        match vote {
            VoteStatus::Success => (),
            VoteStatus::Fail(replica_id) => voter.components[current].replicas[replica_id].recover(),
            VoteStatus::Inconclusive(_num_processing, replica_id) => {
                if consecutive_inconclusive > voter_config::MAX_INCONCLUSIVE {
                    println!("Inconclusive breach!");
                    voter.components[current].replicas[replica_id].recover();
                }
            }
        }

        sl.current_thread()
            .set_param(ThreadParameter::Priority(voter_config::REP_PRIO));
        vote
    }

    fn switch_active_component(voter_lock: &Lock<Voter>, sl: Sl) {
        sl.current_thread()
            .set_param(ThreadParameter::Priority(voter_config::VOTE_PRIO));

        let mut voter = voter_lock.lock();
        voter.new_data = true;
        let current = voter.deref().active_component;
        let next_comp = (current + 1) % 2;

        voter.deref_mut().transfer();
        voter.deref_mut().active_component = next_comp;
        voter.deref_mut().components[next_comp].wake_all();

        sl.current_thread()
            .set_param(ThreadParameter::Priority(voter_config::REP_PRIO));
        sl.thd_yield();
    }

    fn transfer(&mut self) {
        //transfer data from replica local buffers of current comp to next comp.
        let next_comp_idx = (self.active_component + 1) % 2;

        let msg = self.components[self.active_component].replicas[0].data_buffer;
        for replica in &mut self.components[next_comp_idx].replicas {
            for i in 0..BUFF_SIZE {
                replica.data_buffer[i] = msg[i];
            }
        }

        for replica in &mut self.components[self.active_component].replicas {
            for i in 0..BUFF_SIZE {
                replica.data_buffer[i] = 0;
            }
        }

        self.new_data = false;
    }

    pub fn request(
        voter_lock: &Lock<Voter>,
        data: [u8; BUFF_SIZE],
        replica_id: usize,
        sl: Sl,
    ) -> [u8; BUFF_SIZE] {
        println!("Making Request ....");
        {
            let mut voter = voter_lock.lock();
            voter.components[APP_IDX].replicas[replica_id].write(data);
            voter.components[APP_IDX].replicas[replica_id].state_transition(voter_lib::ReplicaState::Blocked);
        }
        sl.block();

        //get data returned from request.
        let voter = voter_lock.lock();
        let ref data = &voter.components[APP_IDX].replicas[0].data_buffer;
        let mut msg = [0; BUFF_SIZE];

        for i in 0..BUFF_SIZE {
            msg[i] = data[i];
        }

        msg
    }

    pub fn get_request(voter_lock: &Lock<Voter>, replica_id: usize, sl: Sl) -> [u8; BUFF_SIZE] {
        println!("Getting request ....");
        let mut msg: [u8; BUFF_SIZE] = [0; BUFF_SIZE];
        {
            let mut voter = voter_lock.lock();
            let buffer = &mut voter.deref_mut().components[SRV_IDX].replicas[replica_id].data_buffer;
            for i in 0..BUFF_SIZE {
                msg[i] = buffer[i];
                buffer[i] = 0;
            }
        }
        msg
    }

    pub fn send_response(voter_lock: &Lock<Voter>, data: [u8; BUFF_SIZE], replica_id: usize, sl: Sl) {
        println!("Sending Response ....");
        {
            let mut voter = voter_lock.lock();
            voter.components[SRV_IDX].replicas[replica_id].write(data);
            voter.components[SRV_IDX].replicas[replica_id].state_transition(voter_lib::ReplicaState::Blocked);
        }
        sl.block();
    }

    pub fn srv_wait(voter_lock: &Lock<Voter>, replica_id: usize, sl: Sl) {
        {
            let mut voter = voter_lock.lock();
            voter.components[SRV_IDX].replicas[replica_id].state_transition(voter_lib::ReplicaState::Blocked);
        }
        sl.block();
    }
}
