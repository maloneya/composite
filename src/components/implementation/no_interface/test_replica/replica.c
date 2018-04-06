#include <cos_kernel_api.h>
#include <voter.h>
#include <cos_types.h>
#include <cobj_format.h>

void cos_init(void)
{
	printc("Welcome to the test replica component\n");

	printc("Invoking voter interface:\n");
	test_call();

	return;
}
