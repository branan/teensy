#![feature(asm,panic_handler,panic_info_message,tool_lints)]
#![no_std]
#![no_main]
#![no_builtins]

#![deny(warnings)]
#![allow(unknown_lints)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::empty_loop)]

extern crate volatile;
extern crate bit_field;

mod mcg;
mod osc;
mod port;
mod sim;
mod uart;
mod watchdog;

use mcg::*;
use osc::*;
use port::*;
use sim::*;
use uart::*;
use watchdog::*;

use core::slice;
use core::fmt::Write;
use volatile::Volatile;

static mut PORT: Option<Port> = None;
static mut WRITER: Option<Uart<'static, 'static>> = None;

extern fn main() {
    unsafe {
        Watchdog::new().disable();
        setup_bss();
    }

    // Enable the crystal oscillator with 10pf of capacitance
    let osc_token = Osc::new().enable(10);

    // Set our clocks:
    // core: 72Mhz
    // peripheral: 36MHz
    // flash: 24MHz
    let mut sim = Sim::new();
    sim.set_dividers(1, 2, 3);
    // We would also set the USB divider here if we wanted to use it.

    // Now we can start setting up the MCG for our needs.
    let mcg = Mcg::new();
    if let Clock::Fei(mut fei) = mcg.clock() {
        // Our 16MHz xtal is "very fast", and needs to be divided
        // by 512 to be in the acceptable FLL range.
        fei.enable_xtal(OscRange::VeryHigh, osc_token);
        let fbe = fei.use_external(512);

        // PLL is 27/6 * xtal == 72MHz
        let pbe = fbe.enable_pll(27, 6);
        pbe.use_pll();
    } else {
        panic!("Somehow the clock wasn't in FEI mode");
    }

    // Initialize the UART as our panic writer. This is unsafe because
    // we are modifying a global variable.
    unsafe {
        PORT = Some(sim.port(PortName::B));
        let rx = PORT.as_ref().unwrap().pin(16).make_rx();
        let tx = PORT.as_ref().unwrap().pin(17).make_tx();
        WRITER = Some(sim.uart(0, Some(rx), Some(tx), (468, 24)));
    };

    let portc = sim.port(PortName::C);
    let mut gpio = portc.pin(5).make_gpio();
    gpio.output();
    gpio.high();

    loop {};
}

extern {
    fn _stack_top();
    static mut _bss_start: u8;
    static mut _bss_end: u8;
}

unsafe fn setup_bss() {
    let bss_start = &mut _bss_start as *mut u8;
    let bss_end = &mut _bss_end as *mut u8;
    let bss_len = bss_end as usize - bss_start as usize;
    let bss = slice::from_raw_parts_mut(bss_start, bss_len);
    for b in &mut bss.iter_mut() {
        *b = 0;
    }
}

#[link_section = ".vectors"]
#[no_mangle]
pub static _VECTORS: [unsafe extern fn(); 2] = [
    _stack_top,
    main,
];

#[link_section = ".flashconfig"]
#[no_mangle]
pub static _FLASHCONFIG: [u8; 16] = [
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0xDE, 0xF9, 0xFF, 0xFF
];

#[panic_handler]
fn teensy_panic(pi: &core::panic::PanicInfo) -> ! {
    if let Some(uart) = unsafe { WRITER.as_mut() } {
        write!(uart, "Panic occured! ");
        if let Some(format_args) = pi.message() {
            core::fmt::write(uart, *format_args).unwrap();
        }
    }

    // Reset the MCU after we've printed our panic.
    let aircr = unsafe {
        &mut *(0xE000ED0C as *mut Volatile<u32>)
    };
    aircr.write(0x05FA0004);
    unreachable!();
}
