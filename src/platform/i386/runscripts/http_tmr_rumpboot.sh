#!/bin/sh

cp root_fprr.o boot.o
cp llboot_comp.o llboot.o
cp replica_http.o replica1.o
cp replica_http.o replica2.o
cp replica_http.o replica3.o
cp test_boot.o hack_stub.o
./cos_linker "llboot.o, ;capmgr.o, ;*rumpcos.o, ;*http_tmr.o, ;*boot.o, ;replica1.o, ;replica2.o, ;replica3.o, ;hack_stub.o, :boot.o-capmgr.o;rumpcos.o-capmgr.o|[parent_]boot.o;http_tmr.o-capmgr.o|[parent_]rumpcos.o;replica1.o-http_tmr.o|capmgr.o;replica2.o-http_tmr.o|capmgr.o;replica3.o-http_tmr.o|capmgr.o" ./gen_client_stub
