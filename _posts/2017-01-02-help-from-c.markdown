---
layout: post
title:  "Part 0: Introduction, and Getting to Blinky With Some C"
date:   2017-01-02 09:34:28 -0800
categories: teensy
---

Welcome! This series of blog posts will walk you through building an
embedded application in pure Rust. We will use existing C setup code,
as well as the manufacturer's documentation, to understand how the
hardware works.

For this project, I'll be using the
[Teensy](https://www.pjrc.com/teensy/) hardware. Teensy is a fairly
inexpensive ARM Cortex-M development board. My primary board for the
blog series will be the Teensy 3.2, but I will also cover topics
appropriate to the Teensy 3.5 and 3.6 as I encounter them. I do not
have access to Teensy 3.0 hardware, but will endeavor to call out
things that might affect it as I encounter them.

Our primary C reference will be the
[Teensyduino](https://www.pjrc.com/teensy/teensyduino.html)
sources. It is highly recommended that you install the Arduino IDE and
Teensyduino, as you'll need to access files from it if you want to
follow along. The documentation for the chips used on each Teensy
board is [available on the Teensy
site](https://www.pjrc.com/teensy/datasheets.html)

I'll be developing this tutorial series on Arch Linux, and any paths
and package names that I list may reflect that. You might need to go
hunting for what a particular package is called, or where a particular
file lives on your system.

Without further ado, let's begin.

# Prerequisites

To build a Rust program that targets the Teensy, we'll need to make a
couple of changes from a normal Rust environment. I'll be assuming
that you use Rustup to manage your Rust toolchain.

First, we'll install a tool called
[xargo](https://github.com/japaric/xargo). Xargo is a wrapper around
Cargo that provides some extra smarts when cross-compiling.

{% highlight shell %}
$ cargo install xargo
{% endhighlight %}

Once installed, Xargo is available at `~/.cargo/bin/xargo`. It's not
necessary to put this on your `PATH` for this tutorial series, as
we'll be using a wrapper script to call Xargo.

Next, we'll need to configure Rustup to install the Rust
sources. Xargo uses these sources to build a `libcore` for our target
architecture.

{% highlight shell %}
$ rustup component add rust-src
{% endhighlight %}

We'll also need Rust nightly. This series does not use any
unstable features, but a nightly Rust is required in order to
implement [lang
items](https://doc.rust-lang.org/book/lang-items.html). This makes
nightlies necessary for all embedded work right now.

{% highlight shell %}
$ rustup toolchain install nightly
{% endhighlight %}

Finally, we'll want ARM-targetting versions of GCC and
Binutils. Installing these will vary based on your operating
system. For my particular Linux distribution (Arch), I simply ran:

{% highlight shell %}
$ sudo pacman -S arm-none-eabi-gcc
{% endhighlight %}

# Getting Started

It's time to begin. As always, we'll create our project with Cargo:

{% highlight shell %}
$ cargo new --bin teensy
$ cd teensy
$ rustup override set nightly
{% endhighlight %}

We'll start by cleaning up `main.rs` to be more embedded-friendly.

{% highlight rust %}
#![no_std]
#![no_main]

#[no_mangle]
pub extern fn main() {
}
{% endhighlight %}

What does all of this do? The `no_std` attribute tells Rustc that we
don't want the Rust standard library. We will still have access to
everything in `core` (and, later, `collections`). The rest of the
standard library contains too many things that depend on a real
operating system - for example, we don't have a filesystem in the
embedded world.

The changes to `main` are also for our specific needs in the embedded
world. The `no_main` attribute tells Rustc to skip its normal shim
around `main`, which normally handles things like parsing
arguments. Again, this is something which does not exist in our
emebedded world. Our `main` function itself is now marked `no_mangle`
and `pub extern`. This set of flags amounts to telling the Rust
compiler, "Make our `main` function available to C code as if it were
a C function". You'll see why we do this later.

We haven't done everything we need to in order to create a functional
application yet, but let's try to build it anyway, just to see how
we're doing:

    $ ~/.cargo/bin/xargo build --target thumbv7em-none-eabi
       Compiling teensy v0.1.0 (file:///home/branan/proj/armor/teensy)
    error: language item required, but not found: `panic_fmt`
    
    error: aborting due to previous error
    
    error: Could not compile `teensy`.
    
    To learn more, run the command again with --verbose.

Whoops! This is why we're using a nightly Rust - we need to be able to
implement this language item. Normally the standard library handles
this for us, but we told the compiler we weren't using it. That means
we're responsible. Fortunately, it's easy. Go ahead and add the
following to the bottom of `main.rs`

{% highlight rust %}
#[lang = "panic_fmt"]
#[no_mangle]
pub extern fn rust_begin_panic(_msg: core::fmt::Arguments,
                               _file: &'static str,
                               _line: u32) -> ! {
    loop {};
}
{% endhighlight %}

You'll also need to tell Rustc that we know what we're doing with
these language items. Add one more line to the very top of `main.rs`:

{% highlight rust %}
#![feature(lang_items)]
{% endhighlight %}

In a future post, we'll make the `panic_fmt` function more useful by
sending the error message out on a serial port. For now, this will let
us keep going without getting bogged down in hardware details too
early.

Let's try that compile again:

    $ ~/.cargo/bin/xargo build --target thumbv7em-none-eabi
       Compiling teensy v0.1.0 (file:///home/branan/proj/armor/teensy)
    error: linking with `arm-none-eabi-gcc` failed: exit code: 1
      |
      = note: "arm-none-eabi-gcc" "-L" "/home/branan/.xargo/lib/rustlib/thumbv7em-none-eabi/lib" "/home/branan/proj/armor/teensy/target/thumbv7em-none-eabi/debug/deps/teensy-c2094b26cd6b2de4.0.o" "-o" "/home/branan/proj/armor/teensy/target/thumbv7em-none-eabi/debug/deps/teensy-c2094b26cd6b2de4" "-Wl,--gc-sections" "-nodefaultlibs" "-L" "/home/branan/proj/armor/teensy/target/thumbv7em-none-eabi/debug/deps" "-L" "/home/branan/proj/armor/teensy/target/debug/deps" "-L" "/home/branan/.xargo/lib/rustlib/thumbv7em-none-eabi/lib" "-Wl,-Bstatic" "-Wl,-Bdynamic" "/home/branan/.xargo/lib/rustlib/thumbv7em-none-eabi/lib/libcore-2437b081b4539566.rlib"
      = note: /usr/lib/gcc/arm-none-eabi/6.3.0/../../../../arm-none-eabi/bin/ld: cannot find crt0.o: No such file or directory
    collect2: error: ld returned 1 exit status
    
    
    error: aborting due to previous error
    
    error: Could not compile `teensy`.
    
    To learn more, run the command again with --verbose.

This is a very nasty-looking error, but all it's saying is that the
standard application startup sequence doesn't exist for our
target. This makes sense, since we're in the wonderful world of
embedded development. This is easy to fix, though - we just tell the
linker that we aren't using standard startup files. Create a new file
`.cargo/config` with the following content:

{% highlight toml %}
[target.thumbv7em-none-eabi]
rustflags = [
    "-C", "link-arg=-nostartfiles",
]

[target.thumbv7em-none-eabihf]
rustflags = [
    "-C", "link-arg=-nostartfiles",
]
{% endhighlight %}

The second block for `thumbv7em-none-eabihf` is used for Teensy 3.5 or
3.6, which include floating point support.

If we try to build one more time...

    $ ~/.cargo/bin/xargo build --target thumbv7em-none-eabi
       Compiling teensy v0.1.0 (file:///home/branan/proj/armor/teensy)
        Finished debug [unoptimized + debuginfo] target(s) in 0.7 secs

Success! After all this setup, We've finally managed to create a
binary file with nothing but an empty function in it. In order for
that function to get called we'll need to setup interrupt handling and
do some chip initialization. Doing all of this in Rust will be the
subject of a future post. For now, we can borrow the C code from
Teensyduino to do it.

# Seting up C initialization

It's time to bring in the Teensyduino startup files. Once we have them
in place, we'll set up a couple of Makefiles to build them correctly.

{% highlight shell %}
$ mkdir link
$ mkdir startup
$ cp /usr/share/arduino/hardware/teensy/avr/cores/teensy3/*.ld link/
$ cp /usr/share/arduino/hardware/teensy/avr/cores/teensy3/kinetis.h startup/
$ cp /usr/share/arduino/hardware/teensy/avr/cores/teensy3/mk20dx128.c startup/
{% endhighlight %}

There are two sets of files here. The first are linker scripts. These
define how your program code is laid out in memory. For an embedded
device like the Teensy, this also means indicating which data lives in
flash vs RAM, and what needs to be copied from one to the other at
startup. More on this when we implement startup in Rust. There is one
linker script for each CPU type used by the different Teensy
boards. The second files are the current C initialization code.

To the copied files, we will also add a Makefile for building the
startup code. `$(OUTDIR)`, `$(CPU)`, and `$(FPU)` will be set by our
top-level build scripts.

{% highlight make %}
all:: $(OUTDIR)/libstartup.a

$(OUTDIR)/libstartup.a: $(OUTDIR)/mk20dx128.o
	arm-none-eabi-ar rvs $@ $<

$(OUTDIR)/mk20dx128.o: mk20dx128.c kinetis.h
	arm-none-eabi-gcc -fno-common -ffreestanding -mcpu=cortex-m4 -mthumb -O2 -I. $(CPU) $(FPU) -DF_CPU=48000000 -c $< -o $@
{% endhighlight %}

If you're not familiar with Makefile syntax, the above amounts to
compiling the `mk20dx128.c` file and linking it into a static library
called `startup.a`.

To call this makefile, we'll need to create a cargo [build
script](http://doc.crates.io/build-script.html). This script mostly
just takes environment variables and transforms them for the Makefile
we created above. It also tells Cargo to rebuild our project if the
active linker script changes. We'll be creating a top-level Makefile
to set the required environment variables and the active linker script
next.

This code is full of unwraps. Since most of this build script will be
tossed when we move to a Rust-based initialization, it's not worth the
time to work on serious error handling.

{% highlight rust %}
use std::process::Command;
use std::env;

fn main() {
    let target = env::var("TARGET").unwrap();

    let outdir = env::var("OUT_DIR").unwrap();
    let make_outdir = format!("OUTDIR={}", outdir);

    let cpu = env::var("CPU").unwrap().to_uppercase();
    let make_cpu= format!("CPU=-D__{}__", cpu);

    let fpu : i32 = env::var("FPU").unwrap().parse().unwrap();
    let make_fpu = match fpu {
        1 => format!("FPU=-mfloat-abi=hard -mfpu=fpv4-sp-d16 -fsingle-precision-constant"),
        _ => format!("FPU=")
    };


    let output = Command::new("make")
        .arg("-C").arg("startup")
        .arg(make_outdir)
        .arg(make_cpu)
        .arg(make_fpu)
        .output().unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    if ! output.status.success() {
        panic!("Didn't successfully make!: {}", stderr);
    }
    
    println!("cargo:rustc-link-search=native={}", outdir);
    println!("cargo:rustc-link-lib=static=startup");
    println!("cargo:rerun-if-changed=target/{}/layout.ld", target);
}
{% endhighlight %}

We also need to tell Cargo to use our build script. Update
`Cargo.toml` to include this.

{% highlight toml %}
[package]
name = "teensy"
version = "0.1.0"
authors = ["Branan Riley <me@branan.info>"]
build = "build.rs"
{% endhighlight %}

We're almost there! We now need a Makefile to pass in the right
environment variables to our `build.rs` script depending on which
Teensy model we're targetting. We could do this ourselves on the
command line, but it's easier to switch targets (and harder to make
mistakes!) if we have a bit of a script to do it.

{% highlight make %}
# These are the parameters you'd normally pass on the command line
# Sane defaults are specified here
MODE?=release
BOARD?=teensy32
BIN?=$(shell basename `pwd`)

ifeq ($(BOARD),teensy30)
	TARGET=thumbv7em-none-eabi
	CPU=mk20dx128
	FPU=0
else ifeq ($(BOARD),teensy31)
	TARGET=thumbv7em-none-eabi
	CPU=mk20dx256
	FPU=0
else ifeq ($(BOARD),teensy32)
	TARGET=thumbv7em-none-eabi
	CPU=mk20dx256
	FPU=0
else ifeq ($(BOARD),teensy35)
	TARGET=thumbv7em-none-eabihf
	CPU=mk64fx512
	FPU=1
else ifeq ($(BOARD),teensy36)
	TARGET=thumbv7em-none-eabihf
	CPU=mk66fx1m0
	FPU=1
else
	$(error Unknown board type)
endif

ifeq ($(MODE),debug)
	CARGO_FLAGS=
else ifeq ($(MODE),release)
	CARGO_FLAGS=--release
else
	$(error Unknown build mode)
endif

OUTDIR=target/$(TARGET)/$(MODE)
LAYOUT=target/$(TARGET)/layout.ld
HEX=$(OUTDIR)/$(BIN).hex
ELF=$(OUTDIR)/$(BIN)

all:: $(HEX)

$(HEX): $(ELF)
	arm-none-eabi-objcopy -R .stack -O ihex $(ELF) $(HEX)

$(ELF): $(LAYOUT)
	CPU=$(CPU) FPU=$(FPU) ~/.cargo/bin/xargo build --target=$(TARGET) $(CARGO_FLAGS)

flash: $(HEX)
	teensy-loader-cli -w -mmcu=$(CPU) $(HEX) -v

# This rule will cause us to only update the layout.ld file when its
# md5sum doesn't match the requested one for our chip. This allows
# switching the BOARD target without needing to clean in between and
# without causing full rebuilds unnecessarily.
.PHONY: $(LAYOUT)
$(LAYOUT): link/$(CPU).ld
	@mkdir -p $(OUTDIR)
	@md5sum $< | cmp -s $@.md5 -; if test $$? -ne 0; then md5sum $< > $@.md5; cp $< $@; fi
{% endhighlight %}

There's a lot going on in this Makefile, so I'll walk through it from
the top. The first few lines specify the common options, with
reasonable defaults. You can build in `debug` or `release` mode, tell
the Makefile which cargo binary we want to use (for now, it will be
`teensy`, or whatever your project is called), and select which Teensy
board we want to target. I recommend changing the `BOARD` line to
match whichever model you're using.

The next chunk of code specifies various attributes about each
board. These are passed along to Xargo and our build script to ensure
our code is built and linked to target the correct chip for each board
type. There is similar logic for debug/release build selection, and
then some simple variables that are used within the body of the make
rules.

the `$(ELF)` target is the binary generated by Cargo/Xargo. We pass
along some information for the build script in environment variables
here. This elf file is converted to a simple binary blob in the
`$(HEX)` target. The `flash` target can then send this file over to a
target board.

The `$(LAYOUT)` target has some fancy logic to copy over the correct
layout file for our board if the current one is wrong. This, coupled
with the `cargo:rerun-if-changed=target` line from our build script,
will make Cargo rebuild our project if we change BOARD targets. If we
simply copied the file over every time, Cargo would always think it
had changed and rebuild every time.

There's one more thing before we can test this out - We need to
instruct Cargo to use the linker script we've laid down. We'll do this
by updating `.cargo/config`.

{% highlight toml %}
[target.thumbv7em-none-eabi]
rustflags = [
    "-C", "link-arg=-Ttarget/thumbv7em-none-eabi/layout.ld",
    "-C", "link-arg=-nostartfiles",
]

[target.thumbv7em-none-eabihf]
rustflags = [
    "-C", "link-arg=-Ttarget/thumbv7em-none-eabihf/layout.ld",
    "-C", "link-arg=-nostartfiles",
]
{% endhighlight %}

Phew! That's it for build tooling. Let's move on to verifying our
startup code builds and everything links correctly. Now that we have a
Makefile, let's go ahead and use it:

    $ make
    CPU=mk20dx256 FPU=0 ~/.cargo/bin/xargo build --target=thumbv7em-none-eabi --release
       Compiling teensy v0.1.0 (file:///home/branan/proj/armor/teensy)
    error: failed to run custom build command for `teensy v0.1.0 (file:///home/branan/proj/armor/teensy)`
    process didn't exit successfully: `/home/branan/proj/armor/teensy/target/release/build/teensy-15718513675c47b4/build-script-build` (exit code: 101)
    --- stderr
     thread 'main' panicked at 'Didn't successfully make!: mk20dx128.c:32:39: fatal error: core_pins.h: No such file or directory
     #include "core_pins.h" // testing only
                                           ^
    compilation terminated.
    make[1]: *** [Makefile:7: /home/branan/proj/armor/teensy/target/thumbv7em-none-eabi/release/build/teensy-624a19296ad35474/out/mk20dx128.o] Error 1
    ', build.rs:28
    note: Run with `RUST_BACKTRACE=1` for a backtrace.
    
    make: *** [Makefile:50: target/thumbv7em-none-eabi/release/teensy] Error 101

Hmm. It looks like that file we copied over for startup assumes that
it's in the full arduino environment. Let's go ahead and clean it
up. There are several changes that need to be made in order for it to
work. I'll walk through them all below, but won't show the errors
associated with each one.

# Cleaning up the Startup Code

Let's start with that missing include. As the comment suggests, it's
not used at all. We can remove those lines entirely starting at line 32.

{% highlight C %}
#include "kinetis.h"
// #include "core_pins.h" // testing only
// #include "ser_print.h" // testing only
{% endhighlight %}

The `millis` count used by Arduino is in a different file, but is
written in this one. We can go ahead and change that variable from
`extern` to local at line 139:

{% highlight C %}
volatile uint32_t systick_millis_count;
void systick_default_isr(void)
{
	systick_millis_count++;
}
{% endhighlight %}

The startup code calls out to another function that does all of the
initialiation for the Arduino APIs. We don't need any of that, so
rather than trying to make that code work in our temporary C startup,
we will also comment out that call on line 1097:

{% highlight C %}
	//init_pins();
	__enable_irq();

	// _init_Teensyduino_internal_();
{% endhighlight %}

The arduino support code also includes some initialization for the
chip's real time clock. This is a useful feature that we'll want to
support in the future, but for now it's not needed. We can comment out
the calls to the RTC setup. This code exists in two places right next
to each other, on lines 1109 and 1123

{% highlight C %}
		#if ARDUINO >= 10600
		// rtc_set((uint32_t)&__rtc_localtime);
		#else
		// rtc_set(TIME_T);
		#endif
{% endhighlight %}

We will disable calling out to static constructors. We don't have any
at this point in our project, and never should if we stick to pure
Rust. We can confirm that this is safe once we have an executable
built. This change is at line 1131.

{% highlight C %}
	// __libc_init_array();

	startup_late_hook();
	main();
	while (1) ;
}
{% endhighlight %}

At the bottom of the file are a number of support functions for a
C-style standard library. We don't want or need them, and can go ahead
and comment out the one that will cause us problems. `<sys/stat.h>`
doesn't exist in our embedded world, and removing that header also
requires us to remove the `_fstat` function which relies on it. This
code lives on line 1159.

{% highlight C %}
// #include <sys/stat.h>

// __attribute__((weak)) 
// int _fstat(int fd, struct stat *st)
// {
//     st->st_mode = S_IFCHR;
//     return 0;
// }
{% endhighlight %}

With those modifications, `make` should now execute cleanly and leave
you with `target/thumbv7em-none-eabi/release/teensy.hex`. If you
wanted to, you could send this to your Teensy now with `make
flash`. Before we do that, though, let's make it do something so we
can tell it's working - it will turn on the LED.

# Turning on the Lights - unsafe

For now, we'll do this with unsafe code right in `main`. Once we see
that working, we will create a safe(r) interface for accessing the
LED.

Pins on the Kinetis are grouped into "ports". The pins in these ports
can be configured to interface to internal hardware, such as a serial
port or an ADC, or they can be configured as simple I/O devices, which
is what we want. Our process for setting up the LED will look
something like this:

* Switch the right pin into GPIO (General Purpose I/O) mode
* Set the pin to be output
* Set the pin high
* Jump up and down in excitement when it works

On all Teensys, the LED is connected to Port C, Pin 5. We'll start by
accessing the Pin Control Register (PCR) for that pin, and configuring
it as a GPIO. Looking in kinetis.h, we can see that PORTC_PCR5 has the
address `0x4004B014`. From the chip documentation, we can also find
that bits 8-10 of the PCR are the "Mux". The Mux selects which
internal function a given pin is connected to. Most values are
"pin-specific", meaning that different pins have different
features. But for all pins, value 0b001 indicates GPIO. Let's go
ahead and set that from main.

{% highlight rust %}
#[no_mangle]
pub extern fn main() {
    unsafe {
        let pcr = (0x4004B014 as * mut u32);
        core::ptr::write_volatile(pcr, 1<<8);
    }
}
{% endhighlight %}

This will set all other fields of the control register to 0. In a real
program this could affect the operation of your application, but for
our purposes here it's fine.

Now that the pin is in GPIO mode, we can go ahead and set the pin we
want as an output, and then turn on the LED by setting that pin
high. We will do this with the GPIOC_PDDR and GPIOC_PSOR registers
(Port Data Direction and Port Set Output). These registers function as
bitmasks. Any bit with a 1 in GPIOC_PDDR is an output. Any bit that we
write as a 1 in GPIOC_PSOR will cause that pin to go high.

{% highlight rust %}
#[no_mangle]
pub extern fn main() {
    unsafe {
        let pcr = 0x4004B014 as * mut u32;
        let pddr = 0x400FF094 as *mut u32;
        let psor = 0x400FF084 as *mut u32;
        core::ptr::write_volatile(pcr, 1<<8);
        core::ptr::write_volatile(pddr, 1<<5);
        core::ptr::write_volatile(psor, 1<<5);
    }
}
{% endhighlight %}

To make it blink, we can also use the GPIOC_PCOR register. Any bit
written as a 1 in this register will cause the corresponding pin to be
set low. By placing these volatile writes in a loop we can force a
delay between the pin going high and low. Bingo! We have blinky.

It would also be possible to write to the registers outside of the
loop and include a `noop` instruction to force the compiler to keep
the loop around. This results in longer code, which I didn't want for
this first post.

{% highlight rust %}
#[no_mangle]
pub extern fn main() {
    unsafe {
        let pcr = 0x4004B014 as * mut u32;
        let pddr = 0x400FF094 as *mut u32;
        let psor = 0x400FF084 as *mut u32;
        let pcor = 0x400FF088 as *mut u32;

        core::ptr::write_volatile(pcr, 1<<8);
        core::ptr::write_volatile(pddr, 1<<5);
        loop {
            for _ in 0..10000000 {
                core::ptr::write_volatile(psor, 1<<5);
            }
            for _ in 0..10000000 {
                core::ptr::write_volatile(pcor, 1<<5);
            }
        }
    }
}
{% endhighlight %}

At this point, you can plug in your Teensy, run `make flash` and you
should see a flashing orange light.