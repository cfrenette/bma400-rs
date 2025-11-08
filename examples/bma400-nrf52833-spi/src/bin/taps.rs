#![no_std]
#![no_main]

use bma400_nrf52833_spi as _; // global logger + panicking behavior + memory layout

use bma400::{BMA400, InterruptPins, OutputDataRate, PowerMode, SPIInterface};
use core::cell::RefCell;
use cortex_m::interrupt::Mutex;
use cortex_m::prelude::_embedded_hal_blocking_delay_DelayMs;
use cortex_m_rt::entry;
use embedded_hal_bus::spi::ExclusiveDevice;
use nrf52833_hal::{
    Delay, Spim, Timer, delay,
    gpio::{self, Level, Output, Pin, PushPull},
    gpiote::Gpiote,
    pac::{self, SPIM0, interrupt},
    spim::{Frequency, Mode, Phase, Pins, Polarity},
};
type SpiDevice = ExclusiveDevice<Spim<SPIM0>, Pin<Output<PushPull>>, Delay>;

// Shared access to the accelerometer and GPIO Tasks and Events peripheral
static GPIO: Mutex<RefCell<Option<Gpiote>>> = Mutex::new(RefCell::new(None));
type AccelDevice = BMA400<SPIInterface<SpiDevice>>;
static ACCEL: Mutex<RefCell<Option<AccelDevice>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    // Gain access to the peripherals
    let peripherals = nrf52833_hal::pac::Peripherals::take().unwrap();
    let p0 = gpio::p0::Parts::new(peripherals.P0);
    let p1 = gpio::p1::Parts::new(peripherals.P1);
    let core_peripherals = pac::CorePeripherals::take().unwrap();
    let syst = core_peripherals.SYST;

    // Initialize the chip select pin to high
    let cs = p1.p1_02.into_push_pull_output(Level::High).degrade();
    let spi_pins = Pins {
        miso: Some(p0.p0_01.into_floating_input().degrade()),
        mosi: Some(p0.p0_13.into_push_pull_output(Level::Low).degrade()),
        sck: Some(p0.p0_17.into_push_pull_output(Level::Low).degrade()),
    };

    // Initialize the GPIO SPI Bus
    let bus = Spim::new(
        peripherals.SPIM0,
        spi_pins,
        Frequency::M8,
        Mode {
            polarity: Polarity::IdleLow,
            phase: Phase::CaptureOnFirstTransition,
        },
        0,
    );

    // Obtain an SPIDevice instance from the bus with no sharing (CS pin always set to high)
    let spi_bus = ExclusiveDevice::new(bus, cs, delay::Delay::new(syst)).unwrap();

    // BMA400: Initialize the Accelerometer
    let mut accel = BMA400::new_spi(spi_bus).unwrap();

    // BMA400: Set the power mode to normal and the output data rate to 200Hz
    accel
        .config_accel()
        .with_power_mode(PowerMode::Normal)
        .with_odr(OutputDataRate::Hz200)
        .write()
        .unwrap();

    // BMA400: Map the tap interrupt to the INT1 pin
    accel
        .config_int_pins()
        .with_tap(InterruptPins::Int1)
        .write()
        .unwrap();

    // BMA400: Enable the single and double tap interrupts and set the interrupt mode
    // to latching (persist until cleared by reading the interrupt status register)
    accel
        .config_interrupts()
        .with_latch_int(true)
        .with_d_tap_int(true)
        .with_s_tap_int(true)
        .write()
        .unwrap();

    // Set up the hardware interrupt for the pin connected to INT1
    let gpiote = Gpiote::new(peripherals.GPIOTE);
    let channel0 = gpiote.channel0();
    channel0
        .input_pin(&p0.p0_10.into_floating_input().degrade())
        .lo_to_hi()
        .enable_interrupt();
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
