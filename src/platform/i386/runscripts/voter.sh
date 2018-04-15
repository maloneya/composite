#!/bin/sh

cp llboot_comp.o llboot.o
cp root_fprr.o boot.o
cp replica_http.o replica1.o
cp replica_http.o replica2.o
cp replica_http.o replica3.o

./cos_linker "llboot.o, ;*sl_voter_cpt.o, ;capmgr.o, ;replica1.o, ;*boot.o, ;replica2.o, ;replica3.o, :boot.o-capmgr.o;sl_voter_cpt.o-capmgr.o|[parent_]boot.o;replica1.o-sl_voter_cpt.o|capmgr.o;sl_voter_cpt.o-capmgr.o|[parent_]boot.o;replica2.o-sl_voter_cpt.o|capmgr.o;sl_voter_cpt.o-capmgr.o|[parent_]boot.o;replica3.o-sl_voter_cpt.o|capmgr.o" ./gen_client_stub
