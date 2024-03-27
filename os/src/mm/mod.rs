mod frame_allocator;
mod heap_allocator;
mod memory_set;
mod page_table;

pub use arch::{VPNRange,StepByOne,PhysAddr,PhysPage,VirtAddr,VirtPage};
pub use frame_allocator::{frame_alloc, frame_alloc_more, frame_dealloc, FrameTracker};
pub use memory_set::{kernel_token, MapArea, MapPermission, MapType, MemorySet, KERNEL_SPACE};

pub use arch::{PageTable,PTE,MappingFlags};
pub use page_table::{
    translated_byte_buffer, translated_ref, translated_refmut, translated_str, UserBuffer,
};

pub fn init() {
	allocator::init();
    frame_allocator::init_frame_allocator();
    // KERNEL_SPACE.exclusive_access().activate();
}
