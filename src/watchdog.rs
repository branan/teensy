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
            let unlock_ptr = core::ptr::addr_of_mut!(self.unlock);
            unlock_ptr.write_volatile(0xC520);
            unlock_ptr.write_volatile(0xD928);
            __nop();
            __nop();
            let ctrl_ptr = core::ptr::addr_of_mut!(self.stctrlh);
            let mut ctrl = ctrl_ptr.read_volatile();
            ctrl &= !(0x00000001);
            ctrl_ptr.write_volatile(ctrl);

        }
    }
}
