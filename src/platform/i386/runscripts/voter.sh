#!/bin/sh

cp llboot_comp.o llboot.o
cp root_fprr.o boot.o
cp test_boot.o dummy1.o
./cos_linker "llboot.o, ;*sl_voter_cpt.o, ;capmgr.o, ;dummy1.o, ;*boot.o, :boot.o-capmgr.o;sl_voter_cpt.o-boot.o|capmgr.o" ./gen_client_stub
