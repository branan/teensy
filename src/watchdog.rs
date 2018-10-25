use volatile::Volatile;
use bit_field::BitField;

use core::arch::arm::__NOP;

#[repr(C,packed)]
pub struct Watchdog {
    stctrlh: Volatile<u16>,
    stctrll: Volatile<u16>,
    tovalh: Volatile<u16>,
    tovall: Volatile<u16>,
    winh: Volatile<u16>,
    winl: Volatile<u16>,
    refresh: Volatile<u16>,
    unlock: Volatile<u16>,
    tmrouth: Volatile<u16>,
    tmroutl: Volatile<u16>,
    rstcnt: Volatile<u16>,
    presc: Volatile<u16>
}

impl Watchdog {
    pub unsafe fn new() -> &'static mut Watchdog {
        &mut *(0x40052000 as *mut Watchdog)
    }

    pub fn disable(&mut self) {
        unsafe {
            self.unlock.write(0xC520);
            self.unlock.write(0xD928);
            __NOP();
            __NOP();
            self.stctrlh.update(|ctrl| {
                ctrl.set_bit(0, false);
            });
        }
    }
}
