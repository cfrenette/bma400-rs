//! A platform-agnostic driver for the BMA400 accelerometer implemented using [`embedded-hal`] traits.
//!
//! [`embedded-hal`]: https://crates.io/crates/embedded-hal
//!
//! # Basic Usage
//! I²C - `cargo add bma400 --features=i2c-default`
//! ```
//! // Import an embedded hal implementation
//! use embedded_hal_mock::eh1::i2c::{Mock, Transaction}; // replace as appropriate w/ hal crate for your MCU
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
//! // i2c implements embedded-hal i2c::WriteRead and i2c::Write
//! let mut accelerometer = BMA400::new_i2c(&mut i2c).unwrap();
//! # i2c.done();
//! ```
//! SPI - `cargo add bma400 --features=spi`
//! ```
//! // Import an embedded hal implementation
//! use bma400::{PowerMode, Scale, BMA400};
//! use embedded_hal_mock::eh1::{
//!     pin::{Mock as MockPin, State, Transaction as PinTransaction},
//!     spi::{Mock, Transaction},
//! }; // replace as appropriate w/ hal crate for your MCU
//! # let expected_io = vec![
//! #   Transaction::transfer(vec![0x80, 0x00], vec![0x00,0x00]),
//! #   Transaction::transfer_in_place(vec![0x00], vec![0x00]),
//! #   Transaction::transfer(vec![0x80, 0x00], vec![0x00, 0x00]),
//! #   Transaction::transfer_in_place(vec![0x00], vec![0x90]),
//! # ];
//! # let expected_pin = vec![
//! #   PinTransaction::set(State::Low),
//! #   PinTransaction::set(State::High),
//! #   PinTransaction::set(State::Low),
//! #   PinTransaction::set(State::High),
//! # ];
//! # let mut spi = Mock::new(&expected_io);
//! # let mut csb_pin = MockPin::new(&expected_pin);
//! // spi implements embedded-hal spi::Transfer and spi::Write
//! // csb_pin implements embedded-hal digital::v2::OutputPin
//! let mut accelerometer = BMA400::new_spi(&mut spi, &mut csb_pin).unwrap();
//! # spi.done();
//! # csb_pin.done();
//! ```
//!
//! From here it's the same API for both:
//! ```
//! # use embedded_hal_mock::eh1::{
//! #     spi::{Mock, Transaction},
//! #     pin::{Mock as MockPin, State, Transaction as PinTransaction},
//! # };
//! # use bma400::{
//! #     BMA400,
//! #     PowerMode,
//! #     Scale,
//! # };
//! # let expected_io = vec![
//! #   Transaction::transfer(vec![0x80, 0x00], vec![0x00,0x00]),
//! #   Transaction::transfer_in_place(vec![0x00], vec![0x00]),
//! #   Transaction::transfer(vec![0x80, 0x00], vec![0x00, 0x00]),
//! #   Transaction::transfer_in_place(vec![0x00], vec![0x90]),
//! #   Transaction::write_vec(vec![0x19, 0x02]),
//! #   Transaction::write_vec(vec![0x1A, 0x09]),
//! #   Transaction::transfer(vec![0x84, 0x00], vec![0x00, 0x00]),
//! #   Transaction::transfer_in_place(
//! #       vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
//! #       vec![0x1E, 0x00, 0x10, 0x00, 0xDC, 0x03],
//! #   )];
//! # let expected_pin = vec![
//! #   PinTransaction::set(State::Low),
//! #   PinTransaction::set(State::High),
//! #   PinTransaction::set(State::Low),
//! #   PinTransaction::set(State::High),
//! #   PinTransaction::set(State::Low),
//! #   PinTransaction::set(State::High),
//! #   PinTransaction::set(State::Low),
//! #   PinTransaction::set(State::High),
//! #   PinTransaction::set(State::Low),
//! #   PinTransaction::set(State::High),
//! # ];
//! # let mut spi = Mock::new(&expected_io);
//! # let mut csb_pin = MockPin::new(&expected_pin);
//! # let mut accelerometer = BMA400::new_spi(&mut spi, &mut csb_pin).unwrap();
//! // The accelerometer is in sleep mode at power on
//! // Let's wake it up and set the scale to 2g
//! accelerometer
//!     .config_accel()
//!     .with_power_mode(PowerMode::Normal)
//!     .with_scale(Scale::Range2G)
//!     .write()
//!     .unwrap();
//! // Read a single measurment
//! if let Ok(measurement) = accelerometer.get_data() {
//!     assert_eq!(30, measurement.x);
//!     assert_eq!(16, measurement.y);
//!     assert_eq!(988, measurement.z);
//! }
//! # spi.done();
//! # csb_pin.done();
//! ```
//! # Features
//! BMA400 can currently be compiled with the following feature flags:
//! - i2c-default: Use I²C with the default address `0b00010100` with SDO pin pulled to GND
//! - i2c-alt: Use I²C with the alternate address `0b00010101` with SDO pin pulled to VDDIO[^address]
//! - spi: Use SPI
//! - float: Enable functions returning floating point values. Currently just `get_temp_celsius()`
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

#![warn(missing_docs, unsafe_code)]
#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]

pub(crate) use embedded_hal as hal;

#[cfg(all(feature = "async", any(feature = "spi", feature = "i2c")))]
pub(crate) use embedded_hal_async as hal_async;

use hal::delay::DelayUs;
pub mod types;
pub use types::*;
pub(crate) mod registers;
use registers::*;
mod interface;
use interface::{ReadFromRegister, WriteToRegister};
mod config;
pub use config::ActChgConfigBuilder;
use config::Config;
pub use config::GenIntConfigBuilder;
pub use config::OrientChgConfigBuilder;
pub use config::TapConfigBuilder;
pub use config::{
    AccConfigBuilder, AutoLpConfigBuilder, AutoWakeupConfigBuilder, FifoConfigBuilder,
    IntConfigBuilder, IntPinConfigBuilder, WakeupIntConfigBuilder,
};

#[cfg(any(docsrs, feature = "i2c", test))]
#[cfg_attr(docsrs, doc(cfg(feature = "i2c")))]
mod i2c;
#[cfg(any(docsrs, feature = "i2c", test))]
#[cfg_attr(docsrs, doc(cfg(feature = "i2c")))]
pub use i2c::I2CInterface;

#[cfg(any(docsrs, feature = "spi", test))]
#[cfg_attr(docsrs, doc(cfg(feature = "spi")))]
mod spi;
#[cfg(any(docsrs, feature = "spi", test))]
#[cfg_attr(docsrs, doc(cfg(feature = "spi")))]
pub use spi::SPIInterface;

#[cfg(any(docsrs, feature = "async"))]
#[cfg_attr(docsrs, doc(cfg(feature = "async")))]
mod asynch;
#[cfg(any(docsrs, feature = "async"))]
#[cfg_attr(docsrs, doc(cfg(feature = "async")))]
pub use asynch::*;

/// A BMA400 device
pub struct BMA400<T> {
    interface: T,
    config: Config,
}

impl<T, InterfaceError, PinError> BMA400<T>
where
    T: ReadFromRegister<Error = BMA400Error<InterfaceError, PinError>>
        + WriteToRegister<Error = BMA400Error<InterfaceError, PinError>>,
{
    /// Returns the chip ID (0x90)
    ///
    /// # Examples
    /// ```
    /// # use embedded_hal_mock::eh1::i2c::{Mock, Transaction};
    /// # use bma400::BMA400;
    /// # let ADDR = 0b10100;
    /// # let expected = vec![
    /// #        Transaction::write_read(ADDR, vec![0x00], vec![0x90]),
    /// #        Transaction::write_read(ADDR, vec![0x00], vec![0x90]),
    /// #    ];
    /// # let mut i2c = Mock::new(&expected);
    /// # let mut bma400 = BMA400::new_i2c(&mut i2c).unwrap();
    /// let id = bma400.get_id().unwrap();
    /// assert_eq!(0x90, id);
    /// # i2c.done();
    /// ```
    pub fn get_id(&mut self) -> Result<u8, BMA400Error<InterfaceError, PinError>> {
        let mut id = [0];
        self.interface.read_register(ChipId, &mut id)?;
        Ok(id[0])
    }

    /// Reads and returns the status of the command error register
    ///
    /// Errors are cleared on read
    ///
    /// # Examples
    /// ```
    /// # use embedded_hal_mock::eh1::i2c::{Mock, Transaction};
    /// # use bma400::BMA400;
    /// # let ADDR = 0b10100;
    /// # let expected = vec![
    /// #        Transaction::write_read(ADDR, vec![0x00], vec![0x90]),
    /// #        Transaction::write_read(ADDR, vec![0x02], vec![0x02]),
    /// #        Transaction::write_read(ADDR, vec![0x02], vec![0x00]),
    /// #    ];
    /// # let mut i2c = Mock::new(&expected);
    /// # let mut bma400 = BMA400::new_i2c(&mut i2c).unwrap();
    /// // There was an error processing the previous command:
    /// let err = bma400.get_cmd_error().unwrap();
    /// assert!(err);
    /// // Reading the register cleared it:
    /// let err = bma400.get_cmd_error().unwrap();
    /// assert!(!err);
    /// # i2c.done();
    /// ```
    pub fn get_cmd_error(&mut self) -> Result<bool, BMA400Error<InterfaceError, PinError>> {
        let mut err_byte = [0];
        self.interface.read_register(ErrReg, &mut err_byte)?;
        Ok(err_byte[0] & 0b00000010 != 0)
    }

    /// Reads and returns the sensor [Status] register
    ///
    /// # Examples
    /// ```
    /// # use embedded_hal_mock::eh1::i2c::{Mock, Transaction};
    /// # use bma400::{BMA400, PowerMode};
    /// # let ADDR = 0b10100;
    /// # let expected = vec![
    /// #        Transaction::write_read(ADDR, vec![0x00], vec![0x90]),
    /// #        Transaction::write_read(ADDR, vec![0x03], vec![0x00]),
    /// #    ];
    /// # let mut i2c = Mock::new(&expected);
    /// # let mut bma400 = BMA400::new_i2c(&mut i2c).unwrap();
    /// // Retrieve the statuses from the status register
    /// let status = bma400.get_status().unwrap();
    /// // The sensor's current power mode
    /// let power_mode = status.power_mode();
    /// assert!(matches!(PowerMode::Sleep, power_mode));
    /// # i2c.done();
    /// ```
    pub fn get_status(&mut self) -> Result<Status, BMA400Error<InterfaceError, PinError>> {
        let mut status_byte = [0];
        self.interface.read_register(StatusReg, &mut status_byte)?;
        Ok(Status::new(status_byte[0]))
    }

    /// Returns a single 3-axis reading as a [Measurement], with no adjustment for the selected [Scale]
    ///
    /// To get scaled data use [`get_data`](BMA400::get_data)
    ///
    /// # Examples
    /// ```
    /// # use embedded_hal_mock::eh1::i2c::{Mock, Transaction};
    /// # use bma400::BMA400;
    /// # let ADDR = 0b10100;
    /// # let expected = vec![
    /// #        Transaction::write_read(ADDR, vec![0x00], vec![0x90]),
    /// #        Transaction::write_read(ADDR, vec![0x04], vec![0x0F, 0x00, 0x08, 0x00, 0xEE, 0x01]),
    /// #    ];
    /// # let mut i2c = Mock::new(&expected);
    /// # let mut bma400 = BMA400::new_i2c(&mut i2c).unwrap();
    /// // Get a single unscaled (raw) measurement reading at the default (4g) scale
    /// let m = bma400.get_unscaled_data().unwrap();
    /// assert_eq!(15, m.x);    // (30 milli-g)
    /// assert_eq!(8, m.y);     // (16 milli-g)
    /// assert_eq!(494, m.z);   // (988 milli-g)
    /// # i2c.done();
    /// ```
    pub fn get_unscaled_data(
        &mut self,
    ) -> Result<Measurement, BMA400Error<InterfaceError, PinError>> {
        let mut bytes = [0u8; 6];
        self.interface.read_register(AccXLSB, &mut bytes)?;
        Ok(Measurement::from_bytes_unscaled(&bytes))
    }

    /// Returns a single 3-axis reading as a [Measurement] adjusted for the selected [Scale]
    ///
    /// To get unscaled data use [`get_unscaled_data()`](BMA400::get_unscaled_data)
    ///
    /// # Examples
    /// ```
    /// # use embedded_hal_mock::eh1::i2c::{Mock, Transaction};
    /// # use bma400::BMA400;
    /// # let ADDR = 0b10100;
    /// # let expected = vec![
    /// #        Transaction::write_read(ADDR, vec![0x00], vec![0x90]),
    /// #        Transaction::write_read(ADDR, vec![0x04], vec![0x0F, 0x00, 0x08, 0x00, 0xEE, 0x01]),
    /// #    ];
    /// # let mut i2c = Mock::new(&expected);
    /// # let mut bma400 = BMA400::new_i2c(&mut i2c).unwrap();
    /// // Get a single scaled measurement reading at the default (4g) scale
    /// let m = bma400.get_data().unwrap();
    /// assert_eq!(30, m.x);    // (30 milli-g)
    /// assert_eq!(16, m.y);    // (16 milli-g)
    /// assert_eq!(988, m.z);   // (988 milli-g)
    /// # i2c.done();
    /// ```
    pub fn get_data(&mut self) -> Result<Measurement, BMA400Error<InterfaceError, PinError>> {
        let mut bytes = [0u8; 6];
        self.interface.read_register(AccXLSB, &mut bytes)?;
        Ok(Measurement::from_bytes_scaled(self.config.scale(), &bytes))
    }

    /// Timer reading from the integrated sensor clock.
    ///
    /// The timer has a resolution of 21 bits stored across 3 bytes.
    /// The lowest 3 bits are always zero (the value is left-justified for compatibility with
    /// 25.6kHz clocks). This timer is inactive in sleep mode. The clock rolls over to zero
    /// after `0xFFFFF8`
    ///
    /// # Examples
    /// ```
    /// # use embedded_hal_mock::eh1::i2c::{Mock, Transaction};
    /// # use bma400::BMA400;
    /// # let ADDR = 0b10100;
    /// # let expected = vec![
    /// #        Transaction::write_read(ADDR, vec![0x00], vec![0x90]),
    /// #        Transaction::write_read(ADDR, vec![0x0A], vec![0x0F, 0x00, 0x08]),
    /// #    ];
    /// # let mut i2c = Mock::new(&expected);
    /// # let mut bma400 = BMA400::new_i2c(&mut i2c).unwrap();
    /// // Get a timer reading
    /// let time = bma400.get_sensor_clock().unwrap();
    /// assert_eq!(524303, time); // (524303*312.5µs)
    /// # i2c.done();
    /// ```
    pub fn get_sensor_clock(&mut self) -> Result<u32, BMA400Error<InterfaceError, PinError>> {
        let mut buffer = [0u8; 3];
        self.interface.read_register(SensorTime0, &mut buffer)?;
        let bytes = [buffer[0], buffer[1], buffer[2], 0];
        Ok(u32::from_le_bytes(bytes))
    }

    /// Returns `true` if a power reset has been detected
    ///
    /// Status is cleared when read
    ///
    /// # Examples
    /// ```
    /// # use embedded_hal_mock::eh1::i2c::{Mock, Transaction};
    /// # use bma400::BMA400;
    /// # let ADDR = 0b10100;
    /// # let expected = vec![
    /// #        Transaction::write_read(ADDR, vec![0x00], vec![0x90]),
    /// #        Transaction::write_read(ADDR, vec![0x0D], vec![0x01]),
    /// #        Transaction::write_read(ADDR, vec![0x0D], vec![0x00]),
    /// #    ];
    /// # let mut i2c = Mock::new(&expected);
    /// # let mut bma400 = BMA400::new_i2c(&mut i2c).unwrap();
    /// // Get the reset status after a reset
    /// let reset = bma400.get_reset_status().unwrap();
    /// assert!(reset);
    /// // Reading the register cleared it
    /// let reset = bma400.get_reset_status().unwrap();
    /// assert!(!reset);
    /// # i2c.done();
    /// ```
    pub fn get_reset_status(&mut self) -> Result<bool, BMA400Error<InterfaceError, PinError>> {
        let mut buffer = [0];
        self.interface.read_register(Event, &mut buffer)?;
        Ok(buffer[0] & 0x01 != 0)
    }

    /// Reads and returns the [IntStatus0] interrupt status register
    ///
    /// - Data Ready Interrupt - [`drdy_stat()`](IntStatus0::drdy_stat)
    /// - FIFO Watermark Interrupt (FIFO watermark surpassed) - [`fwm_stat()`](IntStatus0::fwm_stat)
    /// - FIFO Buffer Full - [`ffull_stat()`](IntStatus0::ffull_stat)
    /// - Interrupt Engine Overrun - [`ieng_overrun_stat()`](IntStatus0::ieng_overrun_stat)
    /// - Generic Interrupt 2 - [`gen2_stat()`](IntStatus0::gen2_stat)
    /// - Generic Interrupt 1 - [`gen1_stat()`](IntStatus0::gen1_stat)
    /// - Orientation Changed - [`orientch_stat()`](IntStatus0::orientch_stat)
    /// - Wakeup Activity Interrupt - [`wkup_stat()`](IntStatus0::wkup_stat)
    ///
    /// # Examples
    /// ```
    /// # use embedded_hal_mock::eh1::i2c::{Mock, Transaction};
    /// # use bma400::BMA400;
    /// # let ADDR = 0b10100;
    /// # let expected = vec![
    /// #        Transaction::write_read(ADDR, vec![0x00], vec![0x90]),
    /// #        Transaction::write_read(ADDR, vec![0x0E], vec![0xE0]),
    /// #    ];
    /// # let mut i2c = Mock::new(&expected);
    /// # let mut bma400 = BMA400::new_i2c(&mut i2c).unwrap();
    /// // Get interrupt status0
    /// let status0 = bma400.get_int_status0().unwrap();
    /// let drdy = status0.drdy_stat();
    /// let ffull = status0.ffull_stat();
    /// let ieng_overrun = status0.ieng_overrun_stat();
    /// // The data ready and fifo full interrupts are triggered:
    /// assert!(drdy);
    /// assert!(ffull);
    /// // The interrupt engine is not overrun
    /// assert!(!ieng_overrun);
    /// # i2c.done();
    /// ```
    pub fn get_int_status0(&mut self) -> Result<IntStatus0, BMA400Error<InterfaceError, PinError>> {
        let mut status_byte = [0];
        self.interface
            .read_register(InterruptStatus0, &mut status_byte)?;
        Ok(IntStatus0::new(status_byte[0]))
    }

    /// Reads and returns the [IntStatus1] interrupt status register
    ///
    /// - Interrupt Engine Overrun - [`ieng_overrun_stat()`](IntStatus0::ieng_overrun_stat)
    /// - Double Tap Interrupt - [`d_tap_stat()`](IntStatus1::d_tap_stat)
    /// - Single Tap Interrupt - [`s_tap_stat()`](IntStatus1::s_tap_stat)
    /// - Step Interrupt - [`step_int_stat()`](IntStatus1::step_int_stat)
    ///
    /// # Examples
    /// ```
    /// # use embedded_hal_mock::eh1::i2c::{Mock, Transaction};
    /// # use bma400::BMA400;
    /// # let ADDR = 0b10100;
    /// # let expected = vec![
    /// #        Transaction::write_read(ADDR, vec![0x00], vec![0x90]),
    /// #        Transaction::write_read(ADDR, vec![0x0F], vec![0x0C]),
    /// #    ];
    /// # let mut i2c = Mock::new(&expected);
    /// # let mut bma400 = BMA400::new_i2c(&mut i2c).unwrap();
    /// // Get interrupt status1
    /// let status1 = bma400.get_int_status1().unwrap();
    /// let d_tap = status1.d_tap_stat();
    /// let s_tap = status1.s_tap_stat();
    /// let ieng_overrun = status1.ieng_overrun_stat();
    /// // The double and single tap interrupts are triggered:
    /// assert!(d_tap);
    /// assert!(s_tap);
    /// // The interrupt engine is not overrun
    /// assert!(!ieng_overrun);
    /// # i2c.done();
    /// ```
    pub fn get_int_status1(&mut self) -> Result<IntStatus1, BMA400Error<InterfaceError, PinError>> {
        let mut status_byte = [0];
        self.interface
            .read_register(InterruptStatus1, &mut status_byte)?;
        Ok(IntStatus1::new(status_byte[0]))
    }

    /// Reads and returns the [IntStatus2] interrupt status register
    ///
    /// - Interrupt Engine Overrun - [`ieng_overrun_stat()`](IntStatus0::ieng_overrun_stat)
    /// - Activity Change Z - [`actch_z_stat()`](IntStatus2::actch_z_stat)
    /// - Activity Change Y - [`actch_y_stat()`](IntStatus2::actch_y_stat)
    /// - Activity Change X - [`actch_x_stat()`](IntStatus2::actch_x_stat)
    ///
    /// # Examples
    /// ```
    /// # use embedded_hal_mock::eh1::i2c::{Mock, Transaction};
    /// # use bma400::BMA400;
    /// # let ADDR = 0b10100;
    /// # let expected = vec![
    /// #        Transaction::write_read(ADDR, vec![0x00], vec![0x90]),
    /// #        Transaction::write_read(ADDR, vec![0x10], vec![0x01]),
    /// #    ];
    /// # let mut i2c = Mock::new(&expected);
    /// # let mut bma400 = BMA400::new_i2c(&mut i2c).unwrap();
    /// // Get interrupt status2
    /// let status2 = bma400.get_int_status2().unwrap();
    /// let actch_z = status2.actch_z_stat();
    /// let actch_x = status2.actch_x_stat();
    /// let ieng_overrun = status2.ieng_overrun_stat();
    /// // Activity change detected in the x direction, interrupts are triggered:
    /// assert!(actch_x);
    /// // No activity change in the z direction, and the interrupt engine is not overrun
    /// assert!(!actch_z);
    /// assert!(!ieng_overrun);
    /// # i2c.done();
    /// ```
    pub fn get_int_status2(&mut self) -> Result<IntStatus2, BMA400Error<InterfaceError, PinError>> {
        let mut status_byte = [0];
        self.interface
            .read_register(InterruptStatus2, &mut status_byte)?;
        Ok(IntStatus2::new(status_byte[0]))
    }

    /// Returns the number of unread bytes currently in the FIFO
    ///
    /// # Examples
    /// ```
    /// # use embedded_hal_mock::eh1::i2c::{Mock, Transaction};
    /// # use bma400::BMA400;
    /// # let ADDR = 0b10100;
    /// # let expected = vec![
    /// #        Transaction::write_read(ADDR, vec![0x00], vec![0x90]),
    /// #        Transaction::write_read(ADDR, vec![0x12], vec![0x00, 0x04]),
    /// #    ];
    /// # let mut i2c = Mock::new(&expected);
    /// # let mut bma400 = BMA400::new_i2c(&mut i2c).unwrap();
    /// // Get the FIFO Buffer length
    /// let bytes = bma400.get_fifo_len().unwrap();
    /// assert_eq!(1024, bytes); // It's full!
    /// # i2c.done();
    /// ```
    pub fn get_fifo_len(&mut self) -> Result<u16, BMA400Error<InterfaceError, PinError>> {
        let mut buffer = [0u8; 2];
        self.interface.read_register(FifoLength0, &mut buffer)?;
        let bytes = [buffer[0], buffer[1] & 0b0000_0111];
        Ok(u16::from_le_bytes(bytes))
    }

    /// Reads enough bytes from the FIFO to fill `buffer` and returns a [FifoFrames] iterator
    /// over the [Frame]s in `buffer`
    ///
    /// # Examples
    /// ```
    /// # use embedded_hal_mock::eh1::i2c::{Mock, Transaction};
    /// # use bma400::{BMA400, FrameType};
    /// # let ADDR = 0b10100;
    /// # let expected = vec![
    /// #        Transaction::write_read(ADDR, vec![0x00], vec![0x90]),
    /// #        Transaction::write_read(ADDR, vec![0x14], vec![
    /// #           0x48, 0x6E,
    /// #           0x9E, 0x01, 0x80, 0x0F, 0xFF, 0x0F, 0x7F,
    /// #           0xA0, 0xF8, 0xFF, 0xFF,
    /// #           0x80, 0x00]),
    /// #    ];
    /// # let mut i2c = Mock::new(&expected);
    /// # let mut bma400 = BMA400::new_i2c(&mut i2c).unwrap();
    /// // Read from the FIFO
    /// let mut buffer = [0u8; 15];
    /// let mut frames = bma400.read_fifo_frames(&mut buffer).unwrap();
    ///
    /// // A Control Frame
    /// if let Some(frame) = frames.next() {
    ///     assert!(matches!(frame.frame_type(), FrameType::Control));
    ///     // This frame says there were changes to the data source, the filter1 bandwidth and ODR/OSR/Scale settings
    ///     assert_eq!(Some(true), frame.fifo_src_chg());
    ///     assert_eq!(Some(true), frame.filt1_bw_chg());
    ///     assert_eq!(Some(true), frame.acc1_chg());
    ///     // This is not a data frame and so has no data
    ///     assert_eq!(None, frame.x());
    /// }
    ///
    /// // A Data Frame
    /// if let Some(frame) = frames.next() {
    ///     assert!(matches!(frame.frame_type(), FrameType::Data));
    ///     // All 3 axes have data
    ///     assert_eq!(Some(-2047), frame.x());
    ///     assert_eq!(Some(-1), frame.y());
    ///     assert_eq!(Some(2047), frame.z());
    /// }
    ///
    /// // A Time Frame
    /// if let Some(frame) = frames.next() {
    ///     assert!(matches!(frame.frame_type(), FrameType::Time));
    ///     assert_eq!(Some(0xFFFFF8), frame.time()); // about to roll over!
    /// }
    ///
    /// // No more Frames
    /// assert_eq!(None, frames.next());
    /// # i2c.done();
    /// ```
    pub fn read_fifo_frames<'a>(
        &mut self,
        buffer: &'a mut [u8],
    ) -> Result<FifoFrames<'a>, BMA400Error<InterfaceError, PinError>> {
        if self.config.is_fifo_read_disabled() {
            return Err(ConfigError::FifoReadWhilePwrDisable.into());
        }
        self.interface.read_register(FifoData, buffer)?;
        Ok(FifoFrames::new(buffer))
    }

    /// Flush all data from the FIFO
    ///
    /// # Examples
    /// ```
    /// # use embedded_hal_mock::eh1::i2c::{Mock, Transaction};
    /// # use bma400::BMA400;
    /// # let ADDR = 0b10100;
    /// # let expected = vec![
    /// #        Transaction::write_read(ADDR, vec![0x00], vec![0x90]),
    /// #        Transaction::write_read(ADDR, vec![0x12], vec![0x00, 0x04]),
    /// #        Transaction::write(ADDR, vec![0x7E, 0xB0]),
    /// #        Transaction::write_read(ADDR, vec![0x12], vec![0x00, 0x00]),
    /// #    ];
    /// # let mut i2c = Mock::new(&expected);
    /// # let mut bma400 = BMA400::new_i2c(&mut i2c).unwrap();
    /// // Get the FIFO Buffer length
    /// let bytes = bma400.get_fifo_len().unwrap();
    /// assert_eq!(1024, bytes); // It's full!
    ///                          // Flush all data from the fifo
    /// bma400.flush_fifo().unwrap();
    /// let bytes = bma400.get_fifo_len().unwrap();
    /// assert_eq!(0, bytes); // It's empty!
    /// # i2c.done();
    /// ```
    pub fn flush_fifo(&mut self) -> Result<(), BMA400Error<InterfaceError, PinError>> {
        self.interface.write_register(Command::FlushFifo)?;
        Ok(())
    }

    /// Get the step count
    ///
    /// The counter only increments if the Step Interrupt is enabled
    ///
    /// # Examples
    /// ```
    /// # use embedded_hal_mock::eh1::i2c::{Mock, Transaction};
    /// # use bma400::BMA400;
    /// # let ADDR = 0b10100;
    /// # let expected = vec![
    /// #        Transaction::write_read(ADDR, vec![0x00], vec![0x90]),
    /// #        Transaction::write_read(ADDR, vec![0x15], vec![0x20, 0x05, 0x08]),
    /// #    ];
    /// # let mut i2c = Mock::new(&expected);
    /// # let mut bma400 = BMA400::new_i2c(&mut i2c).unwrap();
    /// // Get the step count
    /// let num_steps = bma400.get_step_count().unwrap();
    /// assert_eq!(525600, num_steps);
    /// # i2c.done();
    /// ```
    pub fn get_step_count(&mut self) -> Result<u32, BMA400Error<InterfaceError, PinError>> {
        let mut buffer = [0u8; 3];
        self.interface.read_register(StepCount0, &mut buffer)?;
        Ok(u32::from_le_bytes([buffer[0], buffer[1], buffer[2], 0]))
    }

    /// Reset the step count to 0
    ///
    /// # Examples
    /// ```
    /// # use embedded_hal_mock::eh1::i2c::{Mock, Transaction};
    /// # use bma400::BMA400;
    /// # let ADDR = 0b10100;
    /// # let expected = vec![
    /// #        Transaction::write_read(ADDR, vec![0x00], vec![0x90]),
    /// #        Transaction::write_read(ADDR, vec![0x15], vec![0x20, 0x05, 0x08]),
    /// #        Transaction::write(ADDR, vec![0x7E, 0xB1]),
    /// #        Transaction::write_read(ADDR, vec![0x15], vec![0x00, 0x00, 0x00]),
    /// #    ];
    /// # let mut i2c = Mock::new(&expected);
    /// # let mut bma400 = BMA400::new_i2c(&mut i2c).unwrap();
    /// // Get the step count
    /// let num_steps = bma400.get_step_count().unwrap();
    /// assert_eq!(525600, num_steps);
    /// // Clear the counter
    /// bma400.clear_step_count().unwrap();
    /// let num_steps = bma400.get_step_count().unwrap();
    /// assert_eq!(0, num_steps); // empty
    /// # i2c.done();
    /// ```
    pub fn clear_step_count(&mut self) -> Result<(), BMA400Error<InterfaceError, PinError>> {
        self.interface.write_register(Command::ClearStepCount)?;
        Ok(())
    }

    /// Activity Recognition
    ///
    /// # Examples
    /// ```
    /// # use embedded_hal_mock::eh1::i2c::{Mock, Transaction};
    /// # use bma400::{BMA400, Activity};
    /// # let ADDR = 0b10100;
    /// # let expected = vec![
    /// #        Transaction::write_read(ADDR, vec![0x00], vec![0x90]),
    /// #        Transaction::write_read(ADDR, vec![0x18], vec![0x01]),
    /// #        Transaction::write_read(ADDR, vec![0x18], vec![0x02]),
    /// #    ];
    /// # let mut i2c = Mock::new(&expected);
    /// # let mut bma400 = BMA400::new_i2c(&mut i2c).unwrap();
    /// // Walking
    /// let activity = bma400.get_step_activity().unwrap();
    /// assert!(matches!(activity, Activity::Walk));
    /// // Running
    /// let activity = bma400.get_step_activity().unwrap();
    /// assert!(matches!(activity, Activity::Run));
    /// # i2c.done();
    /// ```
    pub fn get_step_activity(&mut self) -> Result<Activity, BMA400Error<InterfaceError, PinError>> {
        let mut buffer = [0];
        self.interface.read_register(StepStatus, &mut buffer)?;
        let activity = match buffer[0] & 0b11 {
            0x00 => Activity::Still,
            0x01 => Activity::Walk,
            _ => Activity::Run,
        };
        Ok(activity)
    }

    /// Chip temperature represented as an i8 with 0.5℃ resolution
    ///
    /// -128 (-40.0℃) to
    /// 127 (87.5℃)
    ///
    /// # Examples
    /// ```
    /// # use embedded_hal_mock::eh1::i2c::{Mock, Transaction};
    /// # use bma400::BMA400;
    /// # let ADDR = 0b10100;
    /// # let expected = vec![
    /// #        Transaction::write_read(ADDR, vec![0x00], vec![0x90]),
    /// #        Transaction::write_read(ADDR, vec![0x11], vec![0xD2]),
    /// #    ];
    /// # let mut i2c = Mock::new(&expected);
    /// # let mut bma400 = BMA400::new_i2c(&mut i2c).unwrap();
    /// // Get the temperature
    /// let temp = bma400.get_raw_temp().unwrap();
    /// assert_eq!(-46, temp); // 0℃
    /// # i2c.done();
    /// ```
    pub fn get_raw_temp(&mut self) -> Result<i8, BMA400Error<InterfaceError, PinError>> {
        let mut temp = [0];
        self.interface.read_register(TempData, &mut temp)?;
        let t = i8::from_le_bytes(temp);
        Ok(t)
    }

    /// Chip temperature in degrees celsius with 0.5℃ resolution
    ///
    /// # Examples
    /// ```
    /// # use embedded_hal_mock::eh1::i2c::{Mock, Transaction};
    /// # use bma400::BMA400;
    /// # let ADDR = 0b10100;
    /// # let expected = vec![
    /// #        Transaction::write_read(ADDR, vec![0x00], vec![0x90]),
    /// #        Transaction::write_read(ADDR, vec![0x11], vec![0xD2]),
    /// #    ];
    /// # let mut i2c = Mock::new(&expected);
    /// # let mut bma400 = BMA400::new_i2c(&mut i2c).unwrap();
    /// // Get the temperature
    /// let temp = bma400.get_temp_celsius().unwrap();
    /// assert_eq!(0f32, temp); // 0℃
    /// # i2c.done();
    /// ```
    #[cfg(feature = "float")]
    pub fn get_temp_celsius(&mut self) -> Result<f32, BMA400Error<InterfaceError, PinError>> {
        Ok(f32::from(self.get_raw_temp()?) * 0.5 + 23.0)
    }

    /// Configure how the accelerometer samples, filters and ouputs data
    ///
    /// - [PowerMode] using [`with_power_mode()`](AccConfigBuilder::with_power_mode)
    /// - [DataSource] for [`get_data()`](BMA400::get_data) and [`get_unscaled_data()`](BMA400::get_unscaled_data) using [`with_reg_dta_src()`](AccConfigBuilder::with_reg_dta_src)
    /// - [OversampleRate] for low power and normal modes using [`with_osr_lp()`](AccConfigBuilder::with_osr_lp) and [`with_osr()`](AccConfigBuilder::with_osr) respectively
    /// - [Filter1Bandwidth] using [`with_filt1_bw()`](AccConfigBuilder::with_filt1_bw)
    /// - [OutputDataRate] using [`with_odr()`](AccConfigBuilder::with_odr)
    /// - [Scale] using [`with_scale()`](AccConfigBuilder::with_scale)
    ///
    /// # Examples
    /// ```
    /// # use embedded_hal_mock::eh1::i2c::{Mock, Transaction};
    /// # use bma400::{BMA400, PowerMode, Scale, OversampleRate};
    /// # let ADDR = 0b10100;
    /// # let expected = vec![
    /// #        Transaction::write_read(ADDR, vec![0x00], vec![0x90]),
    /// #        Transaction::write(ADDR, vec![0x19, 0x62]),
    /// #        Transaction::write(ADDR, vec![0x1A, 0xC9]),
    /// #    ];
    /// # let mut i2c = Mock::new(&expected);
    /// # let mut bma400 = BMA400::new_i2c(&mut i2c).unwrap();
    /// // Set the PowerMode to Normal, Scale to 16g
    /// // and low power oversample rate to OSR3
    /// bma400
    ///     .config_accel()
    ///     .with_power_mode(PowerMode::Normal)
    ///     .with_scale(Scale::Range16G)
    ///     .with_osr_lp(OversampleRate::OSR3)
    ///     .write()
    ///     .unwrap();
    /// # i2c.done();
    /// ```
    pub fn config_accel(&mut self) -> AccConfigBuilder<&mut Self> {
        AccConfigBuilder::new(self)
    }

    /// Enable or disable interrupts[^except] and set interrupt latch mode
    ///
    /// [^except]: To enable the Auto-Wakeup Interrupt see [`config_autowkup()`](BMA400::config_autowkup)
    ///
    /// # Examples
    /// ```
    /// # use embedded_hal_mock::eh1::i2c::{Mock, Transaction};
    /// # use bma400::BMA400;
    /// # let ADDR = 0b10100;
    /// # let expected = vec![
    /// #        Transaction::write_read(ADDR, vec![0x00], vec![0x90]),
    /// #        Transaction::write(ADDR, vec![0x1F, 0x40]),
    /// #        Transaction::write(ADDR, vec![0x20, 0x81]),
    /// #    ];
    /// # let mut i2c = Mock::new(&expected);
    /// # let mut bma400 = BMA400::new_i2c(&mut i2c).unwrap();
    /// // Enable the FIFO Watermark and Step Interrupts
    /// // and enable Interrupt Latching
    /// bma400
    ///     .config_interrupts()
    ///     .with_fwm_int(true)
    ///     .with_step_int(true)
    ///     .with_latch_int(true)
    ///     .write()
    ///     .unwrap();
    /// # i2c.done();
    /// ```
    pub fn config_interrupts(&mut self) -> IntConfigBuilder<&mut Self> {
        IntConfigBuilder::new(self)
    }

    /// Map interrupts to the [InterruptPins::Int1] / [InterruptPins::Int2] hardware interrupt pins
    ///  
    /// - Control the pin electrical behavior using [`with_int1_cfg()`](IntPinConfigBuilder::with_int1_cfg) / [`with_int2_cfg()`](IntPinConfigBuilder::with_int2_cfg)
    ///    - [`PinOutputConfig::PushPull`] High = VDDIO, Low = GND
    ///    - [`PinOutputConfig::OpenDrain`] High = VDDIO, Low = High Impedance
    ///
    /// # Examples
    /// ```
    /// # use embedded_hal_mock::eh1::i2c::{Mock, Transaction};
    /// # use bma400::{BMA400, InterruptPins, PinOutputConfig, PinOutputLevel};
    /// # let ADDR = 0b10100;
    /// # let expected = vec![
    /// #        Transaction::write_read(ADDR, vec![0x00], vec![0x90]),
    /// #        Transaction::write(ADDR, vec![0x21, 0x40]),
    /// #    ];
    /// # let mut i2c = Mock::new(&expected);
    /// # let mut bma400 = BMA400::new_i2c(&mut i2c).unwrap();
    /// // Map the FIFO Watermark interrupt to Int1
    /// // and set the pin to set VDDIO when active
    /// bma400
    ///     .config_int_pins()
    ///     .with_fifo_wm(InterruptPins::Int1)
    ///     .with_int1_cfg(PinOutputConfig::PushPull(PinOutputLevel::ActiveHigh))
    ///     .write()
    ///     .unwrap();
    /// # i2c.done();
    /// ```
    pub fn config_int_pins(&mut self) -> IntPinConfigBuilder<&mut Self> {
        IntPinConfigBuilder::new(self)
    }

    /// Configure the 1024 byte FIFO Buffer Behavior
    ///
    /// - Enable / Disable writing data for axes using [`with_axes()`](FifoConfigBuilder::with_axes)
    /// - Enable / Disable 8 bit mode (truncate the 4 least significant bits) to save space in the buffer using [`with_8bit_mode`](FifoConfigBuilder::with_8bit_mode)
    /// - [DataSource] for the FIFO Buffer using [`with_src()`](FifoConfigBuilder::with_src)
    /// - Enable / Disable sending a clock reading (once) on overreading the buffer using [`with_send_time_on_empty()`](FifoConfigBuilder::with_send_time_on_empty)
    /// - Enable / Disable overwriting oldest frames using [`with_stop_on_full()`](FifoConfigBuilder::with_stop_on_full)
    /// - Enable / Disable automatic flush on power mode change using [`with_auto_flush()`](FifoConfigBuilder::with_auto_flush)
    /// - Set the fill threshold for the FIFO watermark interrupt using [`with_watermark_thresh()`](FifoConfigBuilder::with_watermark_thresh)
    /// - Manually Enable / Disable the FIFO read circuit using [`with_read_disabled()`](FifoConfigBuilder::with_read_disabled)
    ///
    /// # Examples
    /// ```
    /// # use embedded_hal_mock::eh1::i2c::{Mock, Transaction};
    /// # use bma400::BMA400;
    /// # let ADDR = 0b10100;
    /// # let expected = vec![
    /// #        Transaction::write_read(ADDR, vec![0x00], vec![0x90]),
    /// #        Transaction::write(ADDR, vec![0x26, 0xE2]),
    /// #        Transaction::write(ADDR, vec![0x27, 0x20]),
    /// #        Transaction::write(ADDR, vec![0x28, 0x03]),
    /// #    ];
    /// # let mut i2c = Mock::new(&expected);
    /// # let mut bma400 = BMA400::new_i2c(&mut i2c).unwrap();
    /// // Enable x, y and z axes, stop on full
    /// // and set the watermark to 800 bytes
    /// bma400
    ///     .config_fifo()
    ///     .with_axes(true, true, true)
    ///     .with_stop_on_full(true)
    ///     .with_watermark_thresh(800)
    ///     .write()
    ///     .unwrap();
    /// # i2c.done();
    /// ```
    pub fn config_fifo(&mut self) -> FifoConfigBuilder<&mut Self> {
        FifoConfigBuilder::new(self)
    }

    /// Configure Auto Low Power settings
    ///
    /// - Set the timeout counter for low power mode using [`with_timeout()`](AutoLpConfigBuilder::with_timeout)
    /// - [AutoLPTimeoutTrigger] (trigger and timer reset condition) using [`with_auto_lp_trigger()`](AutoLpConfigBuilder::with_auto_lp_trigger)
    /// - Set Generic Interrupt 1 as a trigger condition for auto low power using [`with_gen1_int_trigger()`](AutoLpConfigBuilder::with_gen1_int_trigger)
    /// - Set Data Ready as a trigger condition for auto low power using [`with_drdy_trigger()`](AutoLpConfigBuilder::with_drdy_trigger)
    ///
    /// # Examples
    /// ```
    /// # use embedded_hal_mock::eh1::i2c::{Mock, Transaction};
    /// # use bma400::{BMA400, AutoLPTimeoutTrigger};
    /// # let ADDR = 0b10100;
    /// # let expected = vec![
    /// #        Transaction::write_read(ADDR, vec![0x00], vec![0x90]),
    /// #        Transaction::write(ADDR, vec![0x2A, 0x4E]),
    /// #        Transaction::write(ADDR, vec![0x2B, 0x28]),
    /// #    ];
    /// # let mut i2c = Mock::new(&expected);
    /// # let mut bma400 = BMA400::new_i2c(&mut i2c).unwrap();
    /// // Enable auto low power on timeout, reset timeout
    /// // on gen2 interrupt trigger and set the timeout to 500ms
    /// bma400
    ///     .config_auto_lp()
    ///     .with_timeout(1250)
    ///     .with_auto_lp_trigger(AutoLPTimeoutTrigger::TimeoutEnabledGen2IntReset)
    ///     .write()
    ///     .unwrap();
    /// # i2c.done();
    /// ```
    pub fn config_auto_lp(&mut self) -> AutoLpConfigBuilder<&mut Self> {
        AutoLpConfigBuilder::new(self)
    }

    /// Configure Auto Wake-up settings
    ///
    /// - Set the length of time between each wake-up using [`with_wakeup_period()`](AutoWakeupConfigBuilder::with_wakeup_period)
    /// - Enable / Disable periodic wakeup using [`with_periodic_wakeup()`](AutoWakeupConfigBuilder::with_periodic_wakeup)
    /// - Enable / Disable wake-up interrupt using [`with_activity_int()`](AutoWakeupConfigBuilder::with_activity_int)
    ///
    /// # Examples
    /// ```
    /// # use embedded_hal_mock::eh1::i2c::{Mock, Transaction};
    /// # use bma400::BMA400;
    /// # let ADDR = 0b10100;
    /// # let expected = vec![
    /// #        Transaction::write_read(ADDR, vec![0x00], vec![0x90]),
    /// #        Transaction::write(ADDR, vec![0x2C, 0x4E]),
    /// #        Transaction::write(ADDR, vec![0x2D, 0x26]),
    /// #    ];
    /// # let mut i2c = Mock::new(&expected);
    /// # let mut bma400 = BMA400::new_i2c(&mut i2c).unwrap();
    /// // Enable periodic wakeup, auto wakeup on
    /// // activity interrupt trigger and set the
    /// // wakeup period to 500ms
    /// bma400
    ///     .config_autowkup()
    ///     .with_wakeup_period(1250)
    ///     .with_periodic_wakeup(true)
    ///     .with_activity_int(true)
    ///     .write()
    ///     .unwrap();
    /// # i2c.done();
    /// ```
    pub fn config_autowkup(&mut self) -> AutoWakeupConfigBuilder<&mut Self> {
        AutoWakeupConfigBuilder::new(self)
    }

    /// Configure Wake-up Interrupt settings
    ///
    /// - [WakeupIntRefMode] using [`with_ref_mode()`](WakeupIntConfigBuilder::with_ref_mode)
    /// - Set the number of consecutive samples that must satisfy the condition before the interrupt is triggered using [`with_num_samples()`](WakeupIntConfigBuilder::with_num_samples)
    /// - Enable / Disable axes to be evaluated against the condition using [`with_axes()`](WakeupIntConfigBuilder::with_axes)
    /// - Set the interrupt trigger threshold using [`with_threshold()`](WakeupIntConfigBuilder::with_threshold)
    /// - Set the reference acceleration using [`with_ref_accel()`](WakeupIntConfigBuilder::with_ref_accel)
    ///
    /// # Examples
    /// ```
    /// # use embedded_hal_mock::eh1::i2c::{Mock, Transaction};
    /// # use bma400::{BMA400, WakeupIntRefMode};
    /// # let ADDR = 0b10100;
    /// # let expected = vec![
    /// #        Transaction::write_read(ADDR, vec![0x00], vec![0x90]),
    /// #        Transaction::write(ADDR, vec![0x30, 0x20]),
    /// #        Transaction::write(ADDR, vec![0x2F, 0x61]),
    /// #    ];
    /// # let mut i2c = Mock::new(&expected);
    /// # let mut bma400 = BMA400::new_i2c(&mut i2c).unwrap();
    /// // Enable wakeup interrupt for x and y axes w/ a threshold
    /// // of 256 milli-g (at 4g scale) and automatically update the
    /// // reference acceleration once each time the device
    /// // enters low power mode
    /// bma400
    ///     .config_wkup_int()
    ///     .with_ref_mode(WakeupIntRefMode::OneTime)
    ///     .with_threshold(32)
    ///     .with_axes(true, true, false)
    ///     .write()
    ///     .unwrap();
    /// # i2c.done();
    /// ```
    pub fn config_wkup_int(&mut self) -> WakeupIntConfigBuilder<&mut Self> {
        WakeupIntConfigBuilder::new(self)
    }

    /// Configure Orientation Change Interrupt settings
    ///
    /// - Enable / Disable axes evaluated for the interrupt trigger condition using [`with_axes()`](OrientChgConfigBuilder::with_axes)
    /// - [DataSource] used for evaluating the trigger condition [`with_src()`](OrientChgConfigBuilder::with_src)
    /// - Set the [OrientIntRefMode] (reference acceleration update mode) using [`with_ref_mode()`](OrientChgConfigBuilder::with_ref_mode)
    /// - Set the number of samples that a newly detected orientation must be in effect before the interrupt is triggered with [`with_duration()`](OrientChgConfigBuilder::with_duration)
    /// - Manually set the reference acceleration for the interrupt trigger condition using [`with_ref_accel()`](OrientChgConfigBuilder::with_ref_accel)
    ///
    /// # Examples
    /// ```
    /// # use embedded_hal_mock::eh1::i2c::{Mock, Transaction};
    /// # use bma400::{BMA400, OrientIntRefMode};
    /// # let ADDR = 0b10100;
    /// # let expected = vec![
    /// #        Transaction::write_read(ADDR, vec![0x00], vec![0x90]),
    /// #        Transaction::write(ADDR, vec![0x35, 0xE4]),
    /// #        Transaction::write(ADDR, vec![0x36, 0x20]),
    /// #    ];
    /// # let mut i2c = Mock::new(&expected);
    /// # let mut bma400 = BMA400::new_i2c(&mut i2c).unwrap();
    /// // Enable orientation change interrupt all axes, automatically
    /// // update the reference acceleration once each time the device
    /// // enters a new stable orientation with a threshold of 256 milli-g
    /// // (at 4g scale)
    /// bma400
    ///     .config_orientchg_int()
    ///     .with_axes(true, true, true)
    ///     .with_ref_mode(OrientIntRefMode::AccFilt2)
    ///     .with_threshold(32)
    ///     .write()
    ///     .unwrap();
    /// # i2c.done();
    /// ```
    pub fn config_orientchg_int(&mut self) -> OrientChgConfigBuilder<&mut Self> {
        OrientChgConfigBuilder::new(self)
    }

    /// Configure Generic Interrupt 1 settings
    ///
    /// - Enable / Disable axes evaluated for the interrupt trigger condition using [`with_axes()`](GenIntConfigBuilder::with_axes)
    /// - [DataSource] used for evaluating the trigger condition using [`with_src()`](GenIntConfigBuilder::with_src)
    /// - Set the [GenIntRefMode] (reference acceleration update mode) using [`with_ref_mode()`](GenIntConfigBuilder::with_ref_mode)
    /// - Set the [Hysteresis] adjustment amplitude using [`with_hysteresis()`](GenIntConfigBuilder::with_hysteresis)
    /// - Set the [GenIntCriterionMode] (trigger on activity / inactivity) using [`with_criterion_mode()`](GenIntConfigBuilder::with_criterion_mode)
    /// - Set the [GenIntLogicMode] (trigger on any / all axes) using [`with_logic_mode()`](GenIntConfigBuilder::with_logic_mode)
    /// - Set the interrupt trigger threshold using [`with_threshold()`](GenIntConfigBuilder::with_threshold)
    /// - Set the number of cycles that the interrupt condition must be true before the interrupt triggers using [`with_duration()`](GenIntConfigBuilder::with_duration)
    /// - Manually set the reference acceleration for the interrupt trigger condition using [`with_ref_accel()`](GenIntConfigBuilder::with_ref_accel)
    ///
    /// # Examples
    /// ```
    /// # use embedded_hal_mock::eh1::i2c::{Mock, Transaction};
    /// # use bma400::{BMA400, GenIntLogicMode, GenIntCriterionMode};
    /// # let ADDR = 0b10100;
    /// # let expected = vec![
    /// #        Transaction::write_read(ADDR, vec![0x00], vec![0x90]),
    /// #        Transaction::write(ADDR, vec![0x3F, 0xE0]),
    /// #        Transaction::write(ADDR, vec![0x40, 0x01]),
    /// #        Transaction::write(ADDR, vec![0x41, 0x20]),
    /// #        Transaction::write(ADDR, vec![0x48, 0xD4]),
    /// #        Transaction::write(ADDR, vec![0x49, 0x03]),
    /// #    ];
    /// # let mut i2c = Mock::new(&expected);
    /// # let mut bma400 = BMA400::new_i2c(&mut i2c).unwrap();
    /// // Enable Generic Interrupt 1 for all axes, manually set
    /// // reference acceleration, trigger on all axes having
    /// // acceleration within reference +/- 256 milli-g (at 4g scale)
    /// bma400
    ///     .config_gen1_int()
    ///     .with_axes(true, true, true)
    ///     .with_ref_accel(0, 0, 980)
    ///     .with_logic_mode(GenIntLogicMode::And)
    ///     .with_criterion_mode(GenIntCriterionMode::Inactivity)
    ///     .with_threshold(32)
    ///     .write()
    ///     .unwrap();
    /// # i2c.done();
    /// ```
    pub fn config_gen1_int(&mut self) -> GenIntConfigBuilder<&mut Self> {
        GenIntConfigBuilder::new_gen1(self)
    }

    /// Configure Generic Interrupt 2 settings
    ///
    /// - Enable / Disable axes evaluated for the interrupt trigger condition using [`with_axes()`](GenIntConfigBuilder::with_axes)
    /// - [DataSource] used for evaluating the trigger condition using [`with_src()`](GenIntConfigBuilder::with_src)
    /// - Set the [GenIntRefMode] (reference acceleration update mode) using [`with_ref_mode()`](GenIntConfigBuilder::with_ref_mode)
    /// - Set the [Hysteresis] adjustment amplitude using [`with_hysteresis()`](GenIntConfigBuilder::with_hysteresis)
    /// - Set the [GenIntCriterionMode] (trigger on activity / inactivity) using [`with_criterion_mode()`](GenIntConfigBuilder::with_criterion_mode)
    /// - Set the [GenIntLogicMode] (trigger on any / all axes) using [`with_logic_mode()`](GenIntConfigBuilder::with_logic_mode)
    /// - Set the interrupt trigger threshold using [`with_threshold()`](GenIntConfigBuilder::with_threshold)
    /// - Set the number of cycles that the interrupt condition must be true before the interrupt triggers using [`with_duration()`](GenIntConfigBuilder::with_duration)
    /// - Manually set the reference acceleration for the interrupt trigger condition using [`with_ref_accel()`](GenIntConfigBuilder::with_ref_accel)
    ///
    /// # Examples
    /// ```
    /// # use embedded_hal_mock::eh1::i2c::{Mock, Transaction};
    /// # use bma400::{BMA400, GenIntLogicMode, GenIntCriterionMode};
    /// # let ADDR = 0b10100;
    /// # let expected = vec![
    /// #        Transaction::write_read(ADDR, vec![0x00], vec![0x90]),
    /// #        Transaction::write(ADDR, vec![0x4A, 0xE0]),
    /// #        Transaction::write(ADDR, vec![0x4B, 0x02]),
    /// #        Transaction::write(ADDR, vec![0x4C, 0x20]),
    /// #        Transaction::write(ADDR, vec![0x53, 0xD4]),
    /// #        Transaction::write(ADDR, vec![0x54, 0x03]),
    /// #    ];
    /// # let mut i2c = Mock::new(&expected);
    /// # let mut bma400 = BMA400::new_i2c(&mut i2c).unwrap();
    /// // Enable Generic Interrupt 2 for all axes, manually set
    /// // reference acceleration, trigger on any axes having
    /// // acceleration outside reference +/- 256 milli-g (at 4g scale)
    /// bma400
    ///     .config_gen2_int()
    ///     .with_axes(true, true, true)
    ///     .with_ref_accel(0, 0, 980)
    ///     .with_logic_mode(GenIntLogicMode::Or)
    ///     .with_criterion_mode(GenIntCriterionMode::Activity)
    ///     .with_threshold(32)
    ///     .write()
    ///     .unwrap();
    /// # i2c.done();
    /// ```
    pub fn config_gen2_int(&mut self) -> GenIntConfigBuilder<&mut Self> {
        GenIntConfigBuilder::new_gen2(self)
    }

    /// Configure Activity Change Interrupt settings
    ///
    /// - Set the interrupt trigger threshold using [`with_threshold()`](ActChgConfigBuilder::with_threshold)
    /// - Enable / Disable the axes evaluated for the interrupt trigger condition using [`with_axes()`](ActChgConfigBuilder::with_axes)
    /// - [DataSource] used for evaluating the trigger condition using [`with_src()`](ActChgConfigBuilder::with_src)
    /// - [ActChgObsPeriod] (number of samples) using [`with_obs_period()`](ActChgConfigBuilder::with_obs_period)
    ///
    /// # Examples
    /// ```
    /// # use embedded_hal_mock::eh1::i2c::{Mock, Transaction};
    /// # use bma400::{BMA400, ActChgObsPeriod, DataSource};
    /// # let ADDR = 0b10100;
    /// # let expected = vec![
    /// #        Transaction::write_read(ADDR, vec![0x00], vec![0x90]),
    /// #        Transaction::write(ADDR, vec![0x55, 0x20]),
    /// #        Transaction::write(ADDR, vec![0x56, 0xF1]),
    /// #    ];
    /// # let mut i2c = Mock::new(&expected);
    /// # let mut bma400 = BMA400::new_i2c(&mut i2c).unwrap();
    /// // Enable Activity Change Interrupt for all axes observing
    /// // average acceleration over 64 samples. Trigger interrupt
    /// // for axes if more than 256 milli-g (at 4g scale)
    /// // difference from acceleration at the pervious evaluation
    /// bma400
    ///     .config_actchg_int()
    ///     .with_axes(true, true, true)
    ///     .with_src(DataSource::AccFilt2)
    ///     .with_obs_period(ActChgObsPeriod::Samples64)
    ///     .with_threshold(32)
    ///     .write()
    ///     .unwrap();
    ///
    /// # i2c.done();
    /// ```
    pub fn config_actchg_int(&mut self) -> ActChgConfigBuilder<&mut Self> {
        ActChgConfigBuilder::new(self)
    }

    /// Configure Advanced Tap Interrupt Settings
    ///
    /// - Set the axis evaluated for the interrupt trigger condition using [`with_axis()`](TapConfigBuilder::with_axis)
    /// - [TapSensitivity] using [`with_sensitivity()`](TapConfigBuilder::with_sensitivity)
    /// - [MinTapDuration] using [`with_min_duration_btn_taps()`](TapConfigBuilder::with_min_duration_btn_taps)
    /// - [DoubleTapDuration] using [`with_max_double_tap_window()`](TapConfigBuilder::with_max_double_tap_window)
    /// - [MaxTapDuration] using [`with_max_tap_duration()`](TapConfigBuilder::with_max_tap_duration)
    ///
    /// # Examples
    /// ```
    /// # use embedded_hal_mock::eh1::i2c::{Mock, Transaction};
    /// # use bma400::{BMA400, DoubleTapDuration, MinTapDuration, TapSensitivity};
    /// # let ADDR = 0b10100;
    /// # let expected = vec![
    /// #        Transaction::write_read(ADDR, vec![0x00], vec![0x90]),
    /// #        Transaction::write(ADDR, vec![0x58, 0x0E]),
    /// #    ];
    /// # let mut i2c = Mock::new(&expected);
    /// # let mut bma400 = BMA400::new_i2c(&mut i2c).unwrap();
    /// // Set maximum elapsed samples between taps for a double tap
    /// // to 120. Set minimum duration between peaks to be considered
    /// // a separate tap. Set tap sensitivity to most sensitive
    /// bma400
    ///     .config_tap()
    ///     .with_max_double_tap_window(DoubleTapDuration::Samples120)
    ///     .with_min_duration_btn_taps(MinTapDuration::Samples4)
    ///     .with_sensitivity(TapSensitivity::SENS0)
    ///     .write()
    ///     .unwrap();
    /// # i2c.done();
    /// ```
    pub fn config_tap(&mut self) -> TapConfigBuilder<&mut Self> {
        TapConfigBuilder::new(self)
    }

    /// Perform the self test procedure and return [`Ok`] if passed,
    /// [`BMA400Error::SelfTestFailedError`] if failed
    ///
    /// This will disable all interrupts and FIFO write for the duration
    ///
    /// See [p.48 of the datasheet](https://www.bosch-sensortec.com/media/boschsensortec/downloads/datasheets/bst-bma400-ds000.pdf#page=48)
    pub fn perform_self_test(
        &mut self,
        timer: &mut impl DelayUs,
    ) -> Result<(), BMA400Error<InterfaceError, PinError>> {
        // Disable interrupts, set accelerometer test config
        self.config.setup_self_test(&mut self.interface)?;

        // Wait 2ms
        timer.delay_ms(2);

        // Write positive test parameters to SelfTest register
        self.interface
            .write_register(SelfTest::from_bits_truncate(0x07))?;

        // Wait 50ms
        timer.delay_ms(50);

        // Read acceleration and excitation values
        let m_pos = self.get_unscaled_data()?;

        // Write negative test parameters to SelfTest register
        self.interface
            .write_register(SelfTest::from_bits_truncate(0x0F))?;

        // Wait 50ms
        timer.delay_ms(50);

        // Read and store acceleration and excitation values
        let m_neg = self.get_unscaled_data()?;

        // Calculate difference
        let (x, y, z) = (m_pos.x - m_neg.x, m_pos.y - m_neg.y, m_pos.z - m_neg.z);

        // Disable self test
        self.interface.write_register(SelfTest::default())?;

        // Wait 50ms
        timer.delay_ms(50);

        // Re-enable interrupts and previous config
        self.config.cleanup_self_test(&mut self.interface)?;

        // Evaluate results
        if x > 1500 && y > 1200 && z > 250 {
            Ok(())
        } else {
            Err(BMA400Error::SelfTestFailedError)
        }
    }

    /// Returns all settings to default values
    pub fn soft_reset(&mut self) -> Result<(), BMA400Error<InterfaceError, PinError>> {
        self.interface.write_register(Command::SoftReset)?;
        self.config = Config::default();
        let mut buffer = [0];
        // Clear reset detection bit
        self.interface.read_register(Event, &mut buffer)?;
        Ok(())
    }
}

impl<T> BMA400<T> {
    /// Consumes the device instance returning the I²C / SPI Interface
    pub fn destroy(self) -> T {
        self.interface
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        interface::{ReadFromRegister, WriteToRegister},
        registers::{ConfigReg, ReadReg},
        BMA400,
    };
    pub struct NoOpInterface;
    #[derive(Debug)]
    pub struct NoOpError;
    impl ReadFromRegister for NoOpInterface {
        type Error = BMA400Error<NoOpError, ()>;

        fn read_register<T: ReadReg>(
            &mut self,
            _register: T,
            _buffer: &mut [u8],
        ) -> Result<(), Self::Error> {
            Ok(())
        }
    }
    impl WriteToRegister for NoOpInterface {
        type Error = BMA400Error<NoOpError, ()>;

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
