use super::types::{vaddr_t, cbuf_t};
use libc::c_ulong;


extern {
	pub fn memmgr_shared_page_alloc(pgaddr:&*mut vaddr_t) -> cbuf_t;
	pub fn memmgr_shared_page_map(id: cbuf_t,pgaddr:&*mut vaddr_t) -> c_ulong;
}