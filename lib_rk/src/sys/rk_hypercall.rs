use libc::{c_int, size_t, c_uint, c_long};

extern {
    pub fn rk_write(fd:c_int, shdmem_id:c_int, size: size_t) -> c_long;
    pub fn rk_read(fd:c_int, shdmem_id:c_int, size: size_t) -> c_long;
    pub fn rk_socket(domain:c_int, type_arg:c_int, protocol:c_int) -> c_int;
    pub fn rk_bind(sockfd: c_int, shdmem_id:c_int, addrlen:c_uint) -> c_int;
    pub fn rk_accept(sockfd:c_int, shdmem_id:c_int) -> c_int;
    pub fn rk_listen(sockfd:c_int, backlog:c_int) -> c_int;
}