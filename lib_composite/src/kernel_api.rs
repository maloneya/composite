use libc::c_ulong;

use super::sys::cos_defkernel_api::{cos_compinfo_get, cos_defcompinfo, cos_defcompinfo_init, cos_defcompinfo_curr_get};
use super::sys::cos_kernel_api::{cos_compinfo, cos_meminfo_init};
use super::sys::types::{pgtblcap_t, vaddr_t};

// The "friend" C file should provide these symbols
extern {
    static boot_mem_km_base: vaddr_t;
    static cos_mem_kern_pa_sz: c_ulong;
    static boot_captbl_self_untyped_pt: pgtblcap_t;
}


#[derive(Clone, Copy, Debug)]
pub struct DefKernelAPI;
impl !Send for DefKernelAPI{}
impl !Sync for DefKernelAPI{}

impl DefKernelAPI {
    pub fn from_standard_boot_capabilities() -> DefKernelAPI {
        unsafe {
            let defci: *mut cos_defcompinfo = cos_defcompinfo_curr_get();
            let ci: *mut cos_compinfo    = cos_compinfo_get(defci);

            cos_meminfo_init(&mut (*ci).mi, boot_mem_km_base, cos_mem_kern_pa_sz, boot_captbl_self_untyped_pt);
            cos_defcompinfo_init();
        }

        DefKernelAPI
    }

    pub unsafe fn assert_already_initialized() -> DefKernelAPI {
        DefKernelAPI
    }
}