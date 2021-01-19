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

use f3::hal::stm32f30x::{self, gpioc, rcc, GPIOE, RCC, TIM6, tim6};

fn init() -> (GPIOE, TIM6) {
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

    // Use the RCC peripheral to enable timer 6.
    dp.RCC.apb1enr.modify(|_, w| {
        w.tim6en().set_bit()
    });

    // Set up common parameters for timer 6
    // 8 MHz / (psc+1) = 1 kHz is the goal, thus 7999.
    dp.TIM6.psc.write(|w| {
        w.psc().bits(7999)
    });

    // OPM = one pulse mode, CEN = counter enable, and we want to keep it disabled for now.
    dp.TIM6.cr1.write(|w| w.opm().set_bit().cen().clear_bit());

    // Return an owned GPIOE struct.  I think this also means that the rest of the device peripherals are inaccessible.
    (dp.GPIOE, dp.TIM6)
}

fn my_delay(timeinfo: &TIM6, ms: u16) {
    timeinfo.arr.write(|w| w.arr().bits(ms)); // Set the auto reload value.
    timeinfo.cr1.modify(|_, w| w.cen().set_bit()); // Enable timer.
    while !timeinfo.sr.read().uif().bit_is_set() {} // Wait until update
    timeinfo.sr.modify(|_, w| w.uif().clear_bit());  // Clear the flag for next time.
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
    let (gpioe, mytim6) = init(); // Returns an owned GPIOE.


    // gpioe.odr.write(|w| {
    //     w.odr8().set_bit();
    //     w.odr9().set_bit()
    // });

    // Third approach using the HAL crate.  Taken from the f3 repo examples/blinky.rs.
    // But it doesn't compile, as the various FLASH, RCC, GPIOE structs are from the peripheral access crate and don't have
    // constrain or split methods.
    // let dp = stm32f30x::Peripherals::take().unwrap(); // Device peripherals
    // let mut flash = dp.FLASH.constrain();
    // let mut rcc = dp.RCC.constrain();
    // let mut gpioe = dp.GPIOE.split(&mut rcc.ahb);


    iprintln!(stim, "Hello, big world!");

//    let mut x = 0;
    let ms = 500;
    loop {
        // your code goes here
        // x += 1;
        // iprintln!(stim, "x is {}", x);
        
        gpioe.odr.write(|w| {
            w.odr8().set_bit();
            w.odr9().set_bit()
        });

        my_delay(&mytim6, ms);

        gpioe.odr.write(|w| {
            w.odr8().clear_bit();
            w.odr9().clear_bit()
        });

        my_delay(&mytim6, ms);

    }
}
