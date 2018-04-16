#ifndef RK_H
#define RK_H

int     test_entry(int arg1, int arg2, int arg3, int arg4);
int     test_fs(int arg1, int arg2, int arg3, int arg4);
int     get_boot_done(void);
int     rk_socket(int domain, int type, int protocol);
int     rk_bind(int socketfd, int shdmem_id, unsigned addrlen);
/* TODO rename parameters to include information about what is being packed */
ssize_t rk_recvfrom(int arg1, int arg2, int arg3);
ssize_t rk_sendto(int arg1, int arg2, int arg3);
int     rk_setsockopt(int arg1, int arg2, int arg3);
void   *rk_mmap(int arg1, int arg2, int arg3);
long    rk_write(int arg1, int arg2, int arg3);
long    rk_read(int arg1, int arg2, int arg3);
int     rk_listen(int arg1, int arg2);
int     rk_clock_gettime(int arg1, int arg2);
int	rk_select(int arg1, int arg2);
int	rk_accept(int arg1, int arg2);
int	rk_open(int arg1, int arg2, int arg3);
int	rk_unlink(int arg1);
int	rk_ftruncate(int arg1, int arg2);
int	rk_getsockname(int arg1, int arg2);
int	rk_getpeername(int arg1, int arg2);
int	rk_create_thread_context(int thdid);
int	schedinit_child(void);

#endif /* RK_H */
