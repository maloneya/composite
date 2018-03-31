use std::panic;


pub fn trace_init() {
	// panic::set_hook(Box::new(|_| {
	// 	//let fp: *const _;
	// 	let mut fp:u32 = 0;
	// 	unsafe {
	// 		asm!("movl %%ebp, %[fp]"
	// 			: /* no outputs */
	// 			: "fp"(fp)
	// 			: "ebp"
	// 			);
	// 	}
	// 	println!("Got fp: {}",fp);
	// }));
}


