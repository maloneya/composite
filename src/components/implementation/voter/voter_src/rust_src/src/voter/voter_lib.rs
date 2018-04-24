use lib_composite::sl::{Sl, Thread};
use lib_composite::memmgr_api::SharedMemoryRegion;
use lib_composite::sys::types;
use std::fmt;
use voter::*;
use voter::voter_config::*;

#[derive(PartialEq)]
pub enum VoteStatus {
    Fail(types::spdid_t),              /*stores divergent replica id*/
    Inconclusive(u8, types::spdid_t),  /*number of replicas in processing state, id of replica in processing*/
    Success,
}

pub struct Replica {
    pub id:  types::spdid_t,
    pub thd: Option<Thread>,
    pub shrdmem: Option<SharedMemoryRegion>,
    pub data_buffer: [u8; BUFF_SIZE],
    pub ret: Option<i32>,
    pub faulted: bool,
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
            "Replica: [id:{} tid:{:?}]",
            self.id,
            match self.thd.as_ref() {
                None => format!("None"),
                Some(thd) => format!("{}",thd.thread_id),
            }

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
                &VoteStatus::Success => format!("Success"),
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
    pub fn new() -> Replica {
        Replica {
            thd: None,
            id: 0,
            shrdmem: None,
            data_buffer: [0; BUFF_SIZE],
            ret: None,
            faulted: false
        }
    }

    pub fn is_processing(&self) -> bool {
        if self.faulted {return false;}
        let thd = self.thd.as_ref();
        if thd.is_some() {
            return thd.unwrap().get_state() == sl_thd_state::SL_THD_RUNNABLE;
        }
        true /* If the replica has no thread nothing has made a request so its processing */
    }

    pub fn is_blocked(&self) -> bool {
        if self.faulted {return true;}
        let thd = self.thd.as_ref();
        if thd.is_some() {
            return thd.unwrap().get_state() == sl_thd_state::SL_THD_BLOCKED;
        }
        false
    }

    pub fn recover(&mut self) {
        println!("Replica {} faulted",self.id);
        self.faulted = true;
    }

    pub fn request(&mut self, op:i32, data_size: usize, args:[u8;MAX_ARGS], sl: Sl) {
        let data_start = 2+MAX_ARGS;
        let data_end = data_start + PAGE_SIZE;
        /* pack replica data buffer with request information */
        self.data_buffer[0] = op as u8;
        self.data_buffer[1] = data_size as u8;
        self.data_buffer[2..2+MAX_ARGS].copy_from_slice(&args[..]);
        self.data_buffer[data_start..data_end].copy_from_slice(&self.shrdmem.as_mut().unwrap().mem[..]);
        self.thd = Some(Thread {thread_id: sl.current_thread().thdid()});

        println!("request Vote {:?}",&self.data_buffer[0..10]);
    }
}

impl Component {
    pub fn new(num_replicas: i32) -> Component {
        let num_replicas = num_replicas as usize;
        assert!(num_replicas <= MAX_REPS);

        let mut replicas = Vec::new();
        for _i in 0..num_replicas {
            replicas.push(Replica::new());
        }

        Component {
            replicas: replicas,
            num_replicas,
            new_data: false,
        }
    }

    pub fn get_replica_by_spdid(& mut self, spdid: types::spdid_t) -> Option<& mut Replica> {
        let mut found = MAX_REPS + 1;
        for (i, replica) in (&mut self.replicas).iter_mut().enumerate() {
            if replica.id == spdid {
                found = i;
                break;
            }
        }

        if found != MAX_REPS + 1 {
            Some(&mut self.replicas[found])
        }
        else {
            None
        }
    }

    pub fn replicas_initialized(&self) -> bool {
        for replica in &self.replicas {
            if replica.id == 0 {
                return false
            }
        }
        true
    }

    pub fn wake_all(&mut self) {
        for replica in &mut self.replicas {
            if replica.faulted {continue;}
            replica.thd.take().unwrap().wakeup();
        }
    }

    pub fn collect_vote(&mut self) -> VoteStatus {
        let mut processing_replica_id = 0;
        let mut num_processing = 0;

        for replica in &self.replicas {
            if replica.is_processing() {
                num_processing += 1;
                /* rep id is only useful in the case where only 1 replica is still processing
                 * so it would only be set once in that case.
                 */
                processing_replica_id = replica.id;
            }
        }

        /* if any of the replicas are still processing vote inconclusive. */
        if num_processing > 0 {
            return VoteStatus::Inconclusive(num_processing, processing_replica_id);
        }

        /* Check that replicas are blocked */
        for replica in &self.replicas {
            if !replica.is_blocked() {
                return VoteStatus::Inconclusive(num_processing, replica.id);
            }
        }

        /* check the request each replica has made */
        if !self.validate_msgs() {
            return VoteStatus::Fail(self.find_faulted_msg());
        }

        return VoteStatus::Success;
    }

    pub fn validate_msgs(&self) -> bool {
        /* compare each message against the first to look for difference (handle detecting fault later) */

        let mut healthy_msg_id = MAX_REPS + 1;
        for i in 0..MAX_REPS {
            if !self.replicas[i].faulted {
                healthy_msg_id = i;
                break;
            }
        }
        assert_ne!(healthy_msg_id,MAX_REPS + 1);

        let ref mut msg = &self.replicas[healthy_msg_id].data_buffer;

        for replica in &self.replicas {
            if replica.faulted {continue;}
            if !compare_msgs(msg, &replica.data_buffer) {
                return false;
            }
        }

        true
    }

    pub fn find_faulted_msg(&self) -> types::spdid_t {
        /* store the number of replicas that agree, and rep id of sender */
        let mut concensus: [u8; MAX_REPS] = [0; MAX_REPS];

        /* find which replica disagrees with the majority */
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

        /* go through consensus to get the rep id that sent the msg with least agreement */
        let mut min: u8 = (MAX_REPS + 1) as u8;
        let mut faulted:usize = MAX_REPS + 1;
        for (rep_idx, msg_votes) in concensus.iter().enumerate() {
            if *msg_votes < min {
                min = *msg_votes;
                faulted = rep_idx;
            }
        }

        assert_ne!(faulted,MAX_REPS + 1);
        return self.replicas[faulted].id;
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
