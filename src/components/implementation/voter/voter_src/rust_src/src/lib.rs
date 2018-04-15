mod voter;
extern crate libc;
#[macro_use]
extern crate lazy_static;
extern crate lib_composite;

use lib_composite::kernel_api::DefKernelAPI;
use lib_composite::sl::Sl;
use lib_composite::sys::types;
use voter::voter_config;
use libc::c_int;

extern {
    fn cos_inv_token_rs() -> types::spdid_t;
}

#[no_mangle]
pub extern "C" fn replica_done_initializing_rust(shdmem_id: i32) {
    //Not sure how to make this function visible to C
    voter::Voter::replica_done_initializing(shdmem_id);
}

/* FFI Bug - parameters passed to this function get corrupted */
#[no_mangle]
pub extern "C" fn replica_request(opcode: c_int, data_size: c_int, args_ptr:*mut c_int) {
    let sl = unsafe {Sl::assert_scheduler_already_started()};
    let replica_id = unsafe {cos_inv_token_rs()};

    let mut args = [0;voter_config::MAX_ARGS];
    unsafe {
        for i in 0..voter_config::MAX_ARGS as usize {
            args[i] = *args_ptr.offset(i as isize) as u8;
        }
    }

    voter::Voter::request(replica_id,opcode,data_size as usize,args,sl);
}

#[no_mangle]
pub extern "C" fn rust_init() {
    let api = unsafe { DefKernelAPI::assert_already_initialized() };
    Sl::start_scheduler_loop_without_initializing(api, voter::voter_config::REP_PRIO, move |sl: Sl| {
        println!("Entered Scheduling loop\n");

        voter::Voter::initialize(sl);
    });
}

