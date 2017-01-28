use volatile::Volatile;
use bit_field::BitField;

pub enum PortName {
    C
}

#[repr(C,packed)]
pub struct Port {
    pcr: [Volatile<u32>; 32],
    gplcr: Volatile<u32>,
    gpchr: Volatile<u32>,
    reserved_0: [u8; 24],
    isfr: Volatile<u32>,
    reserved_1: [u8; 28],
    dfer: Volatile<u32>,
    dfcr: Volatile<u32>,
    dfwr: Volatile<u32>
}

pub struct Pin {
    port: *mut Port,
    pin: usize
}

#[repr(C,packed)]
struct GpioBitband {
    pdor: [Volatile<u32>; 32],
    psor: [Volatile<u32>; 32],
    pcor: [Volatile<u32>; 32],
    ptor: [Volatile<u32>; 32],
    pdir: [Volatile<u32>; 32],
    pddr: [Volatile<u32>; 32]
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

    pub unsafe fn set_pin_mode(&mut self, p: usize, mode: u32) {
        self.pcr[p].update(|pcr| {
            pcr.set_bits(8..11, mode);
        });
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

        Gpio { gpio: gpio, pin: pin }
    }

    pub fn output(&mut self) {
        unsafe {
            (&mut (*self.gpio)).pddr[self.pin].write(1);
        }
    }

    pub fn high(&mut self) {
        unsafe {
            (&mut (*self.gpio)).psor[self.pin].write(1);
        }
    }
}
