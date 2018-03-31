use libc::{c_char, c_int};

extern {
    pub fn cos_llprint(s: *const c_char, len: c_int);
    pub fn printc(s: *const c_char, ...) -> c_int;
}