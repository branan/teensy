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

pub struct Pin {
    port: &'static mut Port,
    pin: u32
}

#[repr(C,packed)]
pub struct _Gpio {
    pdor: u32,
    psor: u32,
    pcor: u32,
    ptor: u32,
    pdir: u32,
    pddr: u32
}

pub struct Gpio {
    gpio: &'static mut _Gpio,
    pin: u32
}

impl Port {
    pub unsafe fn new(name: PortName) -> &'static mut Port {
        &mut * match name {
            PortName::C => 0x4004B000 as *mut Port
        }
    }

    pub unsafe fn pin(&mut self, p: u32) -> Pin {
        let myself = &mut * (self as *mut Port);
        Pin { port: myself, pin: p }
    }

    pub fn set_pin_mode(&mut self, p: u32, mut mode: u32) {
        unsafe {
            let mut pcr = core::ptr::read_volatile(&self.pcr[p as usize]);
            pcr &= 0xFFFFF8FF;
            mode &= 0x00000007;
            mode <<= 8;
            pcr |= mode;
            core::ptr::write_volatile(&mut self.pcr[p as usize], pcr);
        }
    }

    pub fn name(&self) -> PortName {
        let addr = (self as *const Port) as u32;
        match addr {
            0x4004B000 => PortName::C,
            _ => unreachable!()
        }
    }
}

impl Pin {
    pub fn make_gpio(self) -> Gpio {
        unsafe {
            self.port.set_pin_mode(self.pin, 1);
            Gpio::new(self.port.name(), self.pin)
        }
    }
}

impl Gpio {
    pub unsafe fn new(port: PortName, pin: u32) -> Gpio {
        let addr = match port {
            PortName::C => 0x400FF080 as *mut _Gpio
        };

        Gpio { gpio: &mut *addr, pin: pin }
    }

    pub fn output(&mut self) {
        unsafe {
            core::ptr::write_volatile(&mut self.gpio.pddr, 1 << self.pin);
        }
    }

    pub fn high(&mut self) {
        unsafe {
            core::ptr::write_volatile(&mut self.gpio.psor, 1 << self.pin);
        }
    }
}
