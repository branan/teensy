use volatile::Volatile;
use bit_field::BitField;

pub enum PortName {
    B,
    C
}

#[repr(C,packed)]
struct PortRegs {
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

pub struct Port {
    port: &'static mut PortRegs,
    _gate: super::sim::ClockGate,
}

pub struct Pin {
    port: *mut Port,
    pin: usize
}

pub struct Tx(u8);
pub struct Rx(u8);

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
    pub unsafe fn new(name: PortName, gate: super::sim::ClockGate) -> Port {
        let myself = &mut * match name {
            PortName::B => 0x4004A000 as *mut PortRegs,
            PortName::C => 0x4004B000 as *mut PortRegs
        };

        Port { port: myself, _gate: gate }
    }

    pub unsafe fn pin(&mut self, p: usize) -> Pin {
        Pin { port: self, pin: p }
    }

    pub unsafe fn set_pin_mode(&mut self, p: usize, mode: u32) {
        self.port.pcr[p].update(|pcr| {
            pcr.set_bits(8..11, mode);
        });
    }

    pub fn name(&self) -> PortName {
        let addr = (self.port as *const PortRegs) as u32;
        match addr {
            0x4004A000 => PortName::B,
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

    pub fn make_rx(self) -> Rx {
        unsafe {
            let port = &mut *self.port;
            match (port.name(), self.pin) {
                (PortName::B, 16) => {
                    port.set_pin_mode(self.pin, 3);
                    Rx(0)
                },
                _ => panic!("Invalid serial RX pin")
            }
        }
    }

    pub fn make_tx(self) -> Tx {
        unsafe {
            let port = &mut *self.port;
            match (port.name(), self.pin) {
                (PortName::B, 17) => {
                    port.set_pin_mode(self.pin, 3);
                    Tx(0)
                },
                _ => panic!("Invalid serial TX pin")
            }
        }
    }
}

impl Gpio {
    pub unsafe fn new(port: PortName, pin: usize) -> Gpio {
        let gpio = match port {
            PortName::B => 0x43FE0800 as *mut GpioBitband,
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

impl Rx {
    pub fn uart(&self) -> u8 {
        self.0
    }
}

impl Tx {
    pub fn uart(&self) -> u8 {
        self.0
    }
}
