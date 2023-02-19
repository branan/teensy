use core;

#[derive(Clone,Copy)]
pub enum Clock {
    PortC,
}

#[repr(C,packed)]
pub struct Sim {
    sopt1: u32,
    sopt1_cfg: u32,
    _pad0: [u32; 1023],
    sopt2: u32,
    _pad1: u32,
    sopt4: u32,
    sopt5: u32,
    _pad2: u32,
    sopt7: u32,
    _pad3: [u32; 2],
    sdid: u32,
    _pad4: [u32; 3],
    scgc4: u32,
    scgc5: u32,
    scgc6: u32,
    scgc7: u32,
    clkdiv1: u32,
    clkviv2: u32,
    fcfg1: u32,
    fcfg2: u32,
    uidh: u32,
    uidmh: u32,
    uidml: u32,
    uidl: u32
}

impl Sim {
    pub unsafe fn new() -> &'static mut Sim {
        &mut *(0x40047000 as *mut Sim)
    }

    pub fn enable_clock(&mut self, clock: Clock) {
        unsafe {
            match clock {
                Clock::PortC => {
                    let scgc_ptr = core::ptr::addr_of_mut!(self.scgc5);
                    let mut scgc = scgc_ptr.read_volatile();
                    scgc |= 0x00000800;
                    scgc_ptr.write_volatile(scgc);
                }
            }
        }
    }
}
