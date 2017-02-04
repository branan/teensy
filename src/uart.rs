use volatile::Volatile;
use bit_field::BitField;

use core;

use super::port::{Rx,Tx};

#[repr(C,packed)]
pub struct Uart {
    bdh: Volatile<u8>,
    bdl: Volatile<u8>,
    c1: Volatile<u8>,
    c2: Volatile<u8>,
    s1: Volatile<u8>,
    s2: Volatile<u8>,
    c3: Volatile<u8>,
    d: Volatile<u8>,
    ma1: Volatile<u8>,
    ma2: Volatile<u8>,
    c4: Volatile<u8>,
    c5: Volatile<u8>,
    ed: Volatile<u8>,
    modem: Volatile<u8>,
    ir: Volatile<u8>,
}


impl Uart {
    pub unsafe fn new(id: u8, rx: Option<Rx>, tx: Option<Tx>, clkdiv: (u16,u8)) -> &'static mut Uart {
        if let Some(r) = rx.as_ref() {
            if r.uart() != id {
                panic!("Invalid RX pin for UART {}", id);
            }
        }
        if let Some(t) = tx.as_ref() {
            if t.uart() != id {
                panic!("Invalid TX pin for UART {}", id);
            }
        }
        if clkdiv.0 >= 8192 {
            panic!("Invalid UART clock divider: {}", clkdiv.0);
        }
        if clkdiv.1 >= 32 {
            panic!("Invalid UART fractional divisor: {}", clkdiv.1);
        }

        let uart = match id {
            0 => &mut *(0x4006A000 as *mut Uart),
            _ => panic!("Invalid UART id: {}", id)
        };

        uart.c4.update(|c4| {
            c4.set_bits(0..5, clkdiv.1);
        });
        uart.bdh.update(|bdh| {
            bdh.set_bits(0..5, clkdiv.0.get_bits(8..13) as u8);
        });
        uart.bdl.write(clkdiv.0.get_bits(0..8) as u8);

        uart.c2.update(|c2| {
            c2.set_bit(2, rx.is_some());
            c2.set_bit(3, tx.is_some());
        });

        uart
    }
}

impl core::fmt::Write for Uart {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for b in s.bytes() {
            while !self.s1.read().get_bit(7) {}
            self.d.write(b);
        }
        Ok(())
    }
}
