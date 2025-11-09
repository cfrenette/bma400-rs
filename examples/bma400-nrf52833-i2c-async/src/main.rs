#![no_std]
#![no_main]

use bma400::{BMA400, I2CInterface, InterruptPins, OutputDataRate, PowerMode};
use defmt::{error, info};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_nrf::{
    bind_interrupts,
    gpio::{Input, Pull},
    peripherals::{self, TWISPI0},
    twim::{self, Frequency, Twim},
};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
use embassy_time::Timer;
use panic_probe as _;
use static_cell::ConstStaticCell;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Initializing I2C Bus...");
    // Statically allocate memory for the I2C Bus
    static RAM_BUFFER: ConstStaticCell<[u8; 16]> = ConstStaticCell::new([0; 16]);
    bind_interrupts!(struct Irqs {
        TWISPI0 => twim::InterruptHandler<peripherals::TWISPI0>;
    });
    let p = embassy_nrf::init(Default::default());
    let mut config = twim::Config::default();
    config.frequency = Frequency::K400;
    let i2c = Twim::new(p.TWISPI0, Irqs, p.P1_00, p.P0_26, config, RAM_BUFFER.take());
    info!("Done!");

    info!("Initializing the BMA400...");
    let mut accel = Mutex::<CriticalSectionRawMutex, _>::new(BMA400::new_i2c(i2c).await.unwrap());
    info!("Done");

    // BMA400: Set the power mode to normal and the output data rate to 200Hz
    accel
        .get_mut()
        .config_accel()
        .with_power_mode(PowerMode::Normal)
        .with_odr(OutputDataRate::Hz200)
        .write()
        .await
        .unwrap();

    // BMA400: Map the tap interrupt to the INT1 pin
    accel
        .get_mut()
        .config_int_pins()
        .with_tap(InterruptPins::Int1)
        .write()
        .await
        .unwrap();

    // BMA400: Enable the single and double tap interrupts and set the interrupt mode
    // to latching (persist until cleared by reading the interrupt status register)
    accel
        .get_mut()
        .config_interrupts()
        .with_latch_int(true)
        .with_d_tap_int(true)
        .with_s_tap_int(true)
        .write()
        .await
        .unwrap();

    let tap_interrupt_pin = Input::new(p.P0_10, Pull::None);
    let tap_handler = handle_tap(&accel, tap_interrupt_pin);
    let sample_collector = sample_acceleration(&accel);
    // Join the acceleration sampler task with the tap interrupt handler
    join(tap_handler, sample_collector).await;
}

async fn handle_tap(
    shared_accel: &Mutex<CriticalSectionRawMutex, BMA400<I2CInterface<Twim<'static, TWISPI0>>>>,
    mut tap_pin: Input<'_>,
) {
    tap_pin.wait_for_high().await;
    loop {
        tap_pin.wait_for_high().await;
        // Wait for the device to be free
        let mut accel = shared_accel.lock().await;
        // Read the interrupt status register (clearing the interrupt) and determine the tap type
        match accel.get_int_status1().await {
            Ok(status) => {
                if status.d_tap_stat() {
                    info!("Double tap detected!");
                } else if status.s_tap_stat() {
                    info!("Single tap detected!");
                }
            }
            Err(_) => error!("An error occurred retrieving the interrupt status"),
        }
        tap_pin.wait_for_low().await;
    }
}

async fn sample_acceleration(
    shared_accel: &Mutex<CriticalSectionRawMutex, BMA400<I2CInterface<Twim<'static, TWISPI0>>>>,
) {
    loop {
        Timer::after_millis(2000).await;
        // Wait for the device to be free
        let mut accel = shared_accel.lock().await;
        match accel.get_unscaled_data().await {
            Err(_) => error!("An error occurred reading data!"),
            Ok(m) => info!("Acceleration: x: {}, y: {}, z: {}", m.x, m.y, m.z),
        }
    }
}
