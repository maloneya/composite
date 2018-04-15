#include "vk_types.h"
#include "vk_api.h"
#include "micro_booter.h"

struct cos_compinfo booter_info;
/*
 * the capability for the thread switched to upon termination.
 * FIXME: not exit thread for now
 */
thdcap_t      termthd = BOOT_CAPTBL_SELF_INITTHD_BASE;
unsigned long tls_test[TEST_NTHDS];

#include <llprint.h>

/* For Div-by-zero test */
int num = 1, den = 0;

/* virtual machine id */
int vmid;

cycles_t
hpet_first_period(void)
{
	int ret;
	cycles_t start_period = 0;

	while ((ret = cos_introspect64(&booter_info, BOOT_CAPTBL_SELF_INITHW_BASE, HW_GET_FIRST_HPET, &start_period)) == -EAGAIN) ;
	if (ret) assert(0);

	return start_period;
}

void
vm_init(void *d)
{
	int         rcvd, blocked;
	cycles_t    cycles;
	thdid_t     tid;
	tcap_time_t timeout = 0, thd_timeout;

	vmid = cos_sinv(VM_CAPTBL_SELF_SINV_BASE, VK_SERV_VM_ID << 16 | cos_thdid(), 0, 0, 0);

	cos_meminfo_init(&booter_info.mi, BOOT_MEM_KM_BASE, VM_UNTYPED_SIZE, BOOT_CAPTBL_SELF_UNTYPED_PT);
	cos_compinfo_init(&booter_info, BOOT_CAPTBL_SELF_PT, BOOT_CAPTBL_SELF_CT, BOOT_CAPTBL_SELF_COMP,
	                  (vaddr_t)cos_get_heap_ptr(), vmid == 0 ? DOM0_CAPTBL_FREE : VM_CAPTBL_FREE, &booter_info);

	PRINTC("Virtual-machine booter started.\n");
	test_run_vk();
	PRINTC("Virtual-machine booter done.\n");

	cos_sinv(VM_CAPTBL_SELF_SINV_BASE, VK_SERV_VM_EXIT << 16 | cos_thdid(), 0, 0, 0);
	/* should not be scheduled. but it may be activated if this tcap or it's child tcap has budget! */
}

void
dom0_io_fn(void *id)
{
	arcvcap_t rcvcap = dom0_vio_rcvcap((unsigned int)id);
	while (1) {
		cos_rcv(rcvcap, 0, NULL);
	}
}

void
vm_io_fn(void *id)
{
	arcvcap_t rcvcap = VM_CAPTBL_SELF_IORCV_BASE;
	while (1) {
		cos_rcv(rcvcap, 0, NULL);
	}
}
