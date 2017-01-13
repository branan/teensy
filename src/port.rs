use core;

pub enum PortName {
    C
}

#[repr(C,packed)]
pub struct Port {
    pcr: [u32; 32],
    gplcr: u32,
    gpchr: u32,
    reserved_0: [u8; 24],
    isfr: u32,
    reserved_1: [u8; 28],
    dfer: u32,
    dfcr: u32,
    dfwr: u32
}

pub struct Pin<'a> {
    port: &'a mut Port,
    pin: u32
}

pub struct Gpio {
    pddr: &'static mut u32,
    psor: &'static mut u32
}

impl Port {
    pub unsafe fn new(name: PortName) -> &'static mut Port {
        &mut * match name {
            PortName::C => 0x4004B000 as *mut Port
        }
    }

    pub unsafe fn pin(&mut self, p: u32) -> Pin {
        Pin { port: self, pin: p }
    }

    pub unsafe fn set_pin_mode(&mut self, p: u32, mut mode: u32) {
        let mut pcr = core::ptr::read_volatile(&self.pcr[p as usize]);
        pcr &= 0xFFFFF8FF;
        mode &= 0x00000007;
        mode <<= 8;
        pcr |= mode;
        core::ptr::write_volatile(&mut self.pcr[p as usize], pcr);
    }

    pub fn name(&self) -> PortName {
        let addr = (self as *const Port) as u32;
        match addr {
            0x4004B000 => PortName::C,
            _ => unreachable!()
        }
    }
}

impl<'a> Pin<'a> {
    pub fn make_gpio(self) -> Gpio {
        unsafe {
            self.port.set_pin_mode(self.pin, 1);
            Gpio::new(self.port.name(), self.pin)
        }
    }
}

impl Gpio {
    pub unsafe fn new(port: PortName, pin: u32) -> Gpio {
        let gpio_base = match port {
            PortName::C => 0x43FE1000
        };

        // PSOR is the second field of the GPIO struct.
        // PDDR is the 6thh field. (zero indexed)
        // Each field is 128 bytes long
        // That is: 32 pins, each taking up 32 bits (4 bytes)
        let psor = (gpio_base + 1 * 0x80 + pin) as *mut u32;
        let pddr = (gpio_base + 5 * 0x80 + pin) as *mut u32;
        Gpio { psor: &mut *psor, pddr: &mut *pddr }
    }

    pub fn output(&mut self) {
        unsafe {
            core::ptr::write_volatile(self.pddr, 1);
        }
    }

    pub fn high(&mut self) {
        unsafe {
            core::ptr::write_volatile(self.psor, 1);
        }
    }
}
