#![no_std]
#![no_main]

use bma400_nrf52833 as _; // global logger + panicking behavior + memory layout

use core::cell::RefCell;
use cortex_m::interrupt::Mutex;
use cortex_m::prelude::_embedded_hal_blocking_delay_DelayMs;
use cortex_m_rt::entry;
use nrf52833_hal::{
    gpio,
    gpiote::Gpiote,
    pac::{self, interrupt, TWIM0},
    twim::{Frequency, Pins, Twim}, Timer,
};
use bma400::{BMA400, PowerMode, OutputDataRate, i2c::I2CInterface, InterruptPins};

// Shared access to the accelerometer and GPIO Tasks and Events peripheral
static GPIO: Mutex<RefCell<Option<Gpiote>>> = Mutex::new(RefCell::new(None));
static ACCEL: Mutex<RefCell<Option<BMA400<I2CInterface<Twim<TWIM0>/* lol TODO: newtype? */>>>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    // Gain access to the peripherals
    let peripherals = nrf52833_hal::pac::Peripherals::take().unwrap();
    let p0 = gpio::p0::Parts::new(peripherals.P0);
    let p1 = gpio::p1::Parts::new(peripherals.P1);
    let i2c_pins = Pins {
        scl: p0.p0_26.into_floating_input().degrade(),
        sda: p1.p1_00.into_floating_input().degrade(),
    };
    // Initialize the GPIO I2C interface
    let i2c = Twim::new(peripherals.TWIM0, i2c_pins, Frequency::K400);

    // BMA400: Initialize the Accelerometer
    let mut accel = BMA400::new_i2c(i2c).unwrap();

    // BMA400: Set the power mode to normal and the output data rate to 200Hz
    accel
    .config_accel()
    .with_power_mode(PowerMode::Normal)
    .with_odr(OutputDataRate::Hz200)
    .write().unwrap();


    // BMA400: Map the tap interrupt to the INT1 pin
    accel
    .config_int_pins()
    .with_tap(InterruptPins::Both)
    .write().unwrap();


    // BMA400: Enable the single and double tap interrupts and set the interrupt mode
    // to latching (persist until cleared by reading the interrupt status register)
    accel
    .config_interrupts()
    .with_latch_int(true)
    .with_d_tap_int(true)
    .with_s_tap_int(true)
    .write().unwrap();


    // Set up the hardware interrupt for the pin connected to INT1
    let gpiote = Gpiote::new(peripherals.GPIOTE);
    let channel0 = gpiote.channel0();
    channel0.input_pin(&p0.p0_10.into_floating_input().degrade())
    .lo_to_hi().enable_interrupt();
    channel0.reset_events();

    cortex_m::interrupt::free(move |cs| {
        unsafe {
            pac::NVIC::unmask(pac::Interrupt::GPIOTE);
        }
        pac::NVIC::unpend(pac::Interrupt::GPIOTE);
        *GPIO.borrow(cs).borrow_mut() = Some(gpiote);
        *ACCEL.borrow(cs).borrow_mut() = Some(accel);
        defmt::info!("Now Sensing Taps...");
    });

    let mut timer = Timer::new(peripherals.TIMER0).into_periodic();
    loop {
        timer.delay_ms(2000u32);
        // Borrow mutably in an interrupt free context so we don't get interrupted while
        // the mutex is locked 
        cortex_m::interrupt::free(move |cs| {
            if let Some(bma400) = ACCEL.borrow(cs).borrow_mut().as_mut() {
                
                // BMA400: Read a one-shot measurement from the accelerometer
                match bma400.get_unscaled_data() {
                    Err(_) => defmt::error!("An error occurred reading data!"),
                    Ok(m) => defmt::info!("Acceleration: x: {}, y: {}, z: {}", m.x, m.y, m.z),
                }
            }
        });
    }
    // Convenience function if we don't have a non-returning loop
    //bma400_nrf52833::exit() 
}

// Tap Interrupt Handler
#[interrupt]
fn GPIOTE() {
    // Process in an interrupt-free context so we don't immediately re-enter
    cortex_m::interrupt::free(|cs| {
        if let Some(gpiote) = GPIO.borrow(cs).borrow().as_ref() {
            let tap = gpiote.channel0().is_event_triggered();
            if tap {
                if let Some(bma400) = ACCEL.borrow(cs).borrow_mut().as_mut() {
                    
                    
                    // BMA400: Read the status register
                    match bma400.get_int_status1() {
                        Err(_) => defmt::error!("An error occurred retrieving interrupt status"),
                        Ok(status) => {
                            if status.d_tap_stat() {
                                // Double tap interrupt status is set
                                defmt::info!("Double tap detected!");
                            } else if status.s_tap_stat() {
                                // Single tap interrupt status is set
                                defmt::info!("Single tap detected!");
                            }
                        }
                    }
                }
            }
            // Clear the event
            gpiote.channel0().reset_events();
        }
    });
}