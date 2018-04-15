use super::types::{vaddr_t, cbuf_t};
use libc::c_ulong;


extern {
	pub fn memmgr_shared_page_alloc(pgaddr:&*mut vaddr_t) -> i32;
	pub fn memmgr_shared_page_map(id: i32, pgaddr:&*mut vaddr_t) -> i32;
}
