#ifndef APPLICATION_INTERFACE_H
#define APPLICATION_INTERFACE_H

#include <sys/socket.h>

#define MAX_ARGS 3

/* opcode to pass to voter */
#define WRITE  0
#define READ   1
#define SOCKET 2
#define BIND   3
#define ACCEPT 4
#define LISTEN 5

int request(int opcode, int shdmem_size, int *args);

static inline void
pack_args(int * args, int arg1, int arg2, int arg3)
{
	args[0] = arg1;
	args[1] = arg2;
	args[2] = arg3;
}

int
_voter_write(int fd, int count)
{
	int args[MAX_ARGS];
	pack_args(args,fd,0,0);

	return request(WRITE, count, args);
}

int
_voter_read(int fd, size_t nbyte)
{
	int args[MAX_ARGS];
	pack_args(args,fd,0,0);

	return request(READ,nbyte,args);
}

int
_voter_socket(int domain, int type, int protocol)
{
	int args[MAX_ARGS];
	pack_args(args,domain,type,protocol);

	return request(SOCKET, 0, args);
}

int
_voter_bind(int sockfd, socklen_t addrlen)
{
	int args[MAX_ARGS];
	pack_args(args,sockfd,0,0);

	return request(BIND, addrlen, args);
}

int
_voter_accept(int sockfd,size_t size)
{
	int args[MAX_ARGS];
	pack_args(args,sockfd,0,0);

	return request(ACCEPT, size, args);
}

int
_voter_listen(int sockfd, int backlog)
{
	int args[MAX_ARGS];
	pack_args(args,sockfd,backlog,0);

	return request(LISTEN, 0, args);
}

#endif /* APPLICATION_INTERFACE_H */
