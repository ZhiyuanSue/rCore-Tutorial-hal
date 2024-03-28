use arch::{shutdown,VIRT_ADDR_START};
use crate::task::current_kstack_top;
use core::arch::asm;
use core::panic::PanicInfo;
use log::*;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        error!(
            "[kernel] Panicked at {}:{} {}",
            location.file(),
            location.line(),
            info.message().unwrap()
        );
    } else {
        error!("[kernel] Panicked: {}", info.message().unwrap());
    }
	println!("[kernel] get a panic");
    unsafe {
        backtrace();
    }
    shutdown()
}

unsafe fn backtrace() {
    let mut fp: usize;
    let stop = current_kstack_top();
    asm!("mv {}, s0", out(reg) fp);
    println!("---START BACKTRACE---");
    for i in 0..10 {
        if fp == stop || fp == VIRT_ADDR_START{
            break;
        }
        println!("#{}:fp={:#x}:ra={:#x}", i , fp, *( (fp - 8) as *const usize) | VIRT_ADDR_START );
        fp = *((fp - 16) as *const usize);
    }
    println!("---END   BACKTRACE---");
}
