#!/bin/sh

cp llboot_comp.o llboot.o
cp root_fprr.o boot.o
cp test_boot.o dummy1.o
cp test_boot.o dummy2.o
./cos_linker "llboot.o, ;dummy2.o, ;capmgr.o, ;dummy1.o, ;*sl_voter_cpt.o, :sl_voter_cpt.o-capmgr.o" ./gen_client_stub
