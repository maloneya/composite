#ifndef VOTER_H
#define VOTER_H

#include <posix.h>
#include <sys/socket.h>
#include <sys/mman.h>
#include "../../interface/capmgr/memmgr.h"

int shdmem_id;
vaddr_t shdmem_addr;


/* internal calls */
void _replica_done_initializing(int shdmem_id);
int  _voter_write(int fd, int count);
int  _voter_read(int fd, size_t nbyte);
int  _voter_socket(int domain, int type, int protocol);
int  _voter_bind(int sockfd, socklen_t addrlen);
int  _voter_accept(int sockfd, size_t size);

void
_shdmem_write_at(void * data, int size, int start_offset)
{
	assert(start_offset+size < 4096);
	assert(shdmem_addr > 0 && shdmem_id > -1);

	memcpy((void *)shdmem_addr+start_offset,data,size);
}

void _shdmem_write(void * data, int size) {_shdmem_write_at(data,size,0);}

/* client invoke stubs */
void replica_done_initializing()
{
	shdmem_id = memmgr_shared_page_alloc(&shdmem_addr);
	assert(shdmem_id > -1 && shdmem_addr > 0);

    	_replica_done_initializing(shdmem_id);
}

int
voter_write(int fd, void *buf, int count)
{
	assert(fd);

	_shdmem_write(buf,count);
	return _voter_write(fd,count);
}

long
voter_read(int fd, void *buf, size_t nbyte)
{
	long ret = _voter_read(fd,nbyte);
	assert(ret < PAGE_SIZE);

	memcpy((void *)buf, (void *)shdmem_addr, ret);
	return ret;
}

/* client local stubs */
int
voter_socketcall(int call, unsigned long *args)
{
	int ret = -1;

	switch (call) {
		case 1: { /* socket */
			int domain, type, protocol;

			domain   = *args;
			type     = *(args + 1);
			protocol = *(args + 2);

			ret = _voter_socket(domain,type,protocol);

			break;
		}
		case 2: { /* bind */
			int sockfd;
			void *addr;
			u32_t addrlen;

			sockfd  = *args;
			addr    = (void *)*(args + 1);
			addrlen = *(args + 2);

			_shdmem_write(addr,addrlen);
			ret = _voter_bind(sockfd,addrlen);

			break;
		}
		case 5: { /* accept */
			int sockfd;
			struct sockaddr *sock_addr;
			socklen_t * addrlen;

			sockfd    = *args;
			sock_addr = (struct sockaddr *)*(args + 1);
			addrlen   = (socklen_t *)*(args + 2);

			int size = sizeof(struct sockaddr);

			_shdmem_write_at(sock_addr,size,0);
			_shdmem_write_at(addrlen,sizeof(vaddr_t),size);

			ret = _voter_accept(sockfd,size);

			memcpy(sock_addr, (void *)shdmem_addr, size);
			memcpy(addrlen, (void *)shdmem_addr+size, size);

			break;
		}
		default: {
			printc("%s, ERROR, unimplemented socket call: %d\n", __func__, call);
			assert(0);
		}
	}

	return ret;
}

void *
voter_mmap(void *addr, size_t len, int prot, int flags, int fd, off_t off) /* copied from rk_inv_api.c */
{
	void *ret=0;
	printc("%s\n", __func__);

	if (addr != NULL) {
		printc("parameter void *addr is not supported!\n");
		errno = ENOTSUP;
		return MAP_FAILED;
	}
	if (fd != -1) {
		printc("WARNING, file mapping is not supported, ignoring file\n");
	}

	printc("getting %d number of pages\n", len / PAGE_SIZE);
	addr = (void *)memmgr_heap_page_allocn(len / PAGE_SIZE);
	printc("addr: %p\n", addr);
	if (!addr){
		ret = (void *) -1;
	} else {
		ret = addr;
	}

	if (ret == (void *)-1) {  /* return value comes from man page */
		printc("mmap() failed!\n");
		/* This is a best guess about what went wrong */
		errno = ENOMEM;
	}
	return ret;
}

int
voter_libcmod_init(void)
{
	posix_syscall_override((cos_syscall_t)voter_socketcall, __NR_socketcall);
	posix_syscall_override((cos_syscall_t)voter_mmap, __NR_mmap);
	posix_syscall_override((cos_syscall_t)voter_mmap, __NR_mmap2);
	posix_syscall_override((cos_syscall_t)voter_write, __NR_write);
	posix_syscall_override((cos_syscall_t)voter_read, __NR_read);

	return 0;
}

#endif /* VOTER_H */
