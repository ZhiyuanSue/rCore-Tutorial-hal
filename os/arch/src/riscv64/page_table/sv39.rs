use core::arch::{asm, riscv64::sfence_vma};

use alloc::sync::Arc;
use bitflags::bitflags;

use crate::{
    sigtrx::get_trx_mapping, ArchInterface, MappingFlags, PhysAddr, PhysPage, VirtAddr, VirtPage,
    PAGE_ITEM_COUNT, PAGE_SIZE, VIRT_ADDR_START,
};

#[derive(Copy, Clone, Debug)]
pub struct PTE(usize);

impl PTE {
    #[inline]
    pub const fn new() -> Self {
        Self(0)
    }

    #[inline]
    pub const fn from_ppn(ppn: usize, flags: PTEFlags) -> Self {
        // let flags = flags.union(PTEFlags::D);
        let mut flags = flags;
        if flags.contains(PTEFlags::R) | flags.contains(PTEFlags::X) {
            flags = flags.union(PTEFlags::A)
        }
        if flags.contains(PTEFlags::W) {
            flags = flags.union(PTEFlags::D)
        }
        // TIPS: This is prepare for the extend bits of T-HEAD C906
        #[cfg(c906)]
        if flags.contains(PTEFlags::G) && ppn == 0x8_0000 {
            Self(
                ppn << 10
                    | flags
                        .union(PTEFlags::C)
                        .union(PTEFlags::B)
                        .union(PTEFlags::K)
                        .bits() as usize,
            )
        } else if flags.contains(PTEFlags::G) && ppn == 0 {
            Self(ppn << 10 | flags.union(PTEFlags::SE).union(PTEFlags::SO).bits() as usize)
        } else {
            Self(ppn << 10 | flags.union(PTEFlags::C).bits() as usize)
        }

        #[cfg(not(c906))]
        Self(ppn << 10 | flags.bits() as usize)
    }

    #[inline]
    pub const fn from_addr(addr: usize, flags: PTEFlags) -> Self {
        Self::from_ppn(addr >> 12, flags)
    }

    #[inline]
    pub const fn to_ppn(&self) -> PhysPage {
        PhysPage((self.0 >> 10) & ((1 << 29) - 1))
    }

    #[inline]
    pub fn set(&mut self, ppn: usize, flags: PTEFlags) {
        self.0 = (ppn << 10) | flags.bits() as usize;
    }

    #[inline]
    pub const fn flags(&self) -> PTEFlags {
        PTEFlags::from_bits_truncate((self.0 & 0xff) as u64)
    }

    #[inline]
    pub const fn is_valid(&self) -> bool {
        self.flags().contains(PTEFlags::V) && self.0 > u8::MAX as usize
    }

    /// 判断是否是大页
    ///
    /// 大页判断条件 V 位为 1, R/W/X 位至少有一个不为 0
    /// PTE 页表范围 1G(0x4000_0000) 2M(0x20_0000) 4K(0x1000)
    #[inline]
    pub fn is_huge(&self) -> bool {
        return self.flags().contains(PTEFlags::V)
            && (self.flags().contains(PTEFlags::R)
                || self.flags().contains(PTEFlags::W)
                || self.flags().contains(PTEFlags::X));
    }

    #[inline]
    pub fn is_leaf(&self) -> bool {
        return self.flags().contains(PTEFlags::V)
            && !(self.flags().contains(PTEFlags::R)
                || self.flags().contains(PTEFlags::W)
                || self.flags().contains(PTEFlags::X));
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct PTEFlags: u64 {
        const NONE = 0;
        const V = 1 << 0;
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
        const G = 1 << 5;
        const A = 1 << 6;
        const D = 1 << 7;

        #[cfg(c906)]
        const SO = 1 << 63;
        #[cfg(c906)]
        const C = 1 << 62;
        #[cfg(c906)]
        const B = 1 << 61;
        #[cfg(c906)]
        const K = 1 << 60;
        #[cfg(c906)]
        const SE = 1 << 59;

        const VRWX  = Self::V.bits() | Self::R.bits() | Self::W.bits() | Self::X.bits();
        const ADUVRX = Self::A.bits() | Self::D.bits() | Self::U.bits() | Self::V.bits() | Self::R.bits() | Self::X.bits();
        const ADVRWX = Self::A.bits() | Self::D.bits() | Self::VRWX.bits();
        const ADGVRWX = Self::G.bits() | Self::ADVRWX.bits();
    }
}

impl From<MappingFlags> for PTEFlags {
    fn from(flags: MappingFlags) -> Self {
        if flags == MappingFlags::None {
            Self::NONE
        } else {
            let mut res = Self::V;
            if flags.contains(MappingFlags::R) {
                res |= PTEFlags::R;
            }
            if flags.contains(MappingFlags::W) {
                res |= PTEFlags::W;
            }
            if flags.contains(MappingFlags::X) {
                res |= PTEFlags::X;
            }
            if flags.contains(MappingFlags::U) {
                res |= PTEFlags::U;
            }
            if flags.contains(MappingFlags::A) {
                res |= PTEFlags::A;
            }
            if flags.contains(MappingFlags::D) {
                res |= PTEFlags::D;
            }
            res
        }
    }
}

#[inline]
pub fn get_pte_list(paddr: PhysAddr) -> &'static mut [PTE] {
	info!("get pte_list {}",paddr);
    unsafe { core::slice::from_raw_parts_mut(paddr.get_mut_ptr::<PTE>(), PAGE_ITEM_COUNT) }
}

#[derive(Debug)]
pub struct PageTable(pub(crate) PhysAddr);

impl PageTable {
    pub fn alloc() -> Self {
        let addr = ArchInterface::frame_alloc_persist().into();
        let page_table = Self(addr);
        page_table.restore();
        page_table
    }

    #[inline]
    pub fn restore(&self) {
		info!("go into restore");
        let arr = get_pte_list(self.0);
        arr[0x100] = PTE::from_addr(0x0000_0000, PTEFlags::ADGVRWX);
        arr[0x101] = PTE::from_addr(0x4000_0000, PTEFlags::ADGVRWX);
        arr[0x102] = PTE::from_addr(0x8000_0000, PTEFlags::ADGVRWX);
        arr[0x104] = PTE::from_addr(get_trx_mapping(), PTEFlags::V);
        arr[0x106] = PTE::from_addr(0x8000_0000, PTEFlags::ADGVRWX);
        arr[0..0x100].fill(PTE::from_addr(0, PTEFlags::NONE));
    }

    #[inline]
    pub const fn get_satp(&self) -> usize {
        (8 << 60) | (self.0 .0 >> 12)
    }
	#[inline]
	pub fn from_satp(satp: usize) -> Self {
        let root_ppn=PhysPage::from(satp & ((1usize << 44) - 1));
		Self(PhysAddr::from(root_ppn))
    }

    #[inline]
    pub fn change(&self) {
        unsafe {
            asm!("csrw satp, {0}",  in(reg) self.get_satp());
            riscv::asm::sfence_vma_all();
        }
    }

    #[inline]
    pub fn map(&self, ppn: PhysPage, vpn: VirtPage, flags: MappingFlags, level: usize) {
        // TODO: Add huge page support.
		info!("go into map");
        let mut pte_list = get_pte_list(self.0);
        for i in (1..level).rev() {
            let value = (vpn.0 >> 9 * i) & 0x1ff;
            let pte = &mut pte_list[value];
            if i == 0 {
                break;
            }
            if !pte.is_valid() {
                *pte = PTE::from_ppn(ArchInterface::frame_alloc_persist().0, PTEFlags::V);
				info!("*pte to ppn is {}",(*pte).to_ppn());
            }
			info!("go into map for loop");
            // page_table = PageTable(pte.to_ppn().into());
            pte_list = get_pte_list(pte.to_ppn().into());
        }

        pte_list[vpn.0 & 0x1ff] = PTE::from_ppn(ppn.0, flags.into());
        unsafe {
            sfence_vma(vpn.to_addr(), 0);
        }
    }

    #[inline]
    pub fn unmap(&self, vpn: VirtPage) {
        // TODO: Add huge page support.
        let mut pte_list = get_pte_list(self.0);
        for i in (1..3).rev() {
            let value = (vpn.0 >> 9 * i) & 0x1ff;
            let pte = &mut pte_list[value];
            if !pte.is_valid() {
                return;
            }
            pte_list = get_pte_list(pte.to_ppn().into());
        }

        pte_list[vpn.0 & 0x1ff] = PTE::new();
        unsafe {
            sfence_vma(vpn.to_addr(), 0);
        }
    }

    #[inline]
    pub fn virt_to_phys(&self, vaddr: VirtAddr) -> Option<PhysAddr> {
        let mut paddr = self.0;
        for i in (0..3).rev() {
            let value = (vaddr.0 >> 12 + 9 * i) & 0x1ff;
            let pte = &get_pte_list(paddr)[value];
            // 如果当前页是大页 返回相关的位置
            // vaddr.0 % (1 << (12 + 9 * i)) 是大页内偏移
            if !pte.flags().contains(PTEFlags::V) {
                return None;
            }
            if pte.is_huge() {
                return Some(PhysAddr(
                    pte.to_ppn().0 << 12 | vaddr.0 % (1 << (12 + 9 * i)),
                ));
            }
            paddr = pte.to_ppn().into()
        }
        Some(PhysAddr(paddr.0 | vaddr.0 % PAGE_SIZE))
    }
    fn find_pte(&self, vpn: VirtPage) -> Option<&mut PTE> {
        let idxs = vpn.indexes();
        let mut ppn = self.0;
        let mut result: Option<&mut PTE> = None;
		info!("find_pte");
        for (i, idx) in idxs.iter().enumerate() {
			let pa: PhysAddr = ppn.into();
			let pte =&mut get_pte_list(pa)[*idx];
            // let pte = &mut ppn.get_pte_array()[*idx];
            if i == 2 {
                result = Some(pte);
                break;
            }
            if !pte.is_valid() {
                return None;
            }
            ppn = PhysAddr::from(pte.to_ppn());
        }
		info!("finish find pte");
        result
    }
	pub fn translate(&self, vpn: VirtPage) -> Option<PTE> {
		info!("go into translate");
        self.find_pte(vpn).map(|pte| *pte)
    }
    pub fn translate_va(&self, va: VirtAddr) -> Option<PhysAddr> {
        self.find_pte(va.clone().floor()).map(|pte| {
            let aligned_pa: PhysAddr = pte.to_ppn().into();
            let offset = va.page_offset();
            let aligned_pa_usize: usize = aligned_pa.into();
            (aligned_pa_usize + offset).into()
        })
    }
}

impl Drop for PageTable {
    fn drop(&mut self) {
        for root_pte in get_pte_list(self.0)[..0x100].iter().filter(|x| x.is_leaf()) {
            get_pte_list(root_pte.to_ppn().into())
                .iter()
                .filter(|x| x.is_leaf())
                .for_each(|x| ArchInterface::frame_unalloc(x.to_ppn()));
            ArchInterface::frame_unalloc(root_pte.to_ppn());
        }
        ArchInterface::frame_unalloc(self.0.into());
    }
}
