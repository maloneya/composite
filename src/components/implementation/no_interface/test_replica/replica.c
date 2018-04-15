#include <cos_kernel_api.h>
#include <voter.h>
#include <cos_types.h>
#include <cobj_format.h>

void make_sys_call() {
	int fd = 3;
	int size = 5;
	int data[size];
	for (int i = 0; i < size; i++) 
		data[i] = i;

    printc("Replica making syscall\n");
	int ret = voter_write(fd,data,size);
	printc("replica got: %d\n", ret);
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


