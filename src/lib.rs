//! A platform-agnostic driver for the BMA400 accelerometer implemented using [`embedded-hal`] traits.
//! 
//! [`embedded-hal`]: https://github.com/rust-embedded/embedded-hal
//! 
//! # Usage
//! 
//! ```
//! 
//! ```
//! 
//! # Features
//! BMA400 can currently be compiled with the following feature flags:
//! - i2c-default: Use I²C with the default address `0b00010100`
//! - i2c-alt: Use I²C with the alternate address `0b00010101` with SDO pulled to VDDIO[^address]
//! - spi: Use SPI
//! - float: Enable functions returning floating point values. Currently just [get_temp_celsius()](BMA400::get_temp_celsius)
//! 
//! # The Bosch BMA400 Accelerometer
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
//! ### Typical applications
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
pub(crate) use embedded_hal as hal;
use hal::blocking::delay::DelayMs;
pub mod types;
pub use types::*;
pub(crate) mod registers;
use registers::*;
mod interface;
use interface::{
    ReadFromRegister,
    WriteToRegister,
};
mod config;
use config::{
    Config,
};
pub use config::ActChgConfigBuilder;
pub use config::GenIntConfigBuilder;
pub use config::OrientChgConfigBuilder;
pub use config::TapConfigBuilder;
pub use config::{
    AccConfigBuilder,
    AutoLpConfigBuilder,
    AutoWakeupConfigBuilder,
    FifoConfigBuilder,
    IntConfigBuilder,
    IntPinConfigBuilder,
    WakeupIntConfigBuilder,
};

#[cfg(any(feature = "i2c", test))]
mod i2c;
#[cfg(any(feature = "i2c", test))]
pub use i2c::I2CInterface;

#[cfg(any(feature = "spi", test))]
mod spi;
#[cfg(any(feature = "i2c", test))]
pub use spi::SPIInterface;

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
    pub fn get_id(&mut self) -> Result<u8, BMA400Error<InterfaceError, PinError>> {
        let mut id = [0u8; 1];
        self.interface.read_register(ChipId, &mut id)?;
        Ok(id[0])
    }

    /// Reads and returns the status of the command error register
    ///
    /// Errors are cleared on read
    pub fn get_cmd_error(&mut self) -> Result<bool, BMA400Error<InterfaceError, PinError>> {
        let mut err_byte = [0u8; 1];
        self.interface.read_register(ErrReg, &mut err_byte)?;
        Ok(err_byte[0] & 0b00000010 != 0)
    }

    /// Reads and returns the sensor [Status] register
    pub fn get_status(&mut self) -> Result<Status, BMA400Error<InterfaceError, PinError>> {
        let mut status_byte = [0u8; 1];
        self.interface.read_register(StatusReg, &mut status_byte)?;
        Ok(Status::new(status_byte[0]))
    }

    /// Returns a single 3-axis reading as a [Measurement], with no adjustment for the selected [Scale]
    /// 
    /// To get scaled data use [`get_data`](BMA400::get_data)
    pub fn get_unscaled_data(&mut self) -> Result<Measurement, BMA400Error<InterfaceError, PinError>> {
        let mut bytes = [0u8; 6];
        self.interface.read_register(AccXLSB, &mut bytes)?;
        Ok(Measurement::from_bytes_unscaled(&bytes))
    }

    /// Returns a single 3-axis reading as a [Measurement] adjusted for the selected [Scale]
    ///
    /// To get unscaled data use [`get_unscaled_data()`](BMA400::get_unscaled_data)
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
    pub fn get_sensor_clock(&mut self) -> Result<u32, BMA400Error<InterfaceError, PinError>> {
        let mut buffer = [0u8; 3];
        self.interface.read_register(SensorTime0, &mut buffer)?;
        let bytes = [buffer[0], buffer[1], buffer[2], 0];
        Ok(u32::from_le_bytes(bytes))
    }

    /// Returns `true` if a power reset has been detected
    pub fn get_reset_status(&mut self) -> Result<bool, BMA400Error<InterfaceError, PinError>> {
        let mut buffer = [0u8; 1];
        self.interface.read_register(Event, &mut buffer)?;
        Ok(buffer[0] & 0x01 != 0)
    }

    /// Reads and returns the [IntStatus0] interrupt status register
    pub fn get_int_status0(&mut self) -> Result<IntStatus0, BMA400Error<InterfaceError, PinError>> {
        let mut status_byte = [0u8; 1];
        self.interface.read_register(InterruptStatus0, &mut status_byte)?;
        Ok(IntStatus0::new(status_byte[0]))
    }

    /// Reads and returns the [IntStatus1] interrupt status register
    pub fn get_int_status1(&mut self) -> Result<IntStatus1, BMA400Error<InterfaceError, PinError>> {
        let mut status_byte = [0u8; 1];
        self.interface.read_register(InterruptStatus1, &mut status_byte)?;
        Ok(IntStatus1::new(status_byte[0]))
    }

    /// Reads and returns the [IntStatus2] interrupt status register
    pub fn get_int_status2(&mut self) -> Result<IntStatus2, BMA400Error<InterfaceError, PinError>> {
        let mut status_byte = [0u8; 1];
        self.interface.read_register(InterruptStatus2, &mut status_byte)?;
        Ok(IntStatus2::new(status_byte[0]))
    }

    /// Returns the number of unread bytes currently in the FIFO
    pub fn get_fifo_len(&mut self) -> Result<u16, BMA400Error<InterfaceError, PinError>> {
        let mut buffer = [0u8; 2];
        self.interface.read_register(FifoLength0, &mut buffer)?;
        let bytes = [buffer[0], buffer[1] & 0b0000_0111];
        Ok(u16::from_le_bytes(bytes))
    }

    /// Reads enough bytes from the FIFO to fill`buffer`and returns a [FifoFrames] iterator over the Frames
    pub fn read_fifo_frames<'a>(&mut self, buffer: &'a mut [u8]) -> Result<FifoFrames<'a>, BMA400Error<InterfaceError, PinError>> {
        if self.config.is_fifo_read_disabled() {
            return Err(ConfigError::FifoReadWhilePwrDisable.into());
        }
        self.interface.read_register(FifoData, buffer)?;
        Ok(FifoFrames::new(buffer))
    }

    /// Flush all data from the FIFO
    pub fn flush_fifo(&mut self) -> Result<(), BMA400Error<InterfaceError, PinError>> {
        self.interface.write_register(Command::FlushFifo)?;
        Ok(())
    }

    /// Get the step count
    ///
    /// (the counter only increments if the step interrupt is enabled)
    pub fn get_step_count(&mut self) -> Result<u32, BMA400Error<InterfaceError, PinError>> {
        let mut buffer = [0u8; 3];
        self.interface.read_register(StepCount0, &mut buffer)?;
        Ok(u32::from_le_bytes([buffer[0], buffer[1], buffer[2], 0]))
    }

    /// Reset the step count to 0
    pub fn clear_step_count(&mut self) -> Result<(), BMA400Error<InterfaceError, PinError>> {
        self.interface.write_register(Command::ClearStepCount)?;
        Ok(())
    }

    /// Chip temperature represented as an i8 with 0.5℃ resolution
    ///
    /// -128 (-40.0℃) to
    /// 127 (87.5℃)
    pub fn get_raw_temp(&mut self) -> Result<i8, BMA400Error<InterfaceError, PinError>> {
        let mut temp = [0u8; 1];
        self.interface.read_register(TempData, &mut temp)?;
        let t = i8::from_le_bytes(temp);
        Ok(t)
    }

    #[cfg(feature = "float")]
    /// Chip temperature in degrees celsius with 0.5℃ resolution
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
    pub fn config_accel(&mut self) -> AccConfigBuilder<T> {
        AccConfigBuilder::new(self)
    }

    /// Enable or disable interrupts[^except] and set interrupt latch mode
    /// 
    /// [^except]: To enable the Auto-Wakeup Interrupt see [`config_autowkup()`](BMA400::config_autowkup)
    pub fn config_interrupts(&mut self) -> IntConfigBuilder<T> {
        IntConfigBuilder::new(self)
    }

    /// Map interrupts to the [InterruptPins::Int1] / [InterruptPins::Int2] hardware interrupt pins
    ///  
    /// - Control the pin electrical behavior using [`with_int1_cfg()`](IntPinConfigBuilder::with_int1_cfg) / [`with_int2_cfg()`](IntPinConfigBuilder::with_int2_cfg)
    ///    - [`PinOutputConfig::PushPull`] High = VDDIO, Low = GND
    ///    - [`PinOutputConfig::OpenDrain`] High = VDDIO, Low = High Impedance
    pub fn config_int_pins(&mut self) -> IntPinConfigBuilder<T> {
        IntPinConfigBuilder::new(self)
    }

    /// Configure the 1024 byte FIFO Buffer Behavior
    /// 
    /// - Enable / Disable writing data for axes using [`with_axes()`](FifoConfigBuilder::with_axes)
    /// - Enable / Disable 8 bit mode (truncate the 4 least significant bits) to save space in the buffer
    /// - [DataSource] for the FIFO Buffer using [`with_src()`](FifoConfigBuilder::with_src)
    /// - Enable / Disable sending a clock reading (once) on overreading the buffer using [`with_send_time_on_empty()`](FifoConfigBuilder::with_send_time_on_empty)
    /// - Enable / Disable overwriting oldest frames using [`with_stop_on_full()`](FifoConfigBuilder::with_stop_on_full)
    /// - Enable / Disable automatic flush on power mode change using [`with_auto_flush()`](FifoConfigBuilder::with_auto_flush)
    /// - Set the fill threshold for the FIFO watermark interrupt using [`with_watermark_thresh()`](FifoConfigBuilder::with_watermark_thresh)
    /// - Manually Enable / Disable the FIFO read circuit using [`with_read_disabled()`](FifoConfigBuilder::with_read_disabled)
    pub fn config_fifo(&mut self) -> FifoConfigBuilder<T> {
        FifoConfigBuilder::new(self)
    }

    /// Configure Auto Low Power settings
    /// 
    /// - Set the timeout counter for low power mode using [`with_timeout()`](AutoLpConfigBuilder::with_timeout)
    /// - [AutoLPTimeoutTrigger] (trigger and timer reset condition) using [`with_auto_lp_trigger()`](AutoLpConfigBuilder::with_auto_lp_trigger)
    /// - Set Generic Interrupt 1 as a trigger condition for auto low power using [`with_gen1_int_trigger()`](AutoLpConfigBuilder::with_gen1_int_trigger)
    /// - Set Data Ready as a trigger condition for auto low power using [`with_drdy_trigger()`](AutoLpConfigBuilder::with_drdy_trigger)
    pub fn config_auto_lp(&mut self) -> AutoLpConfigBuilder<T> {
        AutoLpConfigBuilder::new(self)
    }

    /// Configure Auto Wake-up settings
    /// 
    /// - Set the length of time between each wake-up using [`with_wakeup_period()`](AutoWakeupConfigBuilder::with_wakeup_period)
    /// - Enable / Disable periodic wakeup using [`with_periodic_wakeup()`](AutoWakeupConfigBuilder::with_periodic_wakeup)
    /// - Enable / Disable wake-up interrupt using [`with_activity_int()`](AutoWakeupConfigBuilder::with_activity_int)
    pub fn config_autowkup(&mut self) -> AutoWakeupConfigBuilder<T> {
        AutoWakeupConfigBuilder::new(self)
    }

    /// Configure Wake-up Interrupt settings
    /// 
    /// - [WakeupIntRefMode] using [`with_ref_mode()`](WakeupIntConfigBuilder::with_ref_mode)
    /// - Set the number of consecutive samples that must satisfy the condition before the interrupt is triggered using [`with_num_samples()`](WakeupIntConfigBuilder::with_num_samples)
    /// - Enable / Disable axes to be evaluated against the condition using [`with_axes()`](WakeupIntConfigBuilder::with_axes)
    /// - Set the interrupt trigger threshold using [`with_threshold()`](WakeupIntConfigBuilder::with_threshold)
    /// - Set the reference acceleration using [`with_ref_accel()`](WakeupIntConfigBuilder::with_ref_accel)
    pub fn config_wkup_int(&mut self) -> WakeupIntConfigBuilder<T> {
        WakeupIntConfigBuilder::new(self)
    }

    /// Configure Orientation Change Interrupt settings
    /// 
    /// - Enable / Disable axes evaluated for the interrupt trigger condition using [`with_axes()`](OrientChgConfigBuilder::with_axes)
    /// - [DataSource] used for evaluating the trigger condition [`with_src()`](OrientChgConfigBuilder::with_src)
    /// - Set the [OrientIntRefMode] (reference acceleration update mode) using [`with_ref_mode()`](OrientChgConfigBuilder::with_ref_mode)
    /// - Set the number of samples that a newly detected orientation must be in effect before the interrupt is triggered with [`with_duration()`](OrientChgConfigBuilder::with_duration)
    /// - Manually set the reference acceleration for the interrupt trigger condition using [`with_ref_accel()`](OrientChgConfigBuilder::with_ref_accel)
    pub fn config_orientchg_int(&mut self) -> OrientChgConfigBuilder<T> {
        OrientChgConfigBuilder::new(self)
    }

    /// Configure Generic Interrupt 1 settings
    /// 
    /// - Enable / Disable axes evaluated for the interrupt trigger condition using [`with_axes()`](GenIntConfigBuilder::with_axes)
    /// - [DataSource] used for evaluating the trigger condition using [`with_src()`](OrientChgConfigBuilder::with_src)
    /// - Set the [GenIntRefMode] (reference acceleration update mode) using [`with_ref_mode()`](GenIntConfigBuilder::with_ref_mode)
    /// - Set the [Hysteresis] adjustment amplitude using [`with_hysteresis()`](GenIntConfigBuilder::with_hysteresis)
    /// - Set the [GenIntCriterionMode] (trigger on activity / inactivity) using [`with_criterion_mode()`](GenIntConfigBuilder::with_criterion_mode)
    /// - Set the [GenIntLogicMode] (trigger on any / all axes) using [`with_logic_mode()`](GenIntConfigBuilder::with_logic_mode)
    /// - Set the interrupt trigger threshold using [`with_threshold()`](GenIntConfigBuilder::with_threshold)
    /// - Set the number of cycles that the interrupt condition must be true before the interrupt triggers using [`with_duration()`](GenIntConfigBuilder::with_duration)
    /// - Manually set the reference acceleration for the interrupt trigger condition using [`with_ref_accel()`](GenIntConfigBuilder::with_ref_accel)
    pub fn config_gen1_int(&mut self) -> GenIntConfigBuilder<T> {
        GenIntConfigBuilder::new_gen1(self)
    }

    /// Configure Generic Interrupt 2 settings
    /// 
    /// - Enable / Disable axes evaluated for the interrupt trigger condition using [`with_axes()`](GenIntConfigBuilder::with_axes)
    /// - [DataSource] used for evaluating the trigger condition using [`with_src()`](OrientChgConfigBuilder::with_src)
    /// - Set the [GenIntRefMode] (reference acceleration update mode) using [`with_ref_mode()`](GenIntConfigBuilder::with_ref_mode)
    /// - Set the [Hysteresis] adjustment amplitude using [`with_hysteresis()`](GenIntConfigBuilder::with_hysteresis)
    /// - Set the [GenIntCriterionMode] (trigger on activity / inactivity) using [`with_criterion_mode()`](GenIntConfigBuilder::with_criterion_mode)
    /// - Set the [GenIntLogicMode] (trigger on any / all axes) using [`with_logic_mode()`](GenIntConfigBuilder::with_logic_mode)
    /// - Set the interrupt trigger threshold using [`with_threshold()`](GenIntConfigBuilder::with_threshold)
    /// - Set the number of cycles that the interrupt condition must be true before the interrupt triggers using [`with_duration()`](GenIntConfigBuilder::with_duration)
    /// - Manually set the reference acceleration for the interrupt trigger condition using [`with_ref_accel()`](GenIntConfigBuilder::with_ref_accel)
    pub fn config_gen2_int(&mut self) -> GenIntConfigBuilder<T> {
        GenIntConfigBuilder::new_gen2(self)
    }

    /// Configure Activity Change Interrupt settings
    /// 
    /// - Set the interrupt trigger threshold using [`with_threshold()`](ActChgConfigBuilder::with_threshold)
    /// - Enable / Disable the axes evaluated for the interrupt trigger condition using [`with_axes()`](ActChgConfigBulder::with_axes)
    /// - [DataSource] used for evaluating the trigger condition using [`with_src()`](ActChgConfigBuilder::with_src)
    /// - [ActChgObsPeriod] (number of samples) using [`with_obs_period()`](ActChgConfigBuilder::with_obs_period)
    pub fn config_actchg_int(&mut self) -> ActChgConfigBuilder<T> {
        ActChgConfigBuilder::new(self)
    }

    /// Configure Advanced Tap Interrupt Settings
    /// 
    /// - Set the axis evaluated for the interrupt trigger condition using [`with_axis()`](TapConfigBuilder::with_axis)
    /// - [TapSensitivity] using [`with_sensitivity()`](TapConfigBuilder::with_sensitivity)
    /// - [MinTapDuration] using [`with_min_duration_btn_taps()`](TapConfigBuilder::with_min_duration_btn_taps)
    /// - [DoubleTapDuration] using [`with_max_double_tap_window()`](TapConfigBuilder::with_max_double_tap_window)
    /// - [MaxTapDuration] using [`with_max_tap_duration()`](TapConfigBuilder::with_max_tap_duration)
    pub fn config_tap(&mut self) -> TapConfigBuilder<T> {
        TapConfigBuilder::new(self)
    }

    /// Perform the self test procedure and return [`Ok`] if passed,
    /// [`BMA400Error::SelfTestFailedError`] if failed
    ///
    /// This will disable all interrupts and FIFO write for the duration
    ///
    /// See [p.48 of the datasheet](https://www.bosch-sensortec.com/media/boschsensortec/downloads/datasheets/bst-bma400-ds000.pdf#page=48)
    pub fn perform_self_test<Timer: DelayMs<u8>>(&mut self, timer: &mut Timer) -> Result<(), BMA400Error<InterfaceError, PinError>> {

        // Disable interrupts, set accelerometer test config
        self.config.setup_self_test(&mut self.interface)?;

        // Wait 2ms
        timer.delay_ms(2);

        // Write positive test parameters to SelfTest register
        self.interface.write_register(SelfTest::from_bits_truncate(0x07))?;

        // Wait 50ms
        timer.delay_ms(50);

        // Read acceleration and excitation values
        let m_pos = self.get_unscaled_data()?;

        // Write negative test parameters to SelfTest register
        self.interface.write_register(SelfTest::from_bits_truncate(0x0F))?;

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
        let mut buffer = [0u8; 1];
        // Clear reset detection bit
        self.interface.read_register(Event, &mut buffer)?;
        Ok(())
    }

    /// Consumes the device instance returning the I²C / SPI Interface
    pub fn destroy(self) -> T {
        self.interface
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        interface::{
            ReadFromRegister,
            WriteToRegister,
        },
        registers::{
            ReadReg,
            ConfigReg,
        },
        BMA400,
    };
    pub struct NoOpInterface;
    #[derive(Debug)]
    pub struct NoOpError;
    impl ReadFromRegister for NoOpInterface {
        type Error = BMA400Error<NoOpError, ()>;

        fn read_register<T: ReadReg>(&mut self, _register: T, _buffer: &mut [u8]) -> Result<(), Self::Error> {
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
