#!/bin/sh

cp rump_boot.o llboot.o
./cos_linker "llboot.o, :" ./gen_client_stub -v
