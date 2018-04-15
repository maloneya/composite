//RR priority of the replica threads
pub const REP_PRIO: u32 = 5;
//maximum number of consecutive scheduler quantoms we allow a replica to
//continue to process after other components in its component have reached
//consensus
pub const MAX_INCONCLUSIVE: u8 = 10;
//maximum number of replica composite components per voter component
pub const MAX_REPS: usize = 3;
//size of the data buffer passed between application and server components.
pub const BUFF_SIZE: usize = 32;
//largest number of arguments passed from incomming sinv reqeusts 
pub const MAX_ARGS: usize = 3;
