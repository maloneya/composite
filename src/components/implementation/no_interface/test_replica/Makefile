C_OBJS=replica.o
ASM_OBJS=
COMPONENT=test_replica.o
INTERFACES=
DEPENDENCIES=voter capmgr
IF_LIB=
ADDITIONAL_LIBS=-lcos_defkernel_api -lsl_sched -lsl_capmgr -lcos_kernel_api -lsl_mod_fprr -lsl_thd_static_backend -lheap
OBJLIBS+=$(POSIX_LIB)

include ../../Makefile.subsubdir
MANDITORY_LIB=simple_stklib.o
