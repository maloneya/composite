use std::boxed::FnBox;
use std::mem;
use std::time::Duration;

use libc::c_void;

use super::kernel_api::DefKernelAPI;
use super::sys::sl;
use super::sys::types;

// The friend C file should provide these symobls
extern {
    fn assign_thread_data(thd: *mut sl::sl_thd);
    fn schedinit_child() -> i32;
}


#[derive(Clone, Copy, Debug)]
pub struct Sl;
impl !Send for Sl{}
impl !Sync for Sl{}

#[derive(Clone)]
pub struct Thread {
    pub thread_id: types::thdid_t,
}

impl PartialEq<Self> for Thread {
    fn eq(&self, other: &Self) -> bool {
        self.thdid() == other.thdid()
    }
}

impl Eq for Thread {}


#[derive(Clone, Copy, Debug)]
pub enum ThreadParameter {
    Priority(u32)
}

impl Sl {
    pub fn start_scheduler_loop<F: FnBox(Sl) + Send + 'static>(d: DefKernelAPI, root_thread_priority: u32, entrypoint: F) {
        unsafe {
            sl::sl_init();
        }
        Self::start_scheduler_loop_without_initializing(d, root_thread_priority, entrypoint);
    }
    pub fn start_scheduler_loop_without_initializing<F: FnBox(Sl) + Send + 'static>(_: DefKernelAPI, root_thread_priority: u32, entrypoint: F) {
        let mut root_thread = Sl.spawn(entrypoint);
        root_thread.set_param(ThreadParameter::Priority(root_thread_priority));
        unsafe {
            sl::sl_sched_loop()
        }
    }
    pub fn child_start_scheduler_loop_without_initializing<F: FnBox(Sl) + Send + 'static>(_: DefKernelAPI, root_thread_priority: u32, entrypoint: F) {
        let mut root_thread = Sl.spawn(entrypoint);
        root_thread.set_param(ThreadParameter::Priority(root_thread_priority));
        unsafe {
            println!("caling schedinit_child");
            schedinit_child();
            sl::sl_sched_loop()
        }
    }

    pub fn start_scheduler_loop_nonblock_without_initializing<F: FnBox(Sl) + Send + 'static>(_: DefKernelAPI, root_thread_priority: u32, entrypoint: F) {
        let mut root_thread = Sl.spawn(entrypoint);
        root_thread.set_param(ThreadParameter::Priority(root_thread_priority));

        unsafe {
            sl::sl_sched_loop_nonblock()
        }
    }

    pub unsafe fn assert_scheduler_already_started() -> Sl {
        Sl
    }

    pub fn block(&self) {
        unsafe {
            sl::sl_thd_block(0);
        }
    }

    pub fn block_for(&self, duration: Duration) {
        let seconds = duration.as_secs();
        let extra_nanos = duration.subsec_nanos() as u64;
        let microseconds = seconds * (1000 * 1000) + (extra_nanos / 1000);

        let duration_in_cycles = unsafe {
            sl::sl_usec2cyc_rs(microseconds)
        };

        let absolute_timeout = unsafe {
            sl::sl_now_rs() + duration_in_cycles
        };

        unsafe {
            sl::sl_thd_block_timeout(0, absolute_timeout);
        }
    }


    pub fn current_thread(&self) -> Thread {
        Thread {
            thread_id: unsafe {
                sl::sl_thdid_rs()
            }
        }
    }

    pub fn thd_yield(&self) {
        unsafe {
            sl::sl_thd_yield(0);
        }
    }

    pub fn spawn<F: FnBox(Sl) + Send + 'static>(&self, entrypoint: F) -> Thread {
        let boxed_fn = Box::new(FnBoxWrapper {
            inner: Box::new(entrypoint)
        });

        unsafe {
            let thd_ptr = sl::sl_thd_alloc(closure_spawn_wrapper, Box::into_raw(boxed_fn) as *mut c_void);
            assert_ne!(thd_ptr,0 as *mut sl::sl_thd);
            assign_thread_data(thd_ptr);

            Thread {
                thread_id: sl::sl_thd_thdid_rs(thd_ptr),
            }
        }
    }
}

impl Thread {
    pub fn set_param(&mut self, param: ThreadParameter) {
        unsafe {
            sl::sl_thd_param_set(sl::sl_thd_lkup_rs(self.thread_id), param.to_u32())
        }
    }

    pub fn wakeup(&mut self) {
        unsafe {
            sl::sl_thd_wakeup(self.thdid())
        }
    }

    pub fn thdid(&self) -> types::thdid_t {
        self.thread_id
    }

    pub fn get_state(&self) -> sl::sl_thd_state {
        unsafe {
            let thd_ptr = sl::sl_thd_lkup_rs(self.thread_id);
            (*thd_ptr).state.clone()
        }
    }
}

impl ThreadParameter {
    fn to_u32(&self) -> u32 {
        match self {
            &ThreadParameter::Priority(priority) => unsafe {
                sl::sched_param_pack_rs(sl::sched_param_type_t::SCHEDP_PRIO, priority)
            }
        }
    }
}

// Unsafe magic to support spawning a closure as a new thread

// It would be nice to just use a Box<FnBox(Sl)>, and just pass *mut FnBox(Sl) to the thread
// But we can't do that, because 'FnBox(Sl)' is a trait, and thus *mut FnBox(Sl) is a double wide
// fat pointer. Therefore we have to use this wrapper, so we can use a thin pointer
struct FnBoxWrapper<'a>{
    inner: Box<FnBox(Sl) + 'a>
}

extern fn closure_spawn_wrapper(ptr: *mut c_void) {
    // We use the c_void ptr to find the real entrypoint
    let boxed_wrapper = unsafe {
        // This is the only crazy unsafe thing we do
        let wrapper_ptr: *mut FnBoxWrapper = mem::transmute(ptr);

        // Once we get the wrapper ptr, we need to re-box it so we don't leak memory
        Box::from_raw(wrapper_ptr)
    };
    let inner_box: Box<FnBox(Sl)> = boxed_wrapper.inner;
    inner_box(Sl);

    // When the inner closure returns, the thread is done executing, so we can free it
    unsafe {
        sl::sl_thd_free(sl::sl_thd_curr_rs());
    }
}
