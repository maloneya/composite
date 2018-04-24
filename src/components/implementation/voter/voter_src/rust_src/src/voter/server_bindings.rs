use lib_composite::memmgr_api::SharedMemoryRegion;
use lib_composite::sl_lock::Lock;
use std::ops::DerefMut;
use voter::voter_config::MAX_ARGS;
use libc::{c_int, size_t, c_uint, c_long};

const WRITE:u8   = 0;
const READ:u8    = 1;
const SOCKET:u8  = 2;
const BIND:u8    = 3;
const ACCEPT:u8  = 4;
const LISTEN:u8  = 5;

/* serialized_msg offsets */
const OP:usize = 0;
/* data slice  offsets */
const SIZE:usize = 0;
const ARGS:usize = 1;
const DATA:usize = ARGS + MAX_ARGS;

extern {
    pub fn rk_write(fd:c_int, shdmem_id:c_int, size: size_t) -> c_long;
    pub fn rk_read(fd:c_int, shdmem_id:c_int, size: size_t) -> c_long;
    pub fn rk_socket(domain:c_int, type_arg:c_int, protocol:c_int) -> c_int;
    pub fn rk_bind(sockfd: c_int, shdmem_id:c_int, addrlen:c_uint) -> c_int;
    pub fn rk_accept(sockfd:c_int, shdmem_id:c_int) -> c_int;
    pub fn rk_listen(sockfd:c_int, backlog:c_int) -> c_int;
}

/*
 * ret 1: i32: return value from sinv to rk
 * ret 2: bool: true if data from rk_shrdmem needs to be copied to each replica
 */
pub fn handle_request(serialized_msg: &[u8], server_shrdmem_lock: &Lock<SharedMemoryRegion>) -> (i32,bool) {
    let op = serialized_msg[OP];
    let data = &serialized_msg[OP+1..];
    println!("Voter making call:{:?}", op);

    let mut server_shrdmem = server_shrdmem_lock.lock();
    let server_shrdmem = server_shrdmem.deref_mut();

    match op {
        WRITE  => write(data, server_shrdmem),
        READ   => read(data, server_shrdmem),
        SOCKET => socket(data),
        BIND   => bind(data, server_shrdmem),
        ACCEPT => accept(data, server_shrdmem),
        LISTEN => listen(data),
        _ => panic!("op {:?} not supported", op),
    }
}

fn write(data: &[u8], server_shrdmem: &mut SharedMemoryRegion) -> (i32,bool) {
    println!("voter performing write");

    let size = data[SIZE] as usize;
    let fd = data[ARGS] as i32;

    /* calculate the length from where the data starts in the packed buffer */
    let copy_len = data.len() - DATA;
    server_shrdmem.mem[..copy_len].copy_from_slice(&data[DATA..]);
    let ret = unsafe {rk_write(fd,server_shrdmem.id as i32,size)} as i32;
    (ret,false)
}

fn read(data: &[u8], server_shrdmem: &mut SharedMemoryRegion) -> (i32,bool) {
    println!("voter reading");
    let size = data[SIZE] as usize;
    let fd = data[ARGS] as i32;

    let ret = unsafe {rk_read(fd,server_shrdmem.id as i32, size)} as i32;
    (ret,true)
}


fn socket(data: &[u8]) -> (i32,bool) {
    println!("voter socket");
    let domain   = data[ARGS] as i32;
    let type_arg = data[ARGS+1] as i32;
    let protocol = data[ARGS+2] as i32;

    println!("size {} domain {} type {} proto {}",data[SIZE], domain, type_arg,protocol);
    let ret = unsafe {rk_socket(domain,type_arg,protocol)} as i32;
    (ret,false)
}


fn bind(data: &[u8], server_shrdmem: &mut SharedMemoryRegion) -> (i32,bool) {
    println!("voter bind");
    let fd = data[ARGS] as i32;
    let addrlen = data[SIZE] as u32;

    /* calculate the length from where the data starts in the packed buffer */
    let copy_len = data.len() - DATA;
    server_shrdmem.mem[..copy_len].copy_from_slice(&data[DATA..]);
    let ret = unsafe {rk_bind(fd,server_shrdmem.id as i32,addrlen)} as i32;
    (ret,false)
}

fn accept(data: &[u8], server_shrdmem: &mut SharedMemoryRegion) -> (i32,bool) {
    println!("voter accept");

    let fd = data[ARGS] as i32;

    let mut ret = -1;
    while ret == -1 {
        ret = unsafe {rk_accept(fd, server_shrdmem.id as i32)} as i32;
    }
    (ret,true)
}

fn listen(data: &[u8]) -> (i32,bool) {
    println!("voter listen");
    let sockfd = data[ARGS] as i32;
    let backlog = data[ARGS + 1] as i32;

    let ret = unsafe {rk_listen(sockfd,backlog)};
    (ret,false)
}
