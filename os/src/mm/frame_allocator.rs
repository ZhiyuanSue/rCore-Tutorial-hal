use super::{PhysAddr, PhysPage};
use crate::config::MEMORY_END;
use crate::sync::UPIntrFreeCell;
use alloc::vec::Vec;
use log::{info,error};
use core::fmt::{self, Debug, Formatter};
use lazy_static::*;

pub struct FrameTracker {
    pub ppn: PhysPage,
}

impl FrameTracker {
    pub fn new(ppn: PhysPage) -> Self {
        // page cleaning
        let bytes_array = ppn.get_bytes_array();
        for i in bytes_array {
            *i = 0;
        }
        Self { ppn }
    }
}

impl Debug for FrameTracker {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("FrameTracker:PPN={:#x}", usize::from(self.ppn)))
    }
}

impl Drop for FrameTracker {
    fn drop(&mut self) {
        frame_dealloc(self.ppn);
    }
}

trait FrameAllocator {
    fn new() -> Self;
    fn alloc(&mut self) -> Option<PhysPage>;
    fn alloc_more(&mut self, pages: usize) -> Option<Vec<PhysPage>>;
    fn dealloc(&mut self, ppn: PhysPage);
}

pub struct StackFrameAllocator {
    current: usize,
    end: usize,
    recycled: Vec<usize>,
}

impl StackFrameAllocator {
    pub fn init(&mut self, l: PhysPage, r: PhysPage) {
        self.current = usize::from(l);
        self.end = usize::from(r);
        // println!("last {} Physical Frames.", self.end - self.current);
    }
}
impl FrameAllocator for StackFrameAllocator {
    fn new() -> Self {
        Self {
            current: 0,
            end: 0,
            recycled: Vec::new(),
        }
    }
    fn alloc(&mut self) -> Option<PhysPage> {
        if let Some(ppn) = self.recycled.pop() {
			error!("alloc 1");
            Some((ppn & (arch::VIRT_ADDR_START_MASK >> 12)).into())
        } else if self.current == self.end {
			error!("alloc 2");
            None
        } else {
			error!("alloc 3");
            self.current += 1;
            Some(((self.current - 1) & (arch::VIRT_ADDR_START_MASK >> 12)).into())
        }
    }
    fn alloc_more(&mut self, pages: usize) -> Option<Vec<PhysPage>> {
        if self.current + pages >= self.end {
            None
        } else {
            self.current += pages;
            let arr: Vec<usize> = (1..pages + 1).collect();
            let v = arr.iter().map(|x| (self.current - x).into()).collect();
            Some(v)
        }
    }
    fn dealloc(&mut self, ppn: PhysPage) {
		error!("dealloc ppn {}",ppn);
        let ppn = usize::from(ppn) & ((arch::VIRT_ADDR_START_MASK) >> 12);
        // validity check
        if ppn >= self.current || self.recycled.iter().any(|&v| v == ppn) {
            panic!("Frame ppn={:#x} has not been allocated!", ppn);
        }
        // recycle
        self.recycled.push(ppn);
    }
}

type FrameAllocatorImpl = StackFrameAllocator;

lazy_static! {
    pub static ref FRAME_ALLOCATOR: UPIntrFreeCell<FrameAllocatorImpl> =
        unsafe { UPIntrFreeCell::new(FrameAllocatorImpl::new()) };
}

pub fn init_frame_allocator() {
    extern "C" {
        fn ekernel();
    }
    FRAME_ALLOCATOR.exclusive_access().init(
        PhysAddr::from((ekernel as usize) & arch::VIRT_ADDR_START_MASK).ceil(),
        PhysAddr::from(MEMORY_END).floor(),
    );
}

pub fn frame_alloc() -> Option<PhysPage> {
    FRAME_ALLOCATOR
        .exclusive_access()
        .alloc()
        // .map(FrameTracker::new)
		/*here we need a page that will not auto free to satisify the hal level requirement*/
}

pub fn frame_alloc_more(num: usize) -> Option<Vec<FrameTracker>> {
    FRAME_ALLOCATOR
        .exclusive_access()
        .alloc_more(num)
        .map(|x| x.iter().map(|&t| FrameTracker::new(t)).collect())
}

pub fn frame_dealloc(ppn: PhysPage) {
    FRAME_ALLOCATOR.exclusive_access().dealloc(ppn);
}

// #[allow(unused)]
// pub fn frame_allocator_test() {
//     let mut v: Vec<FrameTracker> = Vec::new();
//     for i in 0..5 {
//         let frame = frame_alloc().unwrap();
//         println!("{:?}", frame);
//         v.push(frame);
//     }
//     v.clear();
//     for i in 0..5 {
//         let frame = frame_alloc().unwrap();
//         println!("{:?}", frame);
//         v.push(frame);
//     }
//     drop(v);
//     println!("frame_allocator_test passed!");
// }

// #[allow(unused)]
// pub fn frame_allocator_alloc_more_test() {
//     let mut v: Vec<FrameTracker> = Vec::new();
//     let frames = frame_alloc_more(5).unwrap();
//     for frame in &frames {
//         println!("{:?}", frame);
//     }
//     v.extend(frames);
//     v.clear();
//     let frames = frame_alloc_more(5).unwrap();
//     for frame in &frames {
//         println!("{:?}", frame);
//     }
//     drop(v);
//     println!("frame_allocator_test passed!");
// }
