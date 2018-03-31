use libc::c_int;

use super::types::thdid_t;

#[repr(C)]
#[allow(non_camel_case_types, dead_code)]
pub struct sl_lock {
    pub holder: thdid_t
}

// Provided by the "friend" C file
extern {
    // You'd think these should be 'mut' pointers, but we can't use those
    // Locks are only useful if they're shared, and we can't share *mut ptrs
    pub fn sl_lock_take_rs(lock: *const sl_lock);

    pub fn sl_lock_release_rs(lock: *const sl_lock);
}

// From "sl_lock.h"
extern {
    pub fn sl_lock_init(lock: *mut sl_lock);

    pub fn sl_lock_try_take(lock: *const sl_lock) -> c_int;

    #[allow(dead_code)]
    pub fn sl_lock_holder(lock: *const sl_lock) -> thdid_t;
}