mod voter;
extern crate libc;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate lib_composite;

use lib_composite::kernel_api::DefKernelAPI;
use lib_composite::sl::Sl;
use lib_composite::panic_trace;
use lib_composite::sys::types;
use voter::voter_config;
use std::slice;
use libc::{c_int,c_ulong};

extern {
    fn cos_inv_token_rs() -> types::spdid_t;
}

#[no_mangle]
pub extern "C" fn replica_done_initializing_rust(addr: c_ulong) {
    //Not sure how to make this function visible to C
    voter::Voter::replica_done_initializing(addr as *mut u8);
}

/* FFI Bug - parameters passed to this function get corrupted */
#[no_mangle]
pub extern "C" fn replica_request(data_size: c_int, opcode: c_int) {
    let replica_id = unsafe {
         cos_inv_token_rs()
    };

    voter::Voter::request(data_size,opcode,replica_id);
}

#[no_mangle]
pub extern "C" fn rust_init() {
    let api = unsafe { DefKernelAPI::assert_already_initialized() };
    Sl::start_scheduler_loop_without_initializing(api, voter::voter_config::REP_PRIO, move |sl: Sl| {
        println!("Entered Scheduling loop\n");

        voter::Voter::initialize(sl);
    });
}

