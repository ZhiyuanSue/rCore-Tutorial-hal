pub const VIRT_ADDR_START: usize = 0xffff_ff80_0000_0000;
pub const USER_ADDR_MAX: usize = 0xbf_ffff_ffff;
pub const PAGE_SIZE: usize = 4096;
pub const PAGE_ITEM_COUNT: usize = 512;
pub const SIG_RETURN_ADDR: usize = 0xFFFF_FF80_0000_0000;

pub const SYSCALL_VECTOR: usize = 0x33445566;
