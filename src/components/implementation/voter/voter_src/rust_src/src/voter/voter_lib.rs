use lib_composite::sl::{Sl, Thread, ThreadParameter};
use lib_composite::sys::types;
use std::fmt;
use voter::*;
use voter::voter_config::*;

#[derive(PartialEq)]
pub enum VoteStatus {
    Fail(usize),              /*stores divergent replica id*/
    Inconclusive(u8, usize),  /*number of replicas in processing state, id of replica in processing*/
    Success([u8; BUFF_SIZE]), /*agreed upon message*/
}

pub struct Replica {
    pub id: usize,
    pub thd: Thread,
    pub data_buffer: [u8; BUFF_SIZE],
}

pub struct Component {
    pub replicas: Vec<Replica>,
    pub num_replicas: usize,
    pub new_data: bool,
}

impl fmt::Debug for Replica {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Replica: [replica_id - {} Thdid - {}]",
            self.id,
            self.thd.thdid(),
        )
    }
}

impl fmt::Debug for VoteStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Status: {}",
            match self {
                &VoteStatus::Inconclusive(num_processing, rep) => format!("Inconclusive {}:{:?}", num_processing, rep),
                &VoteStatus::Success(consensus) => format!("Success: consensus request {:?}", consensus),
                &VoteStatus::Fail(rep) => format!("Fail - {:?}", rep),
            }
        )
    }
}

impl fmt::Debug for Component {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Component: num_replicas:{}", self.num_replicas)
    }
}

impl Replica {
    pub fn new(id: usize, mut thd: Thread) -> Replica {
        thd.set_param(ThreadParameter::Priority(voter_config::REP_PRIO));
        Replica {
            thd: thd,
            id,
            data_buffer: [0; BUFF_SIZE],
        }
    }

    pub fn is_processing(&self) -> bool {
        self.thd.get_state() == sl_thd_state::SL_THD_RUNNABLE
    }

    //TODO!
    pub fn recover(&mut self) {
        panic!("Replica {:?} must be recovered", self.id);;
    }

    pub fn write(&mut self, data: [u8; BUFF_SIZE]) {
        println!("rep {:?} write", self.id);
        for i in 0..BUFF_SIZE {
            self.data_buffer[i] = data[i];
        }
    }
}

impl Component {
    pub fn new(thread_ids: [types::thdid_t; MAX_REPS], sl: Sl) -> Component {

        let mut replicas = Vec::new();
        let mut num_replicas = 0;
        println!("{:?}", thread_ids);
        for thread_id in thread_ids.iter() {
            if *thread_id <= 0 {break}

            /* num_replicas doubling as rep_id here */
            replicas.push(Replica::new(num_replicas, Thread {thread_id: *thread_id}));
            num_replicas += 1;
        }

        Component {
            replicas: replicas,
            num_replicas,
            new_data: false,
        }
    }

    pub fn wake_all(&mut self) {
        for replica in &mut self.replicas {
            replica.thd.wakeup();
        }
    }

    pub fn collect_vote(&mut self) -> VoteStatus {
        let mut processing_replica_id = 0;
        let mut num_processing = 0;

        for replica in &self.replicas {
            if replica.is_processing() {
                num_processing += 1;
                //rep id is only useful in the case where only 1 replica is still processing
                //so it would only be set once in that case.
                processing_replica_id = replica.id;
            }
        }

        //if any of the replicas are still processing bail.
        if num_processing > 0 {
            return VoteStatus::Inconclusive(num_processing, processing_replica_id);
        }

        //check the request each replica has made
        if !self.validate_msgs() {
            let faulted = self.find_faulted_msg();
            assert!(faulted > -1);
            return VoteStatus::Fail(faulted as usize);
        }

        return VoteStatus::Success(self.replicas[0].data_buffer.clone());
    }

    pub fn validate_msgs(&self) -> bool {
        //compare each message against the first to look for difference (handle detecting fault later)
        let ref msg = &self.replicas[0].data_buffer;
        for replica in &self.replicas {
            if !compare_msgs(msg, &replica.data_buffer) {
                return false;
            }
        }

        true
    }
    //TODO return result
    pub fn find_faulted_msg(&self) -> i16 {
        //store the number of replicas that agree, and rep id of sender
        let mut concensus: [u8; MAX_REPS] = [0; MAX_REPS];

        //find which replica disagrees with the majority
        for i in 0..self.num_replicas {
            let msg_a = &self.replicas[i].data_buffer;
            for j in 0..self.num_replicas {
                if i == j {
                    continue;
                }

                let msg_b = &self.replicas[j].data_buffer;

                if compare_msgs(msg_a, msg_b) {
                    concensus[i] += 1;
                }
            }
        }
        //go through consensus to get the rep id that sent the msg with least agreement
        let mut min: u8 = 4;
        let mut faulted: i16 = -1;
        for (rep, msg_votes) in concensus.iter().enumerate() {
            if *msg_votes < min {
                min = *msg_votes;
                faulted = rep as i16;
            }
        }
        return faulted;
    }
}

pub fn compare_msgs(msg_a: &[u8; voter_lib::BUFF_SIZE], msg_b: &[u8; voter_lib::BUFF_SIZE]) -> bool {
    for i in 0..voter_lib::BUFF_SIZE {
        if msg_a[i] != msg_b[i] {
            return false;
        }
    }

    true
}
