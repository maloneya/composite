// We kinda need to run on nightly to make xargo work
// However, it would be nice to avoid having to depend on unstable features
// TODO: Investigate switching to marker types
#![feature(optin_builtin_traits)]
// TODO: Remove uses of FnBox once Box<FnOnce()> just works
#![feature(fnbox)]
//used to read frame pointer from %ebp for stack trace
#![feature(asm)]

extern crate libc;

extern crate libc_extra;

#[macro_export]
macro_rules! printc {
    ($($arg:tt)*) => ($crate::print_impl::print_args(format_args!($($arg)*)));
}

pub mod sys;

pub mod kernel_api;
pub mod memmgr_api;
pub mod print_impl;
pub mod sl;
pub mod sl_lock;
pub mod panic_trace;
