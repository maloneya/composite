use super::sys::memmgr::{memmgr_shared_page_alloc,memmgr_shared_page_map};
use super::sys::types::{cbuf_t,vaddr_t};
use std::slice;

const PAGE_SIZE:usize = 4096;

#[derive(Debug)]
pub struct SharedMemoryReigon {
	pub id: cbuf_t,
	pub mem: Box<[u8]>,
}

impl SharedMemoryReigon {
	pub fn page_alloc() -> SharedMemoryReigon {
		unsafe {
			let addr: *mut vaddr_t = 0 as *mut vaddr_t;
			let id:cbuf_t; 
			id = memmgr_shared_page_alloc(&addr);
			assert!(id > 0 && addr != 0  as *mut vaddr_t);

            let slice = slice::from_raw_parts_mut(addr as *mut u8,PAGE_SIZE);

			SharedMemoryReigon {
				id, 
				mem: Box::from_raw(slice),
			}
		}
	}

	pub fn page_map(id: u32) -> SharedMemoryReigon {
		unsafe {
			let addr: *mut vaddr_t = 0 as *mut vaddr_t;
			let ret = memmgr_shared_page_map(id as cbuf_t,&addr);
			assert!(ret > 0 && addr != 0  as *mut vaddr_t);

			let slice = slice::from_raw_parts_mut(addr as *mut u8,PAGE_SIZE);

			SharedMemoryReigon {
				id, 
				mem: Box::from_raw(slice),
			}
		}
	}
}