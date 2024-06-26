use crate::mm::{
    frame_alloc_more, frame_dealloc, kernel_token, FrameTracker, PageTable, PhysAddr, PhysPage,
    StepByOne, VirtAddr,
};
use crate::sync::UPIntrFreeCell;
use alloc::vec::Vec;
use lazy_static::*;
use virtio_drivers::Hal;

lazy_static! {
    static ref QUEUE_FRAMES: UPIntrFreeCell<Vec<FrameTracker>> =
        unsafe { UPIntrFreeCell::new(Vec::new()) };
}

pub struct VirtioHal;

impl Hal for VirtioHal {
    fn dma_alloc(pages: usize) -> usize {
        let trakcers = frame_alloc_more(pages);
        let ppn_base = trakcers.as_ref().unwrap().last().unwrap().ppn;
        QUEUE_FRAMES
            .exclusive_access()
            .append(&mut trakcers.unwrap());
        let pa: PhysAddr = ppn_base.into();
        usize::from(pa) & arch::VIRT_ADDR_START_MASK
    }

    fn dma_dealloc(pa: usize, pages: usize) -> i32 {
        let pa = PhysAddr::from(pa);
        let mut ppn_base: PhysPage = pa.into();
        for _ in 0..pages {
            frame_dealloc(ppn_base);
            ppn_base.step();
        }
        0
    }

    fn phys_to_virt(addr: usize) -> usize {
        addr | arch::VIRT_ADDR_START
    }

    fn virt_to_phys(vaddr: usize) -> usize {
        // usize::from(PageTable::from_satp(kernel_token())
        //     .translate_va(VirtAddr::from(vaddr))
        //     .unwrap())
		vaddr & arch::VIRT_ADDR_START_MASK
    }
}
