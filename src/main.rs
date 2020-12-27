#![no_std]
#![no_main]
#![deny(unsafe_code)]


// pick a panicking behavior
use panic_halt as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics
// use panic_abort as _; // requires nightly
// use panic_itm as _; // logs messages over ITM; requires ITM support
// use panic_semihosting as _; // logs messages to the host stderr; requires a debugger

#[allow(unused_imports)]
use cortex_m_rt::entry;
use cortex_m::{iprintln, Peripherals};
//use cortex_m_semihosting::{hprintln};

#[entry]
fn main() -> ! {
    // Set up ITM monitoring support.  To print to the console, use the iprintln!(stim, "...") or iprint!(stim, "...") macros.
    let mut p = Peripherals::take().unwrap();
    let stim = &mut p.ITM.stim[0];

    iprintln!(stim, "Hello, big world!");
//    hprintln!("Hello, whacky world!").unwrap(); // This if we are using semihosting.

    let mut x = 0;
    loop {
        // your code goes here
        x += 1;
        iprintln!(stim, "x is {}", x);
    }
}
