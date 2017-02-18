use volatile::Volatile;
use bit_field::BitField;

use core::sync::atomic::{AtomicBool,ATOMIC_BOOL_INIT,Ordering};

#[repr(C,packed)]
struct OscRegs {
    cr: Volatile<u8>
}

pub struct Osc {
    osc: &'static mut OscRegs
}

static OSC_INIT: AtomicBool = ATOMIC_BOOL_INIT;

impl Osc {
    pub fn new() -> Osc {
        let was_init = OSC_INIT.swap(true, Ordering::Relaxed);
        if was_init {
            panic!("Cannot initialize OSC: It's already active");
        }
        let regs = unsafe { &mut *(0x40065000 as *mut OscRegs) };
        Osc {osc: regs}
    }

    pub fn enable(&mut self, capacitance: u8) {
        if capacitance % 2 == 1 || capacitance > 30 {
            panic!("Invalid crystal capacitance value: {}", capacitance)
        }

        let mut cr: u8 = 0;

        // The capacitance control bits are backwards, and start at 2pf
        // We swizzle them all here
        cr.set_bit(3, capacitance.get_bit(1));
        cr.set_bit(2, capacitance.get_bit(2));
        cr.set_bit(1, capacitance.get_bit(3));
        cr.set_bit(0, capacitance.get_bit(4));

        // enable the crystal oscillator
        cr.set_bit(7, true);

        self.osc.cr.write(cr);
    }
}

impl Drop for Osc {
    fn drop(&mut self) {
        OSC_INIT.store(false, Ordering::Relaxed);
    }
}
