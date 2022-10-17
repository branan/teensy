use core;
use core::arch::arm::__nop;

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
            let mut brw = self.unlock;
            // core::ptr::write_volatile(&mut self.unlock, 0xC520);
            core::ptr::write_volatile(&mut brw, 0xC520);
            // core::ptr::write_volatile(&mut self.unlock, 0xD928);
            core::ptr::write_volatile(&mut brw, 0xD928);
            __nop();
            __nop();
            let brw = self.stctrlh;
            // let mut ctrl = core::ptr::read_volatile(&self.stctrlh);
            let mut ctrl = core::ptr::read_volatile(&brw);
            ctrl &= !(0x00000001);
            let mut brw = self.stctrlh;
            // core::ptr::write_volatile(&mut self.stctrlh, ctrl);
            core::ptr::write_volatile(&mut brw, ctrl);
        }
    }
}
