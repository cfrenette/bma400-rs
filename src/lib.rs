//! A platform-agnostic driver for the BMA400 accelerometer implemented using [`embedded-hal`] traits.
//!
//! [`embedded-hal`]: https://crates.io/crates/embedded-hal
//!
//! # Basic Usage
//! I²C - `cargo add bma400 --features=i2c`
//! ```
//! // Import an embedded hal implementation
//! use embedded_hal_mock::eh1::i2c::{Mock, Transaction};
//! // replace as appropriate w/ hal crate for your MCU
//! use bma400::{
//!     BMA400,
//!     PowerMode,
//!     Scale,
//! };
//! # let ADDR = 0b10100;
//! # let expected = vec![
//! #        Transaction::write_read(ADDR, vec![0x00], vec![0x90]),
//! #    ];
//! # let mut i2c = Mock::new(&expected);
//! // i2c implements embedded-hal i2c::I2c
//! let mut accelerometer = BMA400::new_i2c(&mut i2c).unwrap();
//! # i2c.done();
//! ```
//! SPI - `cargo add bma400 --features=spi`
//! ```
//! // Import an embedded hal implementation
//! use embedded_hal_mock::eh1::spi::{Mock, Transaction};
//! // replace as appropriate w/ hal crate for your MCU
//! use bma400::{
//!     BMA400,
//!     PowerMode,
//!     Scale,
//! };
//! # let expected_io = vec![
//! #   Transaction::transaction_start(),
//! #   Transaction::write_vec(vec![0x80, 0x00]),
//! #   Transaction::read_vec(vec![0x00]),
//! #   Transaction::transaction_end(),
//! #   Transaction::transaction_start(),
//! #   Transaction::write_vec(vec![0x80, 0x00]),
//! #   Transaction::read_vec(vec![0x90]),
//! #   Transaction::transaction_end(),
//! # ];
//! # let mut spi = Mock::new(&expected_io);
//! // spi implements embedded-hal spi::SpiDevice
//! let mut accelerometer = BMA400::new_spi(&mut spi).unwrap();
//! # spi.done();
//! ```
//!
//! From here it's the same API for both:
//! ```
//! # use embedded_hal_mock::eh1::spi::{Mock, Transaction};
//! # use bma400::{
//! #     BMA400,
//! #     PowerMode,
//! #     Scale,
//! # };
//! # let expected_io = vec![
//! #   Transaction::transaction_start(),
//! #   Transaction::write_vec(vec![0x80, 0x00]),
//! #   Transaction::read_vec(vec![0x00]),
//! #   Transaction::transaction_end(),
//! #   Transaction::transaction_start(),
//! #   Transaction::write_vec(vec![0x80, 0x00]),
//! #   Transaction::read_vec(vec![0x90]),
//! #   Transaction::transaction_end(),
//! #   Transaction::transaction_start(),
//! #   Transaction::write_vec(vec![0x19, 0x02]),
//! #   Transaction::transaction_end(),
//! #   Transaction::transaction_start(),
//! #   Transaction::write_vec(vec![0x1A, 0x09]),
//! #   Transaction::transaction_end(),
//! #   Transaction::transaction_start(),
//! #   Transaction::write_vec(vec![0x84, 0x00]),
//! #   Transaction::read_vec(
//! #       vec![0x1E, 0x00, 0x10, 0x00, 0xDC, 0x03],
//! #   ),
//! #   Transaction::transaction_end(),
//! # ];
//! # let mut spi = Mock::new(&expected_io);
//! # let mut accelerometer = BMA400::new_spi(&mut spi).unwrap();
//! // The accelerometer is in sleep mode at power on
//! // Let's wake it up and set the scale to 2g
//! accelerometer
//!     .config_accel()
//!     .with_power_mode(PowerMode::Normal)
//!     .with_scale(Scale::Range2G)
//!     .write().unwrap();
//! // Read a single measurment
//! if let Ok(measurement) = accelerometer.get_data() {
//!     assert_eq!(30, measurement.x);
//!     assert_eq!(16, measurement.y);
//!     assert_eq!(988, measurement.z);
//! }
//! # spi.done();
//! ```
//! # Features
//! BMA400 can currently be compiled with the following feature flags:
//! - i2c: Use I²C
//! - spi: Use SPI
//! - float: Enable functions returning floating point values. Currently just `get_temp_celsius()`
//! - embedded-hal-async: Swaps blocking API for async API implemented using embedded-hal-async
//! traits
//!
//! # The Bosch BMA400 Accelerometer
//! [Datasheet](https://www.bosch-sensortec.com/media/boschsensortec/downloads/datasheets/bst-bma400-ds000.pdf)
//!
//! Basic Description
//! 12 bit, digital, triaxial acceleration sensor with smart on-chip motion and position-triggered interrupt features.
//!
//! Key features
//! - Small Package Size
//!   - LGA package (12 pins), footprint 2mm x 2mm, height 0.95 mm
//! - Ultra-low power
//!   - Low current consumption of data acquisition without compromising on performance (< 14.5 µA with highest performance)
//! - Programmable functionality
//!   - Acceleration ranges ±2g/±4g/±8g/±16g
//!   - Low-pass filter bandwidths = (0.24/0.48)*ODR up to a max. output data read out of 800Hz
//! - On-chip FIFO
//!   - Integrated FIFO on sensor with 1 KB
//! - On-chip interrupt features
//!   - Auto-low power/Auto wakeup
//!   - Activity/In-activity
//!   - Step Counter (overall device current consumption 4µA)
//!   - Activity Recognition (Walking, Running, Standing still)
//!   - Orientation detection
//!   - Tap/double tap
//! - Digital interface
//!   - SPI (4-wire, 3-wire)
//!   - I²C
//!   - 2 interrupt pins
//!   - VDDIO voltage range: 1.2V to 3.6V
//! - RoHS compliant, halogen-free
//!
//! Typical applications
//! - Step Counting with ultra-low current consumption for extensive battery lifetime
//! - Advanced system power management for mobile applications and (smart) watches
//! - Fitness applications / Activity Tracking
//! - Tap / double tap sensing
//! - Drop detection for warranty logging
//! - Window/door measurements for climate control and alarm systems
//! - IoT applications powered by coin cell driven batteries, requiring <1µA and auto-wakeup functionality
//!
//! [^address]: For more info on I²C address select, see: [https://www.bosch-sensortec.com/media/boschsensortec/downloads/datasheets/bst-bma400-ds000.pdf#page=108](https://www.bosch-sensortec.com/media/boschsensortec/downloads/datasheets/bst-bma400-ds000.pdf#page=108)
#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(missing_docs)]
#![warn(missing_docs, unsafe_code)]
#![no_std]
pub(crate) use embedded_hal;
use embedded_hal::delay::DelayNs;
pub mod types;
#[cfg(any(feature = "embedded-hal-async"))]
pub(crate) use embedded_hal_async;
pub use types::*;
#[cfg(any(feature = "embedded-hal-async"))]
mod asynch;
#[cfg(not(feature = "embedded-hal-async"))]
mod blocking;
pub mod config;
use config::Config;
pub(crate) mod registers;

mod private {
    pub trait Sealed {}
    impl<SPI> Sealed for crate::SPIInterface<SPI> {}
    impl<I2C> Sealed for crate::I2CInterface<I2C> {}
}

/// A BMA400 device
pub struct BMA400<T> {
    interface: T,
    config: Config,
}

/// I²C Interface wrapper
// Wrapper class to instantiate BMA400 with an I²C interface
pub struct I2CInterface<I2C> {
    // Suppress Lint: this is used in the trait impl
    #[allow(unused)]
    addr: u8,
    i2c: I2C,
}

impl<I2C> I2CInterface<I2C> {
    /// Consumes the Interface returning the underlying I²C peripheral
    pub fn destroy(self) -> I2C {
        self.i2c
    }
}

/// SPI Interface wrapper
// Wrapper class to instantiate BMA400 with an SPI interface
// (extending the SpiDevice trait to WriteToRegister and ReadFromRegister)
#[derive(Debug)]
pub struct SPIInterface<SPI> {
    spi: SPI,
}

impl<SPI> SPIInterface<SPI> {
    /// Consumes the Interface returning underlying SPI peripheral and the pin
    pub fn destroy(self) -> SPI {
        self.spi
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        BMA400,
        blocking::{ReadFromRegister, WriteToRegister},
        registers::{ConfigReg, ReadReg},
    };
    pub struct NoOpInterface;
    impl private::Sealed for NoOpInterface {}
    #[derive(Debug)]
    pub struct NoOpError;
    impl ReadFromRegister for NoOpInterface {
        type Error = BMA400Error<NoOpError>;

        fn read_register<T: ReadReg>(
            &mut self,
            _register: T,
            _buffer: &mut [u8],
        ) -> Result<(), Self::Error> {
            Ok(())
        }
    }
    impl WriteToRegister for NoOpInterface {
        type Error = BMA400Error<NoOpError>;

        fn write_register<T: ConfigReg>(&mut self, _register: T) -> Result<(), Self::Error> {
            Ok(())
        }
    }
    pub fn get_test_device() -> BMA400<NoOpInterface> {
        BMA400 {
            interface: NoOpInterface,
            config: Config::default(),
        }
    }
}
