#!/bin/sh

cp llboot_comp.o llboot.o
cp root_fprr.o boot.o
cp test_replica.o replica1.o
./cos_linker "llboot.o, ;*sl_voter_cpt.o, ;capmgr.o, ;replica1.o, ;*boot.o, :boot.o-capmgr.o;sl_voter_cpt.o-capmgr.o|[parent_]boot.o;replica1.o-sl_voter_cpt.o" ./gen_client_stub
