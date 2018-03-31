use std::fmt::Arguments;
use std::io::{Result, Write};
use std::mem::transmute;

use libc::{c_void, fwrite};

use libc_extra::stdout;

struct Printf;

impl Write for Printf {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        Ok(unsafe{
            fwrite(buf.as_ptr() as *const c_void, 1, buf.len(), transmute(stdout))
        })
    }

    // Composite printf auto flushes
    fn flush(&mut self) -> Result<()> {
        return Ok(())
    }
}

pub fn print_args(args: Arguments) {
    Printf.write_fmt(args).expect("Cannot fail to write to stdout!");
}