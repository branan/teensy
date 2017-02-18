#![feature(lang_items,asm,plugin)]
#![plugin(clippy)]
//#![deny(warnings)]
#![no_std]
#![no_main]
#![no_builtins]

extern crate volatile;
extern crate bit_field;

mod mcg;
mod osc;
mod port;
mod sim;
mod uart;
mod watchdog;

use core::slice;
use core::fmt::Write;
use volatile::Volatile;

static mut WRITER: Option<&'static mut uart::Uart> = None;

#[allow(empty_loop)]
extern fn main() {
    let wdog = unsafe {
        watchdog::Watchdog::new()
    };
    wdog.disable();

    let (mcg,osc) = unsafe {
        (mcg::Mcg::new(),
         osc::Osc::new())
    };

    let mut sim = sim::Sim::new();

    unsafe { setup_bss() };
    // Enable the crystal oscillator with 10pf of capacitance
    osc.enable(10);
    sim.enable_clock(sim::Clock::Uart0);
    // Set our clocks:
    // core: 72Mhz
    // peripheral: 36MHz
    // flash: 24MHz
    sim.set_dividers(1, 2, 3);
    // We would also set the USB divider here if we wanted to use it.

    // Now we can start setting up the MCG for our needs.
    if let mcg::Clock::Fei(mut fei) = mcg.clock() {
        // Our 16MHz xtal is "very fast", and needs to be divided
        // by 512 to be in the acceptable FLL range.
        fei.enable_xtal(mcg::OscRange::VeryHigh);
        let fbe = fei.use_external(512);

        // PLL is 27/6 * xtal == 72MHz
        let pbe = fbe.enable_pll(27, 6);
        pbe.use_pll();
    } else {
        panic!("Somehow the clock wasn't in FEI mode");
    }
    let mut portb = sim.port(port::PortName::B);
    let mut portc = sim.port(port::PortName::C);

    // Initialize the UART as our panic writer
    unsafe {
        let rx = portb.pin(16).make_rx();
        let tx = portb.pin(17).make_tx();
        let uart = uart::Uart::new(0, Some(rx), Some(tx), (468, 24));
        WRITER = Some(uart);
    };

    let pin = unsafe { portc.pin(5) };

    let mut gpio = pin.make_gpio();

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

#[lang = "panic_fmt"]
#[no_mangle]
#[allow(empty_loop)]
pub extern fn rust_begin_panic(msg: core::fmt::Arguments,
                               file: &'static str,
                               line: u32) -> ! {
    if let Some(mut uart) = unsafe { WRITER.as_mut() } {
        write!(uart, "panicked at '").unwrap();
        uart.write_fmt(msg).unwrap();
        write!(uart, "', {}:{}\n", file, line).unwrap();
    }

    // Reset the MCU after we've printed our panic.
    let mut aircr = unsafe {
        &mut *(0xE000ED0C as *mut Volatile<u32>)
    };
    aircr.write(0x05FA0004);
    unreachable!();
}
