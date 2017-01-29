use core::mem;
use volatile::Volatile;
use bit_field::BitField;

#[repr(C,packed)]
pub struct Mcg {
    c1: Volatile<u8>,
    c2: Volatile<u8>,
    c3: Volatile<u8>,
    c4: Volatile<u8>,
    c5: Volatile<u8>,
    c6: Volatile<u8>,
    s: Volatile<u8>,
    _pad0: u8,
    sc: Volatile<u8>,
    _pad1: u8,
    atcvh: Volatile<u8>,
    atcvl: Volatile<u8>,
    c7: Volatile<u8>,
    c8: Volatile<u8>,
}

pub struct Fei {
    mcg: &'static mut Mcg
}

pub struct Fbe {
    mcg: &'static mut Mcg
}

pub struct Pbe {
    mcg: &'static mut Mcg
}

pub enum Clock {
    Fei(Fei),
    Fbe(Fbe),
    Pbe(Pbe)
}

impl Mcg {
    pub unsafe fn new() -> &'static mut Mcg {
        &mut *(0x40064000 as *mut Mcg)
    }

    pub fn clock(&'static mut self) -> Clock {
        let source: OscSource = unsafe {
            mem::transmute(self.c1.read().get_bits(6..8))
        };
        let fll_internal = self.c1.read().get_bit(2);
        let pll_enabled = self.c6.read().get_bit(6);

        match (fll_internal, pll_enabled, source) {
            (true, false, OscSource::LockedLoop) => Clock::Fei(Fei{ mcg: self }),
            (false, false, OscSource::External) => Clock::Fbe(Fbe{ mcg: self }),
            (_, true, OscSource::External) => Clock::Pbe(Pbe{ mcg: self }),
            _ => panic!("The current clock mode cannot be represented as a known struct")
        }
    }
}

#[allow(dead_code)]
pub enum OscRange {
    Low = 0,
    High = 1,
    VeryHigh = 2
}

#[allow(dead_code)]
enum OscSource {
    LockedLoop = 0,
    Internal = 1,
    External = 2
}

impl Fei {
    pub fn enable_xtal(&mut self, range: OscRange) {
        let mut c2: u8 = 0;
        c2.set_bits(4..6, range as u8);
        c2.set_bit(2, true);
        self.mcg.c2.write(c2);
    }

    pub fn use_external(self, divide: u32) -> Fbe {
        let osc = self.mcg.c2.read().get_bits(4..6);
        let frdiv = if osc == OscRange::Low as u8 {
            match divide {
                1 => 0,
                2 => 1,
                4 => 2,
                8 => 3,
                16 => 4,
                32 => 5,
                64 => 6,
                128 => 7,
                _ => panic!("Invalid external clock divider: {}", divide)
            }
        } else {
            match divide {
                32 => 0,
                64 => 1,
                128 => 2,
                256 => 3,
                512 => 4,
                1024 => 5,
                1280 => 6,
                1536 => 7,
                _ => panic!("Invalid external clock divider: {}", divide)
            }
        };

        let mut c1: u8 = 0;
        c1.set_bits(3..6, frdiv);
        c1.set_bits(6..8, OscSource::External as u8);
        self.mcg.c1.write(c1);

        // Once we write to the control register, we need to wait for
        // the new clock to stabilize before we move on.
        // First: Wait for the crystal oscilator to become enabled
        // Next: Wait for the FLL to be pointed at the crystal
        // Last: Wait for our clock source to be the crystal osc
        while !self.mcg.s.read().get_bit(1) {}
        while self.mcg.s.read().get_bit(4) {}
        while self.mcg.s.read().get_bits(2..4) != OscSource::External as u8 {}

        Fbe { mcg: self.mcg }
    }
}

impl Fbe {
    pub fn enable_pll(self, numerator: u8, denominator: u8) -> Pbe {
        if numerator < 24 || numerator > 55 {
            panic!("Invalid PLL VCO divide factor: {}", numerator);
        }

        if denominator < 1 || denominator > 25 {
            panic!("Invalid PLL reference divide factor: {}", denominator);
        }

        self.mcg.c5.update(|c5| {
            c5.set_bits(0..5, denominator - 1);
        });

        self.mcg.c6.update(|c6| {
            c6.set_bits(0..6, numerator - 24);
            c6.set_bit(6, true);
        });

        // Wait for PLL to be enabled, using the crystal oscillator
        while !self.mcg.s.read().get_bit(5) {}
        // Wait for the PLL to be "locked" and stable
        while !self.mcg.s.read().get_bit(6) {}

        Pbe { mcg: self.mcg }
    }
}

impl Pbe {
    pub fn use_pll(self) {
        self.mcg.c1.update(|c1| {
            c1.set_bits(6..8, OscSource::LockedLoop as u8);
        });

        // mcg.c1 and mcg.s have slightly different behaviors.  In c1,
        // we use one value to indicate "Use whichever LL is
        // enabled". In s, it is differentiated between the FLL at 0,
        // and the PLL at 3. Instead of adding a value to OscSource
        // which would be invalid to set, we just check for the known
        // value "3" here.
        while self.mcg.s.read().get_bits(2..4) != 3 {}
    }
}
