mod voter;

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate lib_composite;

use lib_composite::kernel_api::DefKernelAPI;
use lib_composite::sl::Sl;
use lib_composite::panic_trace;
use lib_composite::sl::ThreadParameter;

 extern {
 	fn print_hack(n: i8);
 }


#[no_mangle]
pub extern "C" fn test_call_rs() {
	unsafe {print_hack(1);}
    println!("Executing test call in Rust");
}

#[no_mangle]
pub extern "C" fn rust_init() {
    let api = unsafe { DefKernelAPI::assert_already_initialized() };
    Sl::start_scheduler_loop_without_initializing(api, voter::voter_config::REP_PRIO, move |sl: Sl| {
        println!("Entered Scheduling loop\n");

        voter::Voter::monitor_application(sl);
    });
}

