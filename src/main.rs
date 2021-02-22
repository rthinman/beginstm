#![no_std]
#![no_main]
//#![deny(unsafe_code)]


// pick a panicking behavior
use panic_halt as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics
// use panic_abort as _; // requires nightly
// use panic_itm as _; // logs messages over ITM; requires ITM support
// use panic_semihosting as _; // logs messages to the host stderr; requires a debugger

#[allow(unused_imports)]
use core::cell::RefCell;
use core::ops::DerefMut;
use core::sync::atomic::{AtomicBool, Ordering};

use cortex_m_rt::entry;
use cortex_m::{iprintln, Peripherals, interrupt::{free, Mutex}};
//use cortex_m_semihosting::{hprintln};

use nb::block;  // Needed for the block! macro.

use stm32f3xx_hal as hal;

use hal::pac;
use hal::prelude::*;
use hal::pwm::tim1;
use hal::timer::{Timer, Event};
use hal::delay::Delay;
use hal::stm32;
use stm32::{interrupt, Interrupt};
//use hal::pac::interrupt; // interrupt available from either pac or stm32.  Requires "rt" feature of the crate.

// Static variables.
static TIM: Mutex<RefCell<Option<Timer<stm32::TIM7>>>> = Mutex::new(RefCell::new(None));
static LED: Mutex<RefCell<Option<hal::gpio::PXx<hal::gpio::Output<hal::gpio::PushPull>>>>> = Mutex::new(RefCell::new(None));
static USER_BUTTON_PRESSED: AtomicBool = AtomicBool::new(false);

#[interrupt]
// Timer toggles the LED.
fn TIM7() {
    free(|cs| {
        if let Some(ref mut tim7) = TIM.borrow(cs).borrow_mut().deref_mut() {
            tim7.clear_update_interrupt_flag()
        }
        if let Some(ref mut led) = LED.borrow(cs).borrow_mut().deref_mut() {
            led.toggle().unwrap()
        }
    });
}

#[interrupt]
// In the stm32f3-discovery board crate, this is abstracted to a button module or crate.
fn EXTI0() {
    // Clear the interrupt request so it won't fire again before another press.
    unsafe { 
        let exti = &(*stm32f3xx_hal::stm32::EXTI::ptr());
        exti.pr1.write(|w| w.pr0().set_bit())
    }
    // PA0 has a low-pass filter, so don't need to debounce in software.
    // Relaxed means only this operation is atomic, no constraints on other operations.
    USER_BUTTON_PRESSED.store(true, Ordering::Relaxed);
}

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

    // Set up timer 7 for an interrupt.
    // Hertz value is the rate of interrupt firing.
    let mut atimer = Timer::tim7(dp.TIM7, 5.hz(), clocks, &mut rcc.apb1);
    atimer.listen(Event::Update);  // Listen for the update event
    // Move the timer into the static Mutex that is accessed by the interrupt.
    free(|cs| {
        TIM.borrow(cs).replace(Some(atimer));
    });

    // Delay struct using the SYSTICK that I can use for blocking delays.
    let mut mydelay = Delay::new(p.SYST, clocks);

    // Configure pins on port A, where the user button is. (For polling.)
    // let mut gpioa = dp.GPIOA.split(&mut rcc.ahb);
    // let user_button = gpioa.pa0.into_floating_input(&mut gpioa.moder, &mut gpioa.pupdr);
    // Alternately, configure PA0 as an external interrupt source.
    dp.EXTI.imr1.modify(|_, w| w.mr0().set_bit()); // External interrupt peripheral, interrupt mask register 1, bit zero for PA0.
    dp.SYSCFG.exticr1.modify(|_, w| unsafe { w.exti0().bits(0x00)}); // Connect PA0 to the EXTI0 interrupt line.
    dp.EXTI.rtsr1.modify(|_, w| w.tr0().set_bit());                  // Set the rising edge trigger for bit0 = PA0.
    // External interrupt is enabled below before the loop.

    // Configure pins on port E, where the board's LEDs are.
    let mut gpioe = dp.GPIOE.split(&mut rcc.ahb);
    // output for TIM1
    // PE9 is the red "North" LED.
    // PE11 is the green "East" LED.
    let pe9 = gpioe.pe9.into_af2(&mut gpioe.moder, &mut gpioe.afrh);
    let pe11 = gpioe.pe11.into_af2(&mut gpioe.moder, &mut gpioe.afrh);
    // regular push-pull output PE13 is the red "South" LED.
    let mut led = gpioe.pe13.into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper).downgrade().downgrade();

    // Configure TIM3, one of the general-purpose timers.
    // This is part of the Timer struct and timer module.
    // It can be used for blocking/nonblocking delays via the nb crate.
    let mut mytim3 = Timer::tim3(dp.TIM3, 1000.hz(), clocks, &mut rcc.apb1);

    // Flash the LED manually to show how to use delays.
    led.set_high().unwrap();
    mydelay.delay_ms(1000u16); // Using the HAL delay struct and SYSTICK.
    led.set_low().unwrap();
    mytim3.start(10.hz()); // 0.1 second delay.  The weird thing is that https://docs.rs/stm32f3xx-hal/0.6.1/stm32f3xx_hal/prelude/trait._embedded_hal_timer_CountDown.html
                           // says it wants a time, but instead it wants a Hertz struct.
    block!(mytim3.wait()).unwrap();  // Block until the timer times out.
    led.set_high().unwrap();
    cortex_m::asm::delay(8_000_000); // Cortex delay for 8M cycles = 1 sec.

    // Now that we have played around with the led, move it into the static so the interrupt can use it.
    // If just doing this and not manually toggling as above, "led" does not need to be defined as mutable.  
    free(|cs| {
        LED.borrow(cs).replace(Some(led));
    });

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

    // Enable interrupts.
    unsafe {
        stm32::NVIC::unmask(Interrupt::TIM7);
        stm32::NVIC::unmask(Interrupt::EXTI0);
    }

    // Loop, flashing the South LED manually if first line is uncommented and led is not moved to LED, or inside the interrupt otherwise.
    loop {
//        led.toggle().unwrap();
        // Check button once per interrupt.  Note that is_high() returns a result.
        // match user_button.is_high() {
        //     Ok(true) => {
        //         iprintln!(stim, "Button pressed");
        //     }
        //     Ok(false) => {
        //         ()
        //     }
        //     Err(_) => {
        //         iprintln!(stim, "Error from button call");
        //     }
        // }
        // Alternate form if we don't want to print anything about the error.
        // if let Ok(true) = user_button.is_high() {
        //     iprintln!(stim, "Button pressed");
        // }

        if USER_BUTTON_PRESSED.swap(false, Ordering::AcqRel) {
            // swap() stores the false and returns the previous value.
            // AcqRel ordering: all writes in other threads are visible before the modification of the swap.
            iprintln!(stim, "Button pressed");
        }

        cortex_m::asm::wfi();     // Wait for interrupt.
    }
}
