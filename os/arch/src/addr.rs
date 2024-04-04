use core::{
    ffi::CStr,
    fmt::{Debug, Display},
    mem::size_of,
    ops::Add,
};

use crate::{PAGE_SIZE, VIRT_ADDR_START};

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PhysAddr(pub(crate) usize);
impl From<PhysPage> for PhysAddr {
    fn from(value: PhysPage) -> Self {
        Self(value.0 << 12)
    }
}

impl PhysAddr {
    #[inline]
    pub fn addr(&self) -> usize {
        self.0
    }

    #[inline]
    pub fn get_ptr<T>(&self) -> *const T {
        (self.0 | VIRT_ADDR_START) as *const T
    }

    #[inline]
    pub const fn get_mut_ptr<T>(&self) -> *mut T {
        (self.0 | VIRT_ADDR_START) as *mut T
    }

    #[inline]
    pub fn slice_with_len<T>(&self, len: usize) -> &'static [T] {
        unsafe { core::slice::from_raw_parts(self.get_ptr(), len) }
    }

    #[inline]
    pub fn slice_mut_with_len<T>(&self, len: usize) -> &'static mut [T] {
        unsafe { core::slice::from_raw_parts_mut(self.get_mut_ptr(), len) }
    }

    #[inline]
    pub fn get_cstr(&self) -> &CStr {
        unsafe { CStr::from_ptr(self.get_ptr::<i8>()) }
    }
    pub fn floor(&self) -> PhysPage {
        PhysPage(self.0 / PAGE_SIZE)
    }
    pub fn ceil(&self) -> PhysPage {
        if self.0 == 0 {
            PhysPage(0)
        } else {
            PhysPage((self.0 - 1 + PAGE_SIZE) / PAGE_SIZE)
        }
    }
    pub fn page_offset(&self) -> usize {
        self.0 & (PAGE_SIZE - 1)
    }
    pub fn aligned(&self) -> bool {
        self.page_offset() == 0
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct VirtAddr(pub(crate) usize);

impl From<usize> for VirtAddr {
    fn from(value: usize) -> Self {
        Self(value)
    }
}
impl From<usize> for PhysAddr {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl From<VirtAddr> for usize {
    fn from(value: VirtAddr) -> Self {
        value.0
    }
}
impl From<PhysAddr> for usize {
    fn from(value: PhysAddr) -> Self {
        value.0
    }
}

impl VirtAddr {
    #[inline]
    pub fn addr(&self) -> usize {
        self.0
    }

    #[inline]
    pub fn get_ptr<T>(&self) -> *const T {
        self.0 as *const T
    }

    #[inline]
    pub fn get_mut_ptr<T>(&self) -> *mut T {
        self.0 as *mut T
    }

    #[inline]
    pub fn get_ref<T>(&self) -> &'static T {
        unsafe { &*(self.0 as *const T) }
    }

    #[inline]
    pub fn get_mut_ref<T>(&self) -> &'static mut T {
        unsafe { &mut *(self.0 as *mut T) }
    }

    #[inline]
    pub fn slice_with_len<T>(&self, len: usize) -> &'static [T] {
        unsafe { core::slice::from_raw_parts(self.get_ptr(), len) }
    }

    #[inline]
    pub fn slice_mut_with_len<T>(&self, len: usize) -> &'static mut [T] {
        unsafe { core::slice::from_raw_parts_mut(self.get_mut_ptr(), len) }
    }

    #[inline]
    pub fn slice_until<T>(&self, is_valid: fn(T) -> bool) -> &'static mut [T] {
        let ptr = self.addr() as *mut T;
        unsafe {
            let mut len = 0;
            if !ptr.is_null() {
                loop {
                    if !is_valid(ptr.add(len).read()) {
                        break;
                    }
                    len += 1;
                }
            }
            core::slice::from_raw_parts_mut(ptr, len)
        }
    }

    #[inline]
    pub fn get_cstr(&self) -> &CStr {
        unsafe { CStr::from_ptr(self.get_ptr::<i8>()) }
    }
	pub fn floor(&self) -> VirtPage {
        VirtPage(self.0 / PAGE_SIZE)
    }
    pub fn ceil(&self) -> VirtPage {
        if self.0 == 0 {
            VirtPage(0)
        } else {
            VirtPage((self.0 - 1 + PAGE_SIZE) / PAGE_SIZE)
        }
    }
    pub fn page_offset(&self) -> usize {
        self.0 & (PAGE_SIZE - 1)
    }
    pub fn aligned(&self) -> bool {
        self.page_offset() == 0
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PhysPage(pub(crate) usize);

impl From<usize> for PhysPage {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl From<PhysAddr> for PhysPage {
    fn from(value: PhysAddr) -> Self {
        Self(value.0 >> 12)
    }
}

impl From<PhysPage> for usize {
    fn from(value: PhysPage) -> Self {
        value.0
    }
}

impl Add<PhysPage> for PhysPage {
    type Output = PhysPage;

    fn add(self, rhs: PhysPage) -> Self::Output {
        PhysPage(self.0 + rhs.0)
    }
}

impl Add<usize> for PhysPage {
    type Output = PhysPage;

    fn add(self, rhs: usize) -> Self::Output {
        PhysPage(self.0 + rhs)
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct VirtPage(pub(crate) usize);
impl From<VirtAddr> for VirtPage {
    fn from(value: VirtAddr) -> Self {
        Self(value.0 >> 12)
    }
}
impl From<usize> for VirtPage {
    fn from(value: usize) -> Self {
        Self(value)
    }
}
impl From<VirtPage> for usize {
    fn from(value: VirtPage) -> Self {
        value.0
    }
}

impl PhysPage {
    #[inline]
    pub const fn new(ppn: usize) -> Self {
        Self(ppn)
    }

    #[inline]
    pub const fn from_addr(addr: usize) -> Self {
        Self(addr >> 12)
    }

    #[inline]
    pub const fn to_addr(&self) -> usize {
        self.0 << 12
    }

    #[inline]
    pub const fn get_buffer(&self) -> &'static mut [u8] {
        unsafe {
            core::slice::from_raw_parts_mut((self.0 << 12 | VIRT_ADDR_START) as *mut u8, PAGE_SIZE)
        }
    }

    #[inline]
    pub fn copy_value_from_another(&self, ppn: PhysPage) {
        self.get_buffer().copy_from_slice(&ppn.get_buffer());
        #[cfg(c906)]
        unsafe {
            asm!(".long 0x0010000b"); // dcache.all
            asm!(".long 0x01b0000b"); // sync.is
        }
    }

    #[inline]
    pub fn drop_clear(&self) {
        // self.get_buffer().fill(0);
        unsafe {
            core::slice::from_raw_parts_mut(
                (self.0 << 12 | VIRT_ADDR_START) as *mut usize,
                PAGE_SIZE / size_of::<usize>(),
            )
            .fill(0);
        }
        #[cfg(c906)]
        unsafe {
            asm!(".long 0x0010000b"); // dcache.all
            asm!(".long 0x01b0000b"); // sync.is
        }
    }
}

impl Add<usize> for VirtPage {
    type Output = VirtPage;

    fn add(self, rhs: usize) -> Self::Output {
        VirtPage(self.0 + rhs)
    }
}

impl From<VirtPage> for VirtAddr {
    fn from(value: VirtPage) -> Self {
        Self(value.to_addr())
    }
}

impl PhysAddr {
    #[inline]
    pub const fn new(addr: usize) -> Self {
        Self(addr)
    }
}

impl VirtPage {
    #[inline]
    pub const fn new(vpn: usize) -> Self {
        Self(vpn)
    }

    #[inline]
    pub const fn from_addr(addr: usize) -> Self {
        Self(addr >> 12)
    }
    #[inline]
    pub const fn to_addr(&self) -> usize {
        self.0 << 12
    }
	pub fn indexes(&self) -> [usize; 3] {
        let mut vpn = self.0;
        let mut idx = [0usize; 3];
        for i in (0..3).rev() {
            idx[i] = vpn & 511;
            vpn >>= 9;
        }
        idx
    }
}

impl VirtAddr {
    #[inline]
    pub const fn new(addr: usize) -> Self {
        Self(addr)
    }
}

impl Display for PhysPage {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("{:#x}", self.0))
    }
}

impl Display for PhysAddr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("{:#x}", self.0))
    }
}

impl Display for VirtPage {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("{:#x}", self.0))
    }
}

impl Display for VirtAddr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("{:#x}", self.0))
    }
}

impl Debug for PhysPage {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("{:#x}", self.0))
    }
}

impl Debug for PhysAddr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("{:#x}", self.0))
    }
}

impl Debug for VirtPage {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("{:#x}", self.0))
    }
}

impl Debug for VirtAddr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("{:#x}", self.0))
    }
}
impl PhysAddr {
    pub fn get_ref<T>(&self) -> &'static T {
        unsafe { (self.0 as *const T).as_ref().unwrap() }
    }
    pub fn get_mut<T>(&self) -> &'static mut T {
        unsafe { (self.0 as *mut T).as_mut().unwrap() }
    }
}
impl PhysPage {
    pub fn get_bytes_array(&self) -> &'static mut [u8] {
        let pa: PhysAddr = (*self).into();
        unsafe { core::slice::from_raw_parts_mut((pa.0|VIRT_ADDR_START) as *mut u8, 4096) }
    }
    pub fn get_mut<T>(&self) -> &'static mut T {
        let pa: PhysAddr = (*self).into();
        pa.get_mut()
    }
}
impl VirtAddr {
    pub fn get_mut<T>(&self) -> &'static mut T {
        unsafe { (self.0 as *mut T).as_mut().unwrap() }
    }
}
impl VirtPage {
    pub fn get_bytes_array(&self) -> &'static mut [u8] {
        let pa: VirtAddr = (*self).into();
        unsafe { core::slice::from_raw_parts_mut((pa.0|VIRT_ADDR_START) as *mut u8, 4096) }
    }
    pub fn get_mut<T>(&self) -> &'static mut T {
        let pa: VirtAddr = (*self).into();
        pa.get_mut()
    }
}
pub trait StepByOne {
    fn step(&mut self);
}
impl StepByOne for VirtPage {
    fn step(&mut self) {
        self.0 += 1;
    }
}
impl StepByOne for PhysPage {
    fn step(&mut self) {
        self.0 += 1;
    }
}
#[derive(Copy, Clone)]
pub struct SimpleRange<T>
where
    T: StepByOne + Copy + PartialEq + PartialOrd + Debug,
{
    l: T,
    r: T,
}
impl<T> SimpleRange<T>
where
    T: StepByOne + Copy + PartialEq + PartialOrd + Debug,
{
    pub fn new(start: T, end: T) -> Self {
        assert!(start <= end, "start {:?} > end {:?}!", start, end);
        Self { l: start, r: end }
    }
    pub fn get_start(&self) -> T {
        self.l
    }
    pub fn get_end(&self) -> T {
        self.r
    }
}
impl<T> IntoIterator for SimpleRange<T>
where
    T: StepByOne + Copy + PartialEq + PartialOrd + Debug,
{
    type Item = T;
    type IntoIter = SimpleRangeIterator<T>;
    fn into_iter(self) -> Self::IntoIter {
        SimpleRangeIterator::new(self.l, self.r)
    }
}
pub struct SimpleRangeIterator<T>
where
    T: StepByOne + Copy + PartialEq + PartialOrd + Debug,
{
    current: T,
    end: T,
}
impl<T> SimpleRangeIterator<T>
where
    T: StepByOne + Copy + PartialEq + PartialOrd + Debug,
{
    pub fn new(l: T, r: T) -> Self {
        Self { current: l, end: r }
    }
}
impl<T> Iterator for SimpleRangeIterator<T>
where
    T: StepByOne + Copy + PartialEq + PartialOrd + Debug,
{
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current == self.end {
            None
        } else {
            let t = self.current;
            self.current.step();
            Some(t)
        }
    }
}
pub type VPNRange = SimpleRange<VirtPage>;