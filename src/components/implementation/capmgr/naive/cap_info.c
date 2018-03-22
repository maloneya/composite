#include <cos_kernel_api.h>
#include <cap_info.h>

static struct cap_comp_info capci[MAX_NUM_COMPS + 1]; /* includes booter information also, so +1 */
static unsigned int cap_comp_count;
u32_t cap_info_schedbmp[MAX_NUM_COMP_WORDS];
static struct cap_shmem_glb_info cap_shmglbinfo;

static inline struct cap_shmem_glb_info *
__cap_info_shmglb_info(void)
{
	return &cap_shmglbinfo;
}

struct cap_comp_info *
cap_info_comp_find(spdid_t spdid)
{
	return &capci[spdid];
}

unsigned int
cap_info_count(void)
{
	return cap_comp_count;
}

struct sl_thd *
cap_info_thd_find(struct cap_comp_info *rci, thdid_t tid)
{
	int i;

	if (!rci || !cap_info_init_check(rci)) return NULL;
	for (i = 0; i < rci->thd_used; i++) {
		if (sl_thd_thdid(rci->thdinfo[i]) == tid) return rci->thdinfo[i];
	}

	return NULL;
}

struct sl_thd *
cap_info_thd_next(struct cap_comp_info *rci)
{
	if (!rci || !cap_info_init_check(rci))   return NULL;
	if (rci->p_thd_iterator < rci->thd_used) return (rci->thdinfo[ps_faa((long unsigned *)&(rci->p_thd_iterator), 1)]);

	return NULL;
}

struct cap_comp_info *
cap_info_comp_init(spdid_t spdid, captblcap_t captbl_cap, pgtblcap_t pgtbl_cap, compcap_t compcap,
		   capid_t cap_frontier, vaddr_t heap_frontier, spdid_t sched_spdid)
{
	struct cos_compinfo       *ci      = cos_compinfo_get(&(capci[spdid].defci));
	struct cap_shmem_info     *cap_shi = cap_info_shmem_info(&capci[spdid]);
	struct cap_shmem_glb_info *rglb    = __cap_info_shmglb_info();

	capci[spdid].cid      = spdid;
	capci[spdid].thd_used = 1;
	capci[spdid].parent   = &capci[sched_spdid];

	cos_meminfo_init(&ci->mi, 0, 0, 0);
	cos_compinfo_init(ci, pgtbl_cap, captbl_cap, compcap, heap_frontier, cap_frontier,
			  cos_compinfo_get(cos_defcompinfo_curr_get()));

	memset(rglb, 0, sizeof(struct cap_shmem_glb_info));
	memset(cap_shi, 0, sizeof(struct cap_shmem_info));
	cap_shi->cinfo = ci;

	capci[spdid].initflag = 1;
	ps_faa((long unsigned *)&cap_comp_count, 1);

	return &capci[spdid];
}

struct cap_aepkey_info *
cap_aepkey_get(cos_aepkey_t key)
{
	if (key == 0) return NULL;

	return &cap_aepkeys[key - 1];
}

void
cap_aepkey_set(cos_aepkey_t key, struct sl_thd *t)
{
	struct cap_aepkey_info *ak = cap_aepkey_get(key);

	if (!ak || ak->slaep) return;
	ak->slaep  = t;
	ak->sndcap = sl_thd_asndcap(t);
}

asndcap_t
cap_aepkey_asnd_get(cos_aepkey_t key)
{
	struct cos_compinfo    *cap_ci = cos_compinfo_get(cos_defcompinfo_curr_get());
	struct cap_aepkey_info *ak     = cap_aepkey_get(key);

	if (!ak) return 0;
	if (ak->sndcap) return ak->sndcap;
	ak->sndcap = cos_asnd_alloc(cap_ci, sl_thd_rcvcap(ak->slaep), cap_ci->captbl_cap);
	assert(ak->sndcap);

	return ak->sndcap;
}

struct sl_thd *
cap_info_thd_init(struct cap_comp_info *rci, struct sl_thd *t, cos_aepkey_t key)
{
	int off;

	if (!rci || !cap_info_init_check(rci)) return NULL;
	if (rci->thd_used >= CAP_INFO_COMP_MAX_THREADS) return NULL;
	if (!t) return NULL;

	off = ps_faa((long unsigned *)&(rci->thd_used), 1);
	rci->thdinfo[off] = t;
	cap_aepkey_set(key, t);

	return t;
}

struct sl_thd *
cap_info_initthd_init(struct cap_comp_info *rci, struct sl_thd *t, cos_aepkey_t key)
{
	if (!rci || !cap_info_init_check(rci)) return NULL;
	if (rci->thd_used >= CAP_INFO_COMP_MAX_THREADS) return NULL;
	if (!t) return NULL;

	rci->thdinfo[0] = t;
	cap_aepkey_set(key, t);

	return t;
}

struct sl_thd *
cap_info_initthd(struct cap_comp_info *rci)
{
	if (!rci) return NULL;

	return rci->thdinfo[0];
}

void
cap_info_init(void)
{
	cap_comp_count = 0;
	memset(cap_info_schedbmp, 0, sizeof(u32_t) * MAX_NUM_COMP_WORDS);
	memset(capci, 0, sizeof(struct cap_comp_info)*(MAX_NUM_COMPS+1));
	memset(cap_aepkeys, 0, sizeof(struct cap_aepkey_info) * CAPMGR_AEPKEYS_MAX);
}

static inline vaddr_t
__cap_info_shm_capmgr_vaddr(int id)
{
	return capci[cos_spd_id()].shminfo.shm_addr[id];
}

static inline void
__cap_info_shm_capmgr_vaddr_set(int id, vaddr_t v)
{
	capci[cos_spd_id()].shminfo.shm_addr[id] = v;
}

static int
__cap_cos_shared_page_mapn(struct cos_compinfo *rci, int num_pages, vaddr_t capvaddr, vaddr_t *compvaddr)
{
	struct cos_compinfo *cap_ci = cos_compinfo_get(cos_defcompinfo_curr_get());
	int                  off    = 0;
	vaddr_t              dst_pg;

	assert(capvaddr);
	if (!capvaddr) return -1;

	*compvaddr = cos_mem_aliasn(rci, cap_ci, capvaddr, num_pages * PAGE_SIZE);
	if (!*compvaddr) return -1;

	return 0;
}

static int
__cap_cos_shared_page_allocn(struct cos_compinfo *rci, int num_pages, vaddr_t *capvaddr, vaddr_t *compvaddr)
{
	struct cos_compinfo *cap_ci = cos_compinfo_get(cos_defcompinfo_curr_get());
	int                  off    = 0;
	vaddr_t              src_pg, dst_pg;

	*capvaddr = src_pg = (vaddr_t)cos_page_bump_allocn(cap_ci, num_pages * PAGE_SIZE);
	if (!(*capvaddr)) return -1;

	if (__cap_cos_shared_page_mapn(rci, num_pages, src_pg, compvaddr)) return -1;

	return 0;
}

int
cap_shmem_region_alloc(struct cap_shmem_info *rsh, int num_pages)
{
	struct cos_compinfo       *rsh_ci    = cap_info_shmem_ci(rsh);
	struct cap_shmem_glb_info *rglb      = __cap_info_shmglb_info();
	int                        alloc_idx = -1, fidx, ret;
	vaddr_t                    cap_addr, comp_addr;

	if (!rsh) goto done;
	/* limits check */
	if ((rglb->total_pages + num_pages) * PAGE_SIZE > MEMMGR_MAX_SHMEM_SIZE) goto done;
	if ((rsh->total_pages + num_pages) * PAGE_SIZE > MEMMGR_COMP_MAX_SHMEM) goto done;
	fidx = ps_faa((long unsigned *)&(rglb->free_region_id), 1);
	if (fidx >= MEMMGR_MAX_SHMEM_REGIONS) goto done;

	/* check id unused */
	if (__cap_info_shm_capmgr_vaddr(fidx) != 0) goto done;
	if (rsh->shm_addr[fidx] != 0) goto done;

	rglb->region_npages[fidx] = num_pages;
	ps_faa((long unsigned *)&(rglb->total_pages), num_pages);
	ps_faa((long unsigned *)&(rsh->total_pages), num_pages);

	ret = __cap_cos_shared_page_allocn(rsh_ci, num_pages, &cap_addr, &comp_addr);
	if (ret) goto done;

	__cap_info_shm_capmgr_vaddr_set(fidx, cap_addr);
	rsh->shm_addr[fidx] = comp_addr;
	alloc_idx = fidx;

done:
	return alloc_idx;
}

int
cap_shmem_region_map(struct cap_shmem_info *rsh, int idx)
{
	struct cos_compinfo       *rsh_ci    = cap_info_shmem_ci(rsh);
	struct cap_shmem_glb_info *rglb      = __cap_info_shmglb_info();
	vaddr_t                    cap_addr  = __cap_info_shm_capmgr_vaddr(idx), comp_addr;
	unsigned long              num_pages = 0;
	int                        ret       = -1;

	if (!rsh) return 0;
	if (idx >= MEMMGR_MAX_SHMEM_REGIONS) return 0;
	if (!cap_addr || rsh->shm_addr[idx] != 0) return 0;
	num_pages = rglb->region_npages[idx];
	if ((rsh->total_pages + num_pages) * PAGE_SIZE > MEMMGR_COMP_MAX_SHMEM) return 0;

	ret = __cap_cos_shared_page_mapn(rsh_ci, num_pages, cap_addr, &comp_addr);
	if (ret) return 0;
	rsh->shm_addr[idx] = comp_addr;

	return rglb->region_npages[idx];
}

vaddr_t
cap_shmem_region_vaddr(struct cap_shmem_info *rsh, int id)
{
	return rsh->shm_addr[id];
}
