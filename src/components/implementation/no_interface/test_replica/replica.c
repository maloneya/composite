#include <cos_kernel_api.h>
#include <voter.h>
#include <cos_types.h>
#include <cobj_format.h>
#include "../../interface/capmgr/memmgr.h"

void cos_init(void)
{
	printc("Welcome to the test replica component\n");

	printc("Invoking voter interface:\n");
	vaddr_t shdmem_addr;
	int shdmem_id;

	shdmem_id = memmgr_shared_page_alloc(&shdmem_addr);
    assert(shdmem_id > -1 && shdmem_addr > 0);

    for (int i = 0; i < 5; i++) {
    	*(int *)(shdmem_addr+i) = i + 1;
    }

	voter_write(shdmem_id,5);

	return;
}
