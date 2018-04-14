#![allow(non_camel_case_types, dead_code)]

use libc::{c_ushort, c_ulong, uint32_t};


pub type cycles_t = u64;
pub type microsec_t = u64;

pub type tcap_prio_t = u64;
pub type thdid_t = c_ushort;

pub type spdid_t = c_ushort;

pub type vaddr_t = c_ulong;
pub type cbuf_t = uint32_t; 

// Capability types
pub type tcap_res_t = c_ulong;
pub type capid_t = c_ulong;
pub type tcap_t = capid_t;
pub type thdcap_t = capid_t;
pub type arcvcap_t = capid_t;
pub type pgtblcap_t = capid_t;
pub type asndcap_t = capid_t;
