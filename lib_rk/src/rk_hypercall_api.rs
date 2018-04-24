use super::sys::rk_hypercall;

pub fn write(fd:i32, shdmem_id:i32, size: usize) -> i32 {
	unsafe {
		rk_hypercall::rk_write(fd,shdmem_id,size)
	}
}

pub fn read(fd:i32, shdmem_id:i32, size: usize) -> i32 {
	unsafe {
		rk_hypercall::rk_read(fd,shdmem_id,size)
	}
}

pub fn socket(domain:i32, type_arg:i32, protocol:i32) -> i32 {
	unsafe {
		rk_hypercall::rk_socket(domain,type_arg,protocol)
	}
}

pub fn bind(sockfd: i32, shdmem_id:i32, addrlen:u32) -> i32 {
	unsafe {
		rk_hypercall::rk_bind(sockfd,shdmem_id,addrlen)
	}
}

pub fn accept(sockfd:i32, shdmem_id:i32) -> i32 {
	unsafe {
		rk_hypercall::rk_accept(sockfd,shdmem_id)
	}
}

pub fn listen(sockfd:i32, backlog:i32) -> i32 {
	unsafe {
		rk_hypercall::rk_listen(sockfd,backlog)
	}
}

