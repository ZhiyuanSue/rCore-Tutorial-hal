#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]

//use crate::drivers::{GPU_DEVICE, KEYBOARD_DEVICE, MOUSE_DEVICE, INPUT_CONDVAR};
use crate::drivers::{GPU_DEVICE, KEYBOARD_DEVICE, MOUSE_DEVICE};
use arch::{
	ArchInterface,Context,TrapType,PhysPage
};
use fdt::node::FdtNode;
extern crate alloc;

#[macro_use]
extern crate bitflags;

#[path = "boards/qemu.rs"]
mod board;

#[macro_use]
mod console;
mod config;
mod drivers;
mod fs;
mod lang_items;
mod mm;
mod net;
mod sbi;
mod sync;
mod syscall;
mod task;
mod timer;
mod trap;

use crate::drivers::chardev::CharDevice;
use crate::drivers::chardev::UART;

core::arch::global_asm!(include_str!("entry.asm"));

fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    unsafe {
        core::slice::from_raw_parts_mut(sbss as usize as *mut u8, ebss as usize - sbss as usize)
            .fill(0);
    }
}

use lazy_static::*;
use sync::UPIntrFreeCell;

lazy_static! {
    pub static ref DEV_NON_BLOCKING_ACCESS: UPIntrFreeCell<bool> =
        unsafe { UPIntrFreeCell::new(false) };
}

struct ArchInterfaceImpl;

#[crate_interface::impl_interface]
impl ArchInterface for ArchInterfaceImpl {
	fn init_logging() {
        let str = include_str!("logo.txt");
        println!("{}", str);
    }
	fn kernel_interrupt(ctx: &mut Context, trap_type: TrapType)
	{

	}
	fn add_memory_region(start: usize, end: usize)
	{

	}
	fn main(hartid: usize)
	{

	}
	fn frame_alloc_persist() -> PhysPage
	{
		PhysPage::new(0)
	}
	fn frame_unalloc(ppn: PhysPage)
	{

	}
	fn prepare_drivers()
	{

	}
	fn try_to_add_device(fdtNode: &FdtNode)
	{

	}
}

#[no_mangle]
pub fn rust_main() -> ! {
    clear_bss();
    mm::init();
    UART.init();
    println!("KERN: init gpu");
    let _gpu = GPU_DEVICE.clone();
    println!("KERN: init keyboard");
    let _keyboard = KEYBOARD_DEVICE.clone();
    println!("KERN: init mouse");
    let _mouse = MOUSE_DEVICE.clone();
    println!("KERN: init trap");
    trap::init();
    trap::enable_timer_interrupt();
    timer::set_next_trigger();
    board::device_init();
    fs::list_apps();
    task::add_initproc();
    *DEV_NON_BLOCKING_ACCESS.exclusive_access() = true;
    task::run_tasks();
    panic!("Unreachable in rust_main!");
}
