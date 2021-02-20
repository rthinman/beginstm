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

use nb::block;  // Needed for the block! macro.

use stm32f3xx_hal as hal;

use hal::pac;
use hal::prelude::*;
use hal::pwm::tim1;
use hal::timer::Timer;
use hal::delay::Delay;
//use hal::timer::Timer::tim3;

// The following imports are in the PWM example, but are not necessary.
//use hal::hal::PwmPin;
//use hal::flash::FlashExt;
//use hal::gpio::GpioExt;
//use hal::rcc::RccExt;
//use hal::time::U32Ext;


#[entry]
fn main() -> ! {
    // Set up ITM monitoring support.  To print to the console, use the iprintln!(stim, "...") or iprint!(stim, "...") macros.
    // See the "itm.rs" example.
    let mut p = Peripherals::take().unwrap();  // Cortex core peripherals
    let stim = &mut p.ITM.stim[0];

    // Get peripherals
    let dp = pac::Peripherals::take().unwrap(); // Device peripherals
    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(8.mhz()).freeze(&mut flash.acr); // Configure clocks and freeze them.  Need 8 MHz for ITM?.

    // Delay struct using the SYSTICK that I can use for blocking delays.
    let mut mydelay = Delay::new(p.SYST, clocks);

    // Configure pins
    // We aren't using the port A or B pins in this example.
    let mut gpioa = dp.GPIOA.split(&mut rcc.ahb);
//    let pa4 = gpioa.pa4.into_af2(&mut gpioa.moder, &mut gpioa.afrl);
    let _pa6 = gpioa.pa6.into_af2(&mut gpioa.moder, &mut gpioa.afrl);
//    let pa7 = gpioa.pa7.into_af2(&mut gpioa.moder, &mut gpioa.afrl);

    let mut gpiob = dp.GPIOB.split(&mut rcc.ahb);
    let _pb0 = gpiob.pb0.into_af2(&mut gpiob.moder, &mut gpiob.afrl);
//    let pb1 = gpiob.pb1.into_af2(&mut gpiob.moder, &mut gpiob.afrl);
    let _pb4 = gpiob.pb4.into_af2(&mut gpiob.moder, &mut gpiob.afrl);

    let mut gpioe = dp.GPIOE.split(&mut rcc.ahb);
    // output for TIM1
    // PE9 is the red "North" LED.
    // PE11 is the green "East" LED.
 //   let pe8 = gpioe.pe8.into_af2(&mut gpioe.moder, &mut gpioe.afrh);
    let pe9 = gpioe.pe9.into_af2(&mut gpioe.moder, &mut gpioe.afrh);
    let pe11 = gpioe.pe11.into_af2(&mut gpioe.moder, &mut gpioe.afrh);
    // regular push-pull output PE13 is the red "South" LED.
    let mut led = gpioe.pe13.into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);

    led.set_low().unwrap();

    // Configure TIM3, one of the general-purpose timers.
    // This is part of the Timer struct and timer module.
    // It can be used for blocking/nonblocking delays via the nb crate.
    let mut mytim3 = Timer::tim3(dp.TIM3, 1000.hz(), clocks, &mut rcc.apb1);


    // Configure TIM1, an advanced timer with complementary pins that drive 
    // the LEDs on the discovery board.  Unfortunately, presently one can use only regular
    // or complementary pins with the HAL, not both at the same time.
    // Flash the LEDs autonomously.
    // From the hal pwm module.
    let tim1_channels = tim1(
        dp.TIM1,
        1280,    // resolution
        1.hz(),  // Frequency
        &clocks, // To get clock frequencies
    );

    let mut tim1_ch1 = tim1_channels.0.output_to_pe9(pe9); // Can't stack .output_to_pe8(pe8) on it, due to the complementary issue.
    tim1_ch1.set_duty(tim1_ch1.get_max_duty() / 2); // 50% duty
    tim1_ch1.enable();
    let mut tim1_ch2 = tim1_channels.1.output_to_pe11(pe11);
    tim1_ch2.set_duty(tim1_ch2.get_max_duty() / 5); // 20% duty
    tim1_ch2.enable();

    iprintln!(stim, "Hello, big world!");

    // Loop, flashing the South LED manually.
    loop {
        led.toggle().unwrap();
//        cortex_m::asm::delay(8_000_000);
        mydelay.delay_ms(1000u16);
        // Toggle by hand
        // Uses `StatefulOutputPin` instead of `ToggleableOutputPin`.
        if led.is_set_low().unwrap() {
            led.set_high().unwrap();
        } else {
            led.set_low().unwrap();
        }
        // Three different ways of delaying.
//        cortex_m::asm::delay(8_000_000);
//        mydelay.delay_ms(1000u16);
        mytim3.start(10.hz()); // 0.1 second delay.  The weird thing is that https://docs.rs/stm32f3xx-hal/0.6.1/stm32f3xx_hal/prelude/trait._embedded_hal_timer_CountDown.html
                               // says it wants a time, but instead it wants a Hertz struct.
        block!(mytim3.wait()).unwrap();  // Block until the timer times out.
    }
}
