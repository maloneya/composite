use lib_composite::sl::Sl;
use lib_composite::sl_lock::Lock;

use voter::voter_config::BUFF_SIZE;
use voter::Voter;
use lazy_static;

lazy_static! {
    static ref VOTER:Lock<Voter> = unsafe {
        Lock::new(Sl::assert_scheduler_already_started(),
                  Voter::new(3,do_work,Sl::assert_scheduler_already_started())
    )};
}

#[derive(Debug)]
enum TestMode {
    Healthy,
    Stuck,
    BadState,
}

const MODE: TestMode = TestMode::Healthy;

pub fn start(sl: Sl) {
    println!("Test app initializing in mode {:?}", MODE);
    Voter::monitor_application(&*VOTER, sl);
}

/************************ Application *************************/
fn do_work(sl: Sl, rep_id: usize) {
    println!("Replica {:?} starting work ....", rep_id);
    match MODE {
        TestMode::Healthy => healthy(sl, rep_id),
        TestMode::Stuck => stuck(sl, rep_id),
        TestMode::BadState => bad_state(sl, rep_id),
    }
}

fn healthy(sl: Sl, rep_id: usize) {
    let mut i = 0;
    loop {
        if i % 100 == 0 {
            make_systemcall(i / 100, rep_id, sl);
            println!("Replica {:?} resuming work ....", rep_id);
        }
        i += 1;
    }
}

fn stuck(sl: Sl, rep_id: usize) {
    let mut i = 0;
    loop {
        if rep_id == 0 {
            continue;
        }
        if i % 100 == 0 {
            make_systemcall(i / 100, rep_id, sl);
            println!("Replica {:?} resuming work ....", rep_id);
        }
        i += 1;
    }
}

fn bad_state(sl: Sl, rep_id: usize) {
    let mut i = 0;
    loop {
        if i % 100 == 0 {
            if rep_id == 0 {
                make_systemcall(0, rep_id, sl);
            } else {
                make_systemcall(1, rep_id, sl);
            }
            println!("Replica {:?} resuming work ....", rep_id);
        }
        i += 1;
    }
}

fn make_systemcall(sys_call: u8, rep_id: usize, sl: Sl) {
    println!("Replica {:?} making syscall {:?}", rep_id, sys_call);

    let data: [u8; BUFF_SIZE] = [sys_call; BUFF_SIZE];
    println!("Rep got {:?}", Voter::request(&*VOTER, data, rep_id, sl)[0]);
}