#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]

//use crate::drivers::{GPU_DEVICE, KEYBOARD_DEVICE, MOUSE_DEVICE, INPUT_CONDVAR};
use crate::drivers::{GPU_DEVICE, KEYBOARD_DEVICE, MOUSE_DEVICE};
use arch::shutdown;
use arch::{
	ArchInterface,Context,TrapType,PhysPage
};
use fdt::node::FdtNode;
use log::info;
use crate::console::stdout_init;
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
mod sync;
mod syscall;
mod task;
mod timer;
mod trap;

use crate::drivers::chardev::CharDevice;
use crate::drivers::chardev::UART;

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
		mm::init();
		UART.init();
        let str = include_str!("logo.txt");
        println!("{}", str);
		stdout_init(Some("info"));
		info!("hello, rCore turtorial");
    }
	fn kernel_interrupt(ctx: &mut Context, trap_type: TrapType)
	{
		println!("[kernel] kernel interrupt");
		println!("KERN: init trap");
		trap::init();
		trap::enable_timer_interrupt();
		timer::set_next_trigger();
	}
	fn add_memory_region(start: usize, end: usize)
	{

	}
	fn main(hartid: usize)
	{
		println!("[kernel] main start");
		
		// trap::init();
		// trap::enable_timer_interrupt();
		// timer::set_next_trigger();
		// board::device_init();
		fs::list_apps();
		task::add_initproc();
		*DEV_NON_BLOCKING_ACCESS.exclusive_access() = true;
		task::run_tasks();
		panic!("Unreachable in rust_main!");
	}
	fn frame_alloc_persist() -> PhysPage
	{
		mm::frame_alloc().unwrap().ppn
	}
	fn frame_unalloc(ppn: PhysPage)
	{
		mm::frame_dealloc(ppn)
	}
	fn prepare_drivers()
	{
		println!("[kernel] prepare drivers");
		board::device_init();
	}
	fn try_to_add_device(fdtNode: &FdtNode)
	{
		println!("[kernel] try to add device");
		// println!("KERN: init gpu");
		let _gpu = GPU_DEVICE.clone();
		// println!("KERN: init keyboard");
		let _keyboard = KEYBOARD_DEVICE.clone();
		// println!("KERN: init mouse");
		let _mouse = MOUSE_DEVICE.clone();
	}
}