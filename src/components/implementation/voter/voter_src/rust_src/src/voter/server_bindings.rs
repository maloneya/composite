use lib_composite::memmgr_api::SharedMemoryReigon;
use voter::voter_config::BUFF_SIZE;
use voter::voter_config::MAX_ARGS;

const WRITE:u8   = 0;
const READ:u8    = 1;
const SOCKET:u8  = 2;
const BIND:u8    = 3;
const ACCEPT:u8  = 4;

/* serialized_msg offsets */
const OP:usize = 0;
/* data slize  offsets */
const SIZE:usize = 0; 
const ARGS:usize = 1; 
const DATA:usize = ARGS + MAX_ARGS;

/* 
 * ret 1: i32: return value from sinv to rk 
 * ret 2: bool: true if data from rk_shrdmem needs to be copied to each replica
 */
pub fn handle_request(serialized_msg: [u8; BUFF_SIZE], server_shrdmem: &mut SharedMemoryReigon) -> (i32,bool) {
    let op = serialized_msg[OP];
    let data = &serialized_msg[OP..];
    println!("Voter making call:{:?}", op);

    match op {
        WRITE  => write(data, server_shrdmem),
        READ   => read(data, server_shrdmem),
        SOCKET => socket(data, server_shrdmem),
        BIND   => bind(data, server_shrdmem),
        ACCEPT => accept(data, server_shrdmem),
        _ => panic!("op {:?} not supported", op),
    }
}

fn write(data: &[u8], server_shrdmem: &mut SharedMemoryReigon) -> (i32,bool) {
    println!("voter performing write");
    let size = data[SIZE];
    let fd = data[ARGS];

    server_shrdmem.mem.copy_from_slice(&data[DATA..]);
    // let ret = rk_write(fd,server_shrdmem.id,size);   
    let ret = 0;
    (ret,false)
}

fn read(data: &[u8], server_shrdmem: &mut SharedMemoryReigon) -> (i32,bool) {
    println!("voter reading");
    let size = data[SIZE];
    let fd = data[ARGS];

    // let ret = rk_read(fd,server_shrdmem.id, size);
    let ret = 0;
    (ret,true)
}


fn socket(data: &[u8], server_shrdmem: &mut SharedMemoryReigon) -> (i32,bool) {
    println!("voter socket");
    let domain   = data[ARGS];
    let _type    = data[ARGS+1];
    let protocol = data[ARGS+2];

    // let ret = rk_socket(domain,_type,protocol);
    let ret = 0;
    (ret,false)
}


fn bind(data: &[u8], server_shrdmem: &mut SharedMemoryReigon) -> (i32,bool) {
    println!("voter bind");
    let fd = data[ARGS];
    let addrlen = data[SIZE];

    server_shrdmem.mem.copy_from_slice(&data[DATA..]);
    // let ret = rk_bind(fd,server_shrdmem.id,addrlen);
    let ret = 0;
    (ret,false)
}

fn accept(data: &[u8], server_shrdmem: &mut SharedMemoryReigon) -> (i32,bool) {
    println!("voter accept");
    let fd = data[ARGS];

    server_shrdmem.mem.copy_from_slice(&data[DATA..]);
    
    // let ret = rk_accept(fd, server_shrdmem.id);
    let ret = 0;
    (ret,true)
}
