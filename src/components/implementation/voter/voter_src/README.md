# Voter Documentation 

Documentation Last Update: 4/29/18

## Overview 
The “Voter” is a composite component written in rust designed to  monitor redundant applications. Applications are booted and scheduled through the Voter component using the hierarchical scheduler api and the SL library.  The redundancy is defined manually within the voter component’s runscript. 

Applications running through the voter will get resources from the voter rather then the typical resource provider. Because of this the voter must export the same interface as what ever the application expects. 

An application’s resource requests are considered to be its “vote”, each time the replicas request resources we look at these “votes” to determine what the accepted state of the system is.  if any replica voted differently then the majority (that is requested a different resource) we mark it as faulted. 

Currently faulted replicas are simply blocked indefinitely and we continue to run with the remaining healthy replicas; However, at this point a recovery mechanism could be implemented that cloned one of the healthy replicas over the faulted one. 

_Component Structure_
C files 
`components/implementation/voter/voter_src/:`
* main.c - Handles component initialization and replica booting. Implements the `request(op,size,args)`  synchronous invocation allowing resource requests to be passed to the voter.  [src](https://github.com/maloneya/composite/blob/rk_voter/src/components/implementation/voter/voter_src/main.c)
* application_interface.h - specifies all the operations the voter can preform on behalf of its application. i.e. - `read(); write();`  [src](https://github.com/maloneya/composite/blob/rk_voter/src/components/implementation/voter/voter_src/application_interface.h)

Voter Rust Crate
`components/implementation/voter/voter_src/rust_src/src/:`
* lib.rs - rust entry ponts [src](https://github.com/maloneya/composite/blob/rk_voter/src/components/implementation/voter/voter_src/rust_src/src/lib.rs)
* voter/mod.rs - voter main file, initialization, monitoring and requesting live here [src](https://github.com/maloneya/composite/blob/rk_voter/src/components/implementation/voter/voter_src/rust_src/src/voter/mod.rs)
* voter/server_bindings.rs - Request demultiplexer, unpacks requests and contacts the server to make them on behalf of the replicated application. Must match application_interface.h [src](https://github.com/maloneya/composite/blob/rk_voter/src/components/implementation/voter/voter_src/rust_src/src/voter/server_bindings.rs)
* voter/voter_config.rs - defines global values for the voter [src](https://github.com/maloneya/composite/blob/rk_voter/src/components/implementation/voter/voter_src/rust_src/src/voter/server_config.rs)
* voter/voter_lib.rs - implements voter mechanisms such as replica state checks, vote comparison, and vote collection [src](https://github.com/maloneya/composite/blob/rk_voter/src/components/implementation/voter/voter_src/rust_src/src/voter/voter_lib.rs)

Voter component interface 
`components/interface/voter/: `
* voter.h - client interface an application must include to call available operations in the voter. includes client stubs to set up shared memory with the voter. must match application_interface.h [src](https://github.com/maloneya/composite/blob/rk_voter/src/components/interface/voter/voter.h)

Dependencies
* lib_composite
* lazy_static 
* lib_rk (if the voter needs to make resource requests to the rump kernel) 

## Voting Mechanism
Application monitoring is done via round robin scheduling. Each replica runs for a scheduler quantum then the voter thread runs and checks the state of each replica. When a replica makes a request the requesting thread id is stored (we don’t keep this permanently because the requesting thread id could change for future requests in multithreaded applications) .

All resource requests that the voter exports are multiplexed via the `request(op, size, args)` function in main.c. This function passes the request data into the voter and packs it into the replica’s “vote” which is represented as an array.  Once the vote is created the voter blocks the replica.

When a vote is collected, if any of the replicas are still processing we return inconclusive, and continue around the scheduling loop again.  If only one replica remains in the processing state we track how many consecutive scheduling quantums it does so. If this exceeds a value defined in the voter config file we mark that replica as faulted as it has likely encountered an error that is causing it to spin indefinitely. 

Once all threads have made the request and block we proceed to compare each of their votes. If all are the same we return a success and proceed to make the resource request on behalf of the replicas. If one is found to be different then the others we mark that replica as faulted and return a failed vote. 

Once the resource request returns to the voter we pass it along to each replica via shared memory or simply a function return value (depending on what the request expects).  The replica threads are woken up and we begin the process again. 

## Application Server Interface
Configuring the voter to accept requests on behalf of a given server is done in two parts:
1. **The client -> voter interface** - The voter must mimic the interface of what ever server the application typically expects. The client interface is defied in `interface/voter.h (client) `and `voter/application_interface.h (voter)` . These files must provide the same set of synchronous invocations as the server.  
2. **The voter -> server interface** - The voter must interface with the server, demultiplexing a replicas vote and calling out to the correct server resource providing function. This is done in -  `voter/rust/server_bindings.rs `

Requests are multiplexed into the voter via `request(op, size, args)` and demultiplexed back out to the server via `handle_request(vote,server_shared_mem)` 

request takes a opcode, the size of data written to the replica-voter shared memory, and an array of function arguments. These are packed into the replicas vote, and once all replicas are verified to have made the same request, the vote is passed to handle request. handle request directs the data based on the opcode to a function that knows how to unpack the data and correctly pass it down to the server. 

To use the voter with a new server you must implement the interface multiplexing and demultiplexing protocol in these three files. For example if you wanted to implement `write(int fd, void* buf, int count)`  it would look something like this: 

```
/********* voter.h **********/ 
/* voter sinv */ 
int _voter_write(int fd, int count);

/* client stub */ 
int
voter_write(int fd, void *buf, int count)
{
	assert(fd);

	_shdmem_write(buf,count);
	return _voter_write(fd,count);
}

/********* application_interface.h **********/ 
#define WRITE 0
int
_voter_write(int fd, int count)
{
	int args[MAX_ARGS];
	pack_args(args,fd,0,0);

	return request(WRITE, count, args);
}


/********** server_bindings.rs ***********/
const WRITE:u8 = 0;
pub fn handle_request(vote,shrdmem) -> (i32,bool) {
  ...
    match op {
        WRITE  => write(data, server_shrdmem),
        _ => panic!("op {:?} not supported", op),
    }
}

fn write(vote,shrdmem) -> (i32,bool) {
    println!("voter performing write");

    let size = data[SIZE] as usize;
    let fd = data[ARGS] as i32;
    let copy_len = data.len() - DATA;
    - COPY DATA TO SERVER SHARD MEM -
	  let ret = SERVER::write(fd,server_shrdmem.id,size);
    (ret,false)
}
```
