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

use f3::hal::stm32f30x::{self, gpioc, rcc, GPIOE, RCC};

fn init() -> GPIOE {
    let dp = stm32f30x::Peripherals::take().unwrap(); // Device peripherals

    // Use the Reset Clock Control peripheral to enable port E (bit 21 on AHBENR).
    dp.RCC.ahbenr.modify(|_, w| {
        w.iopeen().set_bit()
    });

    // Make thees port E pins outputs.
    dp.GPIOE.moder.modify(|_, w| {
        w.moder8().output();
        w.moder9().output()
    });

    // Return an owned GPIOE struct.  I think this also means that the rest of the device peripherals are inaccessible.
    dp.GPIOE
}

#[entry]
fn main() -> ! {
    // Set up ITM monitoring support.  To print to the console, use the iprintln!(stim, "...") or iprint!(stim, "...") macros.
    // See the "itm.rs" example.
    let mut p = Peripherals::take().unwrap();  // Cortex core peripherals
    let stim = &mut p.ITM.stim[0];

    // This approach works, where the device peripheral is owned by main().
    // Commented out to try to write an initialization function.
    // let dp = stm32f30x::Peripherals::take().unwrap(); // Device peripherals
    // dp.RCC.ahbenr.modify(|_, w| {w.iopeen().set_bit()}); // Turn on bus clock for GPIOE peripheral
    // let gpioe = dp.GPIOE;

    // rcc.ahbenr.modify(|_, w| {
    //     w.iopeen().set_bit()
    // });

    // // Make pins outputs
    // gpioe.moder.modify(|_, w| {
    //     w.moder8().output();
    //     w.moder9().output()
    // });

    // Alternate approach with initialization.
    let gpioe = init(); // Returns an owned GPIOE.


    gpioe.odr.write(|w| {
        w.odr8().set_bit();
        w.odr9().set_bit()
    });

//    let rcc = unsafe {&*RCC::ptr()};
//    rcc.ahbenr.modify(|_, w| {w.iopeen().set_bit()});
    // let gpioe = unsafe { &*GPIOE::ptr()};

    iprintln!(stim, "Hello, big world!");
    // unsafe {
    //     const GPIO_BSRR: u32 = 0x48001018;
    //     *(GPIO_BSRR as *mut u32) = 1 << 25;
    // }
//    hprintln!("Hello, whacky world!").unwrap(); // This if we are using semihosting.

    let mut x = 0;
    loop {
        // your code goes here
        x += 1;
        iprintln!(stim, "x is {}", x);
    }
}
