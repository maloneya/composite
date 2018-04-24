use super::sys::memmgr::{memmgr_shared_page_alloc,memmgr_shared_page_map};
use super::sys::types::vaddr_t;
use std::slice;

const PAGE_SIZE:usize = 4096;

#[derive(Debug)]
pub struct SharedMemoryRegion {
	pub id: i32,
	pub mem: Box<[u8]>,
}

impl SharedMemoryRegion {
	pub fn page_alloc() -> SharedMemoryRegion {
		unsafe {
			let addr: *mut vaddr_t = 0 as *mut vaddr_t;
			let id:i32;
			id = memmgr_shared_page_alloc(&addr);
			assert!(id > -1 && addr != 0  as *mut vaddr_t);

            let slice = slice::from_raw_parts_mut(addr as *mut u8,PAGE_SIZE);

			SharedMemoryRegion {
				id,
				mem: Box::from_raw(slice),
			}
		}
	}

	pub fn page_map(id: i32) -> SharedMemoryRegion {
		unsafe {
			let addr: *mut vaddr_t = 0 as *mut vaddr_t;
			let ret = memmgr_shared_page_map(id,&addr);
			assert!(ret > -1 && addr != 0  as *mut vaddr_t);

			let slice = slice::from_raw_parts_mut(addr as *mut u8,PAGE_SIZE);

			SharedMemoryRegion {
				id,
				mem: Box::from_raw(slice),
			}
		}
	}
}
