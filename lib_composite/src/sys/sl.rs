use libc::{c_int, c_uint, c_void};

use super::types::{cycles_t, microsec_t, tcap_prio_t, thdcap_t, thdid_t, tcap_t, asndcap_t, arcvcap_t, tcap_res_t};

#[repr(C)]
#[derive(Clone, Debug, PartialEq)]
#[allow(non_camel_case_types, dead_code)]
pub enum sl_thd_state{
    SL_THD_FREE = 0,
    SL_THD_BLOCKED,
    SL_THD_BLOCKED_TIMEOUT,
    SL_THD_WOKEN,
    SL_THD_RUNNABLE,
    SL_THD_DYING,
}

#[repr(C)]
#[derive(Clone, Debug)]
#[allow(non_camel_case_types)]
pub enum sl_thd_property {
    SL_THD_PROPERTY_OWN_TCAP = 1,
    SL_THD_PROPERTY_SEND     = (1<<1),
}

#[repr(C)]
#[derive(Clone, Debug)]
#[allow(non_camel_case_types)]
pub struct cos_aep_info {
    tc:         tcap_t,
    thd:        thdcap_t,
    pub thdid:  thdid_t,
    rcv:        arcvcap_t,
    func:       extern fn(rcv: arcvcap_t, param: c_void),
    data:       *mut c_void,
}


#[repr(C)]
#[derive(Clone, Debug)]
#[allow(non_camel_case_types)]
pub struct sl_thd {
    pub state:      sl_thd_state,
    properties:     sl_thd_property,
    pub aepinfo:    cos_aep_info,
    sndcap:         asndcap_t,
    prio:           tcap_prio_t,
    dependency:     *mut sl_thd,

    budget:         tcap_res_t,
    last_replenish: cycles_t,
    period:         cycles_t,
    periodic_cycs:  cycles_t,
    timeout_cycs:   cycles_t,
    wakeup_cycs:    cycles_t,
    timeout_idx:    c_int
}

#[allow(non_camel_case_types)]
pub type cos_thd_fn_t = extern fn(*mut c_void);

#[repr(u8)]
#[derive(Clone, Copy)]
#[allow(non_camel_case_types, dead_code)]
pub enum sched_param_type_t {
    SCHEDP_NOOP = 0,
    SCHEDP_PRIO,		/* fixed priority */
    SCHEDP_RPRIO,		/* priority relatively higher than current thread */
    /* priority relatively lower (not numerically) than current thread */
    SCHEDP_RLPRIO,
    SCHEDP_DEADLINE,	/* if != window */
    SCHEDP_BUDGET,		/* exec time */
    SCHEDP_WINDOW,     	/* period */
    SCHEDP_PROPORTION,	/* percent required */
    SCHEDP_WEIGHT,		/* proportion compared to other */
    SCHEDP_IDLE, 		/* idle thread: internal use only */
    SCHEDP_INIT, 		/* initialization threads: internal use only */
    SCHEDP_IPI_HANDLER,     /* IPI handler thread: internal use only */
    SCHEDP_TIMER, 		/* timer thread: internal use only */
    SCHEDP_CORE_ID,         /* create the thread on the target core */
    SCHEDP_MAX		/* maximum value */
}

#[allow(non_camel_case_types)]
pub type sched_param_t = u32;

// Provided by the "friend" C file
extern {
    pub fn sched_param_pack_rs(param_type: sched_param_type_t, value: c_uint) -> sched_param_t;
    pub fn sl_thd_curr_rs() -> *mut sl_thd;
    pub fn sl_thd_lkup_rs(tid: thdid_t) -> *mut sl_thd;
    pub fn sl_thdid_rs() -> thdid_t;
    pub fn sl_thd_thdid_rs(t: *mut sl_thd) -> thdid_t;


    pub fn sl_cyc2usec_rs(cyc: cycles_t) -> microsec_t;
    pub fn sl_usec2cyc_rs(usec: microsec_t) -> cycles_t;

    pub fn sl_now_rs() -> cycles_t;
    pub fn sl_now_usec_rs() -> microsec_t;
}

// From sl.c
#[allow(dead_code)]
extern {
    pub fn sl_init();
    pub fn sl_sched_loop();
    pub fn sl_sched_loop_nonblock();

    pub fn sl_thd_alloc(fun: cos_thd_fn_t, data: *mut c_void) -> *mut sl_thd;
    pub fn sl_thd_free(t: *mut sl_thd);

    pub fn sl_thd_param_set(t: *mut sl_thd, sp: sched_param_t);

    pub fn sl_thdid() -> thdid_t;

    pub fn sl_thd_lkup(tid: thdid_t) -> *mut sl_thd;

    pub fn sl_cs_enter();
    pub fn sl_cs_exit();

    pub fn sl_thd_block(tid: thdid_t);
    pub fn sl_thd_block_timeout(tid: thdid_t, abs_timeout: cycles_t) -> cycles_t;
    pub fn sl_thd_block_periodic(tid: thdid_t) -> c_uint;

    pub fn sl_thd_wakeup(tid: thdid_t);

    pub fn sl_thd_yield(tid: thdid_t);
}