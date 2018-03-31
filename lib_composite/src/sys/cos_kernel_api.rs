use libc::c_ulong;

use super::types::{capid_t, pgtblcap_t, vaddr_t};

#[repr(C)]
#[derive(Clone, Copy)]
#[allow(non_camel_case_types)]
pub struct cos_meminfo {
    untyped_ptr: vaddr_t,
    umem_ptr: vaddr_t,
    kmem_ptr: vaddr_t,

    untyped_frontier: vaddr_t,
    umem_frontier: vaddr_t,
    kmem_frontier: vaddr_t,

    pgtbl_cap: pgtblcap_t
}

#[repr(C)]
#[derive(Clone, Copy)]
#[allow(non_camel_case_types)]
pub struct cos_compinfo {
    pgtbl_cap: capid_t,
    captbl_cap: capid_t,
    comp_cap: capid_t,

    cap_frontier: capid_t,
    caprange_frontier: capid_t,

    cap16_frontier: capid_t,
    cap32_frontier: capid_t,
    cap64_frontier: capid_t,

    vas_frontier: vaddr_t,
    vasrange_frontier: vaddr_t,

    memsrc: *mut cos_compinfo, /* might be self-referential */
    pub mi: cos_meminfo
}

// TODO: Declare the actual functions in this module
extern {
    pub fn cos_meminfo_init(mi: *mut cos_meminfo, untyped_ptr: vaddr_t, untyped_sz: c_ulong, pgtbl_cap: pgtblcap_t);
}