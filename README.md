# bma400-rs
A platform-agnostic Rust driver for the BMA400 accelrometer implemented using [`embedded-hal`](https://github.com/japaric/embedded-hal) traits

🚧 Under Development 🚧

## Status

- [x] Support Basic Sensor Features
- [ ] SPI Support
- [ ] Support Programmable (Custom) Interrupts
- [ ] Tests (In-progress)
- [ ] Documentation
- [ ] More Examples

## Usage

Import an embedded_hal implementation for your target and this crate: 

```rust
use nrf52833_hal::{
  (...)
};
use bma400::{
  BMA400, 
  PowerMode, 
  OutputDataRate, 
  InterruptPins
};

(...)

    // Initialize the accelerometer by passing in an interface 
    // implementing the embedded-hal i2c WriteRead and Write traits
    let mut accel = BMA400::<Twim<TWIM0>>::new_i2c(i2c).unwrap();


    // Set the power mode to normal and the output data rate to 200Hz
    accel
    .config_accel()
    .with_power_mode(PowerMode::Normal)
    .with_odr(OutputDataRate::Hz200)
    .write().unwrap();


    // Map the tap interrupt to the INT1 pin
    accel
    .config_int_pins()
    .with_tap(InterruptPins::Both)
    .write().unwrap();


    // Enable the single and double tap interrupts and enable latching
    // (interrupt persists until cleared by reading the interrupt status register)
    accel
    .config_interrupts()
    .with_latch_int(true)
    .with_d_tap_int(true)
    .with_s_tap_int(true)
    .write().unwrap();

    // Read a one-shot measurement from the accelerometer
    let measurement = accel.get_data().unwrap();
    let x_accel: i16 = measurement.x;
    let y_accel: i16 = measurement.y;
    let z_accel: i16 = measurement.z;

    // Read the interrupt status register containing the tap interrupt
    // (clearing all interrupts with statuses in that register)
    let int_stat1 = accel.get_int_status1().unwrap();
    let taps_detected = int_stat1.d_tap_stat() || int_stat1.s_tap_stat();

    (...)

```
For a full example using the tap interrupt mapped to a GPIO pin on the nrf52833, see `examples/bma400-nrf52833`.

## About the Sensor 

 (from the manufacturer)

#### Basic Description
12 bit, digital, triaxial acceleration sensor with smart on-chip motion and position-triggered interrupt features.

#### Key features
- Small Package Size 
  - LGA package (12 pins), footprint 2mm x 2mm, height 0.95 mm
- Ultra-low power
  - Low current consumption of data acquisition without compromising on performance (< 14.5 µA with highest performance)
- Programmable functionality
  - Acceleration ranges ±2g/±4g/±8g/±16g
  - Low-pass filter bandwidths = (0.24/0.48)*ODR up to a max. output data read out of 800Hz
- On-chip FIFO
  - Integrated FIFO on sensor with 1 KB
- On-chip interrupt features
  - Auto-low power/Auto wakeup
  - Activity/In-activity
  - Step Counter (overall device current consumption 4µA)
  - Activity Recognition (Walking, Running, Standing still)
  - Orientation detection
  - Tap/double tap
- Digital interface
  - SPI (4-wire, 3-wire)
  - I²C, 
  - 2 interrupt pins
- VDDIO voltage range: 1.2V to 3.6V
- RoHS compliant, halogen-free

#### Typical applications
- Step Counting with ultra-low current consumption for extensive battery lifetime
- Advanced system power management for mobile applications and (smart) watches
- Fitness applications / Activity Tracking
- Tap / double tap sensing
- Drop detection for warranty logging
- Window/door measurements for climate control and alarm systems
- IoT applications powered by coin cell driven batteries, requiring <1µA and auto-wakeup
functionality

## License
Licensed under your choice of either:
- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT) 

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
licensed as above, without any additional terms or conditions.