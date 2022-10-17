use core;

#[derive(Clone,Copy)]
pub enum PortName {
    C
}

#[repr(C,packed)]
pub struct Port {
    pcr: [u32; 32],
    gpclr: u32,
    gpchr: u32,
    reserved_0: [u8; 24],
    isfr: u32,
}

pub struct Pin {
    port: *mut Port,
    pin: usize
}

#[repr(C,packed)]
struct GpioBitband {
    pdor: [u32; 32],
    psor: [u32; 32],
    pcor: [u32; 32],
    ptor: [u32; 32],
    pdir: [u32; 32],
    pddr: [u32; 32]
}

pub struct Gpio {
    gpio: *mut GpioBitband,
    pin: usize
}

impl Port {
    pub unsafe fn new(name: PortName) -> &'static mut Port {
        &mut * match name {
            PortName::C => 0x4004B000 as *mut Port
        }
    }

    pub unsafe fn pin(&mut self, p: usize) -> Pin {
        Pin { port: self, pin: p }
    }

    pub unsafe fn set_pin_mode(&mut self, p: usize, mut mode: u32) {

        let brw = self.pcr[p];
        // let mut pcr = core::ptr::read_volatile(&self.pcr[p]);
        let mut pcr = core::ptr::read_volatile(&brw);
        pcr &= 0xFFFFF8FF;
        mode &= 0x00000007;
        mode <<= 8;
        pcr |= mode;
        let mut brw = self.pcr[p];
        core::ptr::write_volatile(&mut brw, pcr);
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
            let port = &mut *self.port;
            port.set_pin_mode(self.pin, 1);
            Gpio::new(port.name(), self.pin)
        }
    }
}

impl Gpio {
    pub unsafe fn new(port: PortName, pin: usize) -> Gpio {
        let gpio = match port {
            PortName::C => 0x43FE1000 as *mut GpioBitband
        };

        Gpio { gpio, pin }
    }

    pub fn output(&mut self) {
        unsafe {
            let mut brw = (*self.gpio).pddr[self.pin];
            // core::ptr::write_volatile(&mut (*self.gpio).pddr[self.pin], 1);
            core::ptr::write_volatile(&mut brw, 1);
        }
    }

    pub fn high(&mut self) {
        unsafe {
            let mut brw = (*self.gpio).psor[self.pin];
            // core::ptr::write_volatile(&mut (*self.gpio).psor[self.pin], 1);
            core::ptr::write_volatile(&mut brw, 1);
        }
    }
}
