#ifndef APPLICATION_INTERFACE_H
#define APPLICATION_INTERFACE_H

/* opcode to pass to voter */
#define WRITE 0

void *request(int shdmem_id, int opcode, int data_size);

int
voter_write(int shdmem_id, int size) {
	void * result = request(shdmem_id, WRITE, size);
	//TODO build a real interface here
	return 0;
}


#endif /* APPLICATION_INTERFACE_H */
