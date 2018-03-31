use libc::c_void;

use super::cos_kernel_api::cos_compinfo;
use super::types::{arcvcap_t, tcap_t, thdcap_t};


#[allow(non_camel_case_types)]
type cos_aepthd_fn_t = extern fn (arcv: arcvcap_t, data: *mut c_void);

#[repr(C)]
#[allow(non_camel_case_types)]
struct cos_aep_info {
    tc: tcap_t,
    thd: thdcap_t,
    rcv: arcvcap_t,
    fun: cos_aepthd_fn_t,
    data: *mut c_void
}

#[repr(C)]
#[allow(non_camel_case_types)]
pub struct cos_defcompinfo {
    pub ci: cos_compinfo,
    sched_aep: cos_aep_info
}

// TODO: Figure out if more of these functions need to be declared
extern {
    pub fn cos_defcompinfo_init();

    pub fn cos_defcompinfo_curr_get() -> *mut cos_defcompinfo;

    pub fn cos_compinfo_get(defci: *mut cos_defcompinfo) -> *mut cos_compinfo;
}