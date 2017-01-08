use core;

#[repr(C,packed)]
pub struct Watchdog {
    stctrlh: u16,
    stctrll: u16,
    tovalh: u16,
    tovall: u16,
    winh: u16,
    winl: u16,
    refresh: u16,
    unlock: u16,
    tmrouth: u16,
    tmroutl: u16,
    rstcnt: u16,
    presc: u16
}

impl Watchdog {
    pub unsafe fn new() -> &'static mut Watchdog {
        &mut *(0x40052000 as *mut Watchdog)
    }

    pub fn disable(&mut self) {
        unsafe {
            core::ptr::write_volatile(&mut self.unlock, 0xC520);
            core::ptr::write_volatile(&mut self.unlock, 0xD928);
            asm!("nop" : : : "memory");
            asm!("nop" : : : "memory");
            let mut ctrl = core::ptr::read_volatile(&self.stctrlh);
            ctrl &= !(0x00000001);
            core::ptr::write_volatile(&mut self.stctrlh, ctrl);
        }
    }
}
