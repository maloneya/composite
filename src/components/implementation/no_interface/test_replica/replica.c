#include <cos_kernel_api.h>
#include <voter.h>
#include <cos_types.h>
#include <cobj_format.h>
#include "../../interface/capmgr/memmgr.h"

void make_sys_call() {
	vaddr_t shdmem_addr;
	int shdmem_id;

	shdmem_id = memmgr_shared_page_alloc(&shdmem_addr);
    assert(shdmem_id > -1 && shdmem_addr > 0);

    for (int i = 0; i < 5; i++) {
    	*(int *)(shdmem_addr+i) = i + 1;
    }
    printc("Replica making syscall\n");
	voter_write(shdmem_id,5);
}

void do_work() {
	for (int i = 0;;i++) {
		if (i == 100) {
			make_sys_call();
			i = 0;
		}
	}
}

void cos_init(void)
{
	printc("Replica booted\n");
	replica_done_initializing();
	do_work();

	return;
}


