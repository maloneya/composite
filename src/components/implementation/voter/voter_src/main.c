#include <cos_component.h>
#include <cobj_format.h>

#include <cos_kernel_api.h>
#include <cos_defkernel_api.h>
#include <res_spec.h>
#include <voter.h>

#include <sl.h>
#include <sl_lock.h>
#include <sl_thd.h>

#include <locale.h>
#include <limits.h>
#include <pthread.h>
#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <sched.h>
#include <schedinit.h>
#include "../../interface/capmgr/memmgr.h"
#include "../../sched/sched_info.h"

#include "application_interface.h"

extern int parent_schedinit_child(void);

#define FIXED_PRIO 5
#define FIXED_BUDGET_MS 2000
#define FIXED_PERIOD_MS 10000

/* These are macro values rust needs, so we duplicate them here */
vaddr_t       boot_mem_km_base            = BOOT_MEM_KM_BASE;
unsigned long cos_mem_kern_pa_sz          = COS_MEM_KERN_PA_SZ;
pgtblcap_t    boot_captbl_self_untyped_pt = BOOT_CAPTBL_SELF_UNTYPED_PT;

/* This are wrappers for static inline functions that rust needs */
sched_param_t
sched_param_pack_rs(sched_param_type_t type, unsigned int value)
{
	return sched_param_pack(type, value);
}

struct sl_thd *
sl_thd_curr_rs()
{
	return sl_thd_curr();
}

thdid_t
sl_thdid_rs()
{
	return sl_thdid();
}

thdid_t
sl_thd_thdid_rs(struct sl_thd *t) {
	return sl_thd_thdid(t);
}

struct sl_thd *
sl_thd_lkup_rs(thdid_t tid)
{
	return sl_thd_lkup(tid);
}

microsec_t
sl_cyc2usec_rs(cycles_t cyc)
{
	return sl_cyc2usec(cyc);
}

cycles_t
sl_usec2cyc_rs(microsec_t usec)
{
	return sl_usec2cyc(usec);
}

cycles_t
sl_now_rs()
{
	return sl_now();
}

microsec_t
sl_now_usec_rs()
{
	return sl_now_usec();
}

void
sl_lock_take_rs(struct sl_lock *lock)
{
	return sl_lock_take(lock);
}

void
sl_lock_release_rs(struct sl_lock *lock)
{
	return sl_lock_release(lock);
}

spdid_t
cos_inv_token_rs() {
	return cos_inv_token();
}

void
print_hack(int n) {
	printc("rust hit %d \n",n);
}

/* This is a bit of a hack, but we setup pthread data for sl threads */
#define _NSIG 65

struct pthread {
	struct pthread *self;
	void **dtv, *unused1, *unused2;
	uintptr_t sysinfo;
	uintptr_t canary, canary2;
	pid_t tid, pid;
	int tsd_used, errno_val;
	volatile int cancel, canceldisable, cancelasync;
	int detached;
	unsigned char *map_base;
	size_t map_size;
	void *stack;
	size_t stack_size;
	void *start_arg;
	void *(*start)(void *);
	void *result;
	struct __ptcb *cancelbuf;
	void **tsd;
	pthread_attr_t attr;
	volatile int dead;
	struct {
		volatile void *volatile head;
		long off;
		volatile void *volatile pending;
	} robust_list;
	int unblock_cancel;
	volatile int timer_id;
	locale_t locale;
	volatile int killlock[2];
	volatile int exitlock[2];
	volatile int startlock[2];
	unsigned long sigmask[_NSIG/8/sizeof(long)];
	char *dlerror_buf;
	int dlerror_flag;
	void *stdio_locks;
	uintptr_t canary_at_end;
	void **dtv_copy;
};

struct pthread backing_thread_data[SL_MAX_NUM_THDS];

void
assign_thread_data(struct sl_thd *thread)
{
	struct cos_compinfo *ci     = cos_compinfo_get(cos_defcompinfo_curr_get());
	thdcap_t             thdcap = sl_thd_thdcap(thread);
	thdid_t              thdid  = sl_thd_thdid(thread);

	/* HACK: We setup some thread specific data to make musl stuff work with sl threads */
	backing_thread_data[thdid].tid = thdid;
	backing_thread_data[thdid].robust_list.head = &backing_thread_data[thdid].robust_list.head;
	backing_thread_data[thdid].tsd = calloc(PTHREAD_KEYS_MAX, sizeof(void*));

	void *addr = memmgr_tls_alloc(thdid);
	cos_thd_mod(ci, thdcap, addr);

	*(void **)addr = &backing_thread_data[thdid];
}

#define MAX_REPS 3

/*rust init also begins sched loop */
extern void  rust_init();
extern void *replica_request();

int voter_initialized = 0;

thdid_t replica_thdids[MAX_REPS];
spdid_t replica_ids[MAX_REPS];
/* c->rust function parameter corruption workaround */
int request_data[MAX_REPS][3];

thdid_t *
get_replica_thdids() {
	return replica_thdids;
}

spdid_t *
get_replica_ids() {
	return replica_ids;
}

void
voter_done_initalizing() {
	voter_initialized = 1;
}

int *
get_request_data(spdid_t id) {
	return request_data[id % MAX_REPS];
}

//request
//unsure exactly what type this should return - do sinv returns work just like functions ? i think so
void *
request(int shdmem_id, int opcode, int data_size)
{
	while (!voter_initialized) sl_thd_yield(0);

	struct sl_thd *t = sl_thd_curr();
	assert(t);
	/* Set up Backing pthread structure in TLS for rust */
	assign_thread_data(t);

	vaddr_t shdmem_addr;
	int ret = memmgr_shared_page_map(shdmem_id, &shdmem_addr);
	assert(ret > -1 && shdmem_addr);
	/* store request data for each replica in a global store accessible from Rust */
	spdid_t spdid = cos_inv_token();
	request_data[spdid % MAX_REPS][0] = data_size;
	request_data[spdid % MAX_REPS][1] = opcode;
	request_data[spdid % MAX_REPS][2] = shdmem_addr;
	/* This will trigger a call back to get the reqeust_data */
	return replica_request();
}


static int
schedinit_self(void)
{
	/* if my init is done and i've all child inits */
	if (self_init && num_child_init == sched_num_childsched_get()) {
		if (parent_schedinit_child() < 0) assert(0);

		return 0;
	}

	return 1;
}


void
sched_child_init(struct sched_childinfo *schedci)
{
	struct sl_thd *initthd = NULL;
	printc("Voter Initializing Replica\n");

	assert(schedci);
	initthd = sched_child_initthd_get(schedci);
	assert(initthd);
	replica_thdids[schedci->id % MAX_REPS] = sl_thd_thdid(initthd);
	replica_ids[schedci->id % MAX_REPS] = schedci->id;

	sl_thd_param_set(initthd, sched_param_pack(SCHEDP_PRIO, FIXED_PRIO));
	sl_thd_param_set(initthd, sched_param_pack(SCHEDP_WINDOW, FIXED_PERIOD_MS));
	sl_thd_param_set(initthd, sched_param_pack(SCHEDP_BUDGET, FIXED_BUDGET_MS));
}

void
cos_init()
{
	struct sl_thd *t;

	t = sl_thd_curr();
	assign_thread_data(t);

	sched_childinfo_init();

	self_init = 1;

	while (schedinit_self()) sl_thd_block_periodic(0);
	PRINTLOG(PRINT_DEBUG, "Replica boot complete\n");

	printc("Entering rust!\n");
	//voter init also begins sched loop
	rust_init();
}

