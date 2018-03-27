#!/bin/sh

cp llboot_test.o llboot.o
./cos_linker "llboot.o, ;*sl_voter_cpt.o, ;capmgr.o, :sl_voter_cpt.o-capmgr.o" ./gen_client_stub
