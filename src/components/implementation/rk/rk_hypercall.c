#include <cos_kernel_api.h>
#include <cos_types.h>
#include <cringbuf.h>
#include <sinv_calls.h>
#include <sys/socket.h>
#include <rumpcalls.h>
#include <vk_types.h>
#include <llprint.h>
#include <rk.h>
#include <memmgr.h>

int rump___sysimpl_socket30(int, int, int);
int rump___sysimpl_bind(int, const struct sockaddr *, socklen_t);
ssize_t rump___sysimpl_recvfrom(int, void *, size_t, int, struct sockaddr *, socklen_t *);
ssize_t rump___sysimpl_sendto(int, const void *, size_t, int, const struct sockaddr *, socklen_t);

/* These synchronous invocations involve calls to and from a RumpKernel */
//extern struct cringbuf *vmrb;
/* TODO when rumpbooter is its own interface, have this as an exported symbol */
struct cringbuf *vmrb = NULL;

int
test_entry(int arg1, int arg2, int arg3, int arg4)
{
        int ret = 0;

        printc("\n*** KERNEL COMPONENT ***\n \tArguments: %d, %d, %d, %d\n", arg1, arg2, arg3, arg4);
        printc("spdid: %d\n", arg4);
        printc("*** KERNEL COMPONENT RETURNING ***\n\n");

        return ret;
}

int
test_fs(int arg1, int arg2, int arg3, int arg4)
{
        int ret = 0;

        printc("\n*** KERNEL COMPONENT ***\n \tArguments: %d, %d, %d, %d\n", arg1, arg2, arg3, arg4);
        printc("spdid: %d\n", arg4);

        /* FS Test */
        printc("Running paws test: VM%d\n", cos_spdid_get());
//      paws_tests();

        printc("*** KERNEL COMPONENT RETURNING ***\n\n");

        return ret;

}

int
get_boot_done(void) {
	return 1;
}

int
rk_socket(int domain, int type, int protocol)
{
	printc("RK socket\n");
	return rump___sysimpl_socket30(domain, type, protocol);
}

int
rk_bind(int sockfd, int shdmem_id, socklen_t socklen)
{
	printc("RK bind\n");
	const struct sockaddr *sock = NULL;
	int ret;
	vaddr_t addr;
	ret = memmgr_shared_page_map(shdmem_id, &addr);
	assert(ret > -1 && addr);
	sock = (const struct sockaddr *)addr;
	return rump___sysimpl_bind(sockfd, sock, socklen);
}

ssize_t
rk_recvfrom(int arg1, int arg2, int arg3)
{
	/*
	 * TODO, simplify this, this is so ugly because it combines two functions that now
	 * don't need to be separated
	 */
	static int shdmem_id = -1;
	static vaddr_t my_addr;
	vaddr_t my_addr_tmp;
	void *buff;
	struct sockaddr *from;
	socklen_t *from_addr_len_ptr;
	int s, buff_shdmem_id, flags, from_shdmem_id, from_addr_len, ret;
	size_t len;

	printc("RK recvfrom\n");

	s = (arg1 >> 16);
	buff_shdmem_id = (arg1 << 16) >> 16;
	len = (arg2 >> 16);
	flags = (arg2 << 16) >> 16;
	from_shdmem_id = (arg3 >> 16);
	from_addr_len = (arg3 << 16) >> 16;

	if (shdmem_id == -1 && my_addr == 0) {
		shdmem_id = buff_shdmem_id;
		ret = memmgr_shared_page_map(buff_shdmem_id, &my_addr);
		assert(ret);
	}

	assert(shdmem_id > -1);
	assert(my_addr > 0);
	/* We are using only one page, make sure the id is the same */
	assert(buff_shdmem_id == from_shdmem_id && buff_shdmem_id == shdmem_id);

	/* TODO, put this in a function */
	/* In the shared memory page, first comes the message buffer for len amount */
	my_addr_tmp = my_addr;
	buff = (void *)my_addr_tmp;
	my_addr_tmp += len;

	/* Second is from addr length ptr */
	from_addr_len_ptr  = (void *)my_addr_tmp;
	*from_addr_len_ptr = from_addr_len;
	my_addr_tmp += sizeof(socklen_t *);

	/* Last is the from socket address */
	from = (struct sockaddr *)my_addr_tmp;

	return rump___sysimpl_recvfrom(s, buff, len, flags, from, from_addr_len_ptr);
}

ssize_t
rk_sendto(int arg1, int arg2, int arg3)
{
	static int shdmem_id = -1;
	static const void *buff;
	const struct sockaddr *sock;
	vaddr_t addr;
	int sockfd, flags, buff_shdmem_id, addr_shdmem_id, ret;
	size_t len;
	socklen_t addrlen;

	sockfd            = (arg1 >> 16);
	buff_shdmem_id    = (arg1 << 16) >> 16;
	len               = (arg2 >> 16);
	flags             = (arg2 << 16) >> 16;
	addr_shdmem_id    = (arg3 >> 16);
	addrlen           = (arg3 << 16) >> 16;

	printc("RK sendto\n");

	if (shdmem_id == -1 && buff == 0) {
		shdmem_id = buff_shdmem_id;
		ret = memmgr_shared_page_map(buff_shdmem_id, &addr);
		assert(ret);
		buff = (const void *)addr;
	}

	assert(shdmem_id > -1);
	assert(buff);
	assert(buff_shdmem_id == addr_shdmem_id && buff_shdmem_id == shdmem_id);

	sock = (const struct sockaddr *)(buff + len);
	assert(sock);

	return rump___sysimpl_sendto(sockfd, buff, len, flags, sock, addrlen);
}
