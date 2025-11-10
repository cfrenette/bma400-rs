# bma400-rs
A platform-agnostic Rust driver for the BMA400 accelerometer implemented using [`embedded-hal`](https://github.com/japaric/embedded-hal) traits

## Status

- [x] Support Basic Sensor Features
- [x] SPI Support
- [x] Support Programmable (Custom) Interrupts
- [x] Tests
- [x] Documentation
- [ ] More Examples

## Basic Usage
I²C - `cargo add bma400 --features=i2c`
 ``` rust
 // Import an embedded hal implementation
 use linux_embedded_hal::I2cdev; // replace as appropriate w/ hal crate for your MCU
 use bma400::{
     BMA400,
     PowerMode,
     Scale,
 };
 // i2c implements embedded-hal i2c::I2c
 let mut accelerometer = BMA400::new_i2c(i2c).unwrap();
 ```
 SPI - `cargo add bma400 --features=spi`
 ``` rust
 // Import an embedded hal implementation
 use linux_embedded_hal::{
  Spidev, 
}; // replace as appropriate w/ hal crate for your MCU
 use bma400::{
     BMA400,
     PowerMode,
     Scale,
 };
 // spi implements embedded-hal spi::SpiDevice
 let mut accelerometer = BMA400::new_spi(spi).unwrap();
 ```
 
 From here it's the same API for both:
 ``` rust
 // The accelerometer is in sleep mode at power on
 // Let's wake it up and set the scale to 2g
 accelerometer
     .config_accel()
     .with_power_mode(PowerMode::Normal)
     .with_scale(Scale::Range2G)
     .write().unwrap();
 // Read a single measurment
 if let Ok(measurement) = accelerometer.get_data() {
     assert_eq!(30, measurement.x);
     assert_eq!(16, measurement.y);
     assert_eq!(988, measurement.z);
 }
 ```

For a full example using the tap interrupt mapped to a GPIO pin on the nrf52833, see `examples/`.

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
