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
use libc::c_int;


extern {
    fn get_request_data(id:types::spdid_t) -> *mut c_int;
    fn cos_inv_token_rs() -> types::spdid_t;
}

#[no_mangle]
pub extern "C" fn replica_done_initializing_rust() {
    //Not sure how to make this function visible to C
    voter::Voter::replica_done_initializing();
}

/* FFI Bug - parameters passed to this function get corrupted */
#[no_mangle]
pub extern "C" fn replica_request() -> [u8; voter_config::BUFF_SIZE] {
    let (msg,opcode,replica_id);

    unsafe {
        replica_id = cos_inv_token_rs();

        let request_data = get_request_data(replica_id);
        let size = *request_data;
        opcode = *request_data.offset(1);
        let msg_ptr:*mut u8 = *request_data.offset(2) as *mut u8;

        msg = slice::from_raw_parts_mut(msg_ptr, size as usize);
    }

    voter::Voter::request(msg,opcode,replica_id)
}

#[no_mangle]
pub extern "C" fn rust_init() {
    let api = unsafe { DefKernelAPI::assert_already_initialized() };
    Sl::start_scheduler_loop_without_initializing(api, voter::voter_config::REP_PRIO, move |sl: Sl| {
        println!("Entered Scheduling loop\n");

        voter::Voter::initialize(sl);
    });
}

