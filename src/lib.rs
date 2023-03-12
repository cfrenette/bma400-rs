#![no_std]
use core::fmt::Debug;
pub(crate) use embedded_hal as hal;
use hal::blocking::delay::DelayMs;
pub mod types;
pub use types::*;
pub(crate) mod registers;
use registers::*;
mod interface;
use interface::{ReadFromRegister, WriteToRegister};
pub mod config;
use config::{
    Config,
    AccConfigBuilder,
    IntConfigBuilder,
    IntPinConfigBuilder,
    FifoConfigBuilder,
    AutoLpConfigBuilder,
    AutoWakeupConfigBuilder,
    WakeupIntConfigBuilder,
};
// Maybe #[cfg(feature = "adv-int-orientchg")]
pub use config::OrientChgConfigBuilder;
// Maybe #[cfg(feature = "adv-int-generic")]
pub use config::GenIntConfigBuilder;
// Maybe #[cfg(feature = "adv-int-actchg")]
pub use config::ActChgConfigBuilder;
// Maybe #[cfg(feature = "adv-int-tap")]
pub use config::TapConfigBuilder;

#[cfg(any(feature = "i2c", test))]
pub mod i2c;

#[cfg(any(feature = "spi", test))]
pub mod spi;

pub struct BMA400<T> {
    interface: T,
    config: Config,
}

impl<T, InterfaceError, PinError> BMA400<T> 
where
    T: ReadFromRegister<Error = BMA400Error<InterfaceError, PinError>> + WriteToRegister<Error = BMA400Error<InterfaceError, PinError>>,
    InterfaceError: Debug,
    PinError: Debug,
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

    /// Returns 3-axis data as a [Measurement], with no adjustment for the selected [Scale]
    pub fn get_unscaled_data(&mut self) -> Result<Measurement, BMA400Error<InterfaceError, PinError>> {
        let mut bytes = [0u8; 6];
        self.interface.read_register(AccXLSB, &mut bytes)?;
        Ok(Measurement::from_bytes_unscaled(&bytes))
    }

    /// Returns data as a [Measurement] adjusted for the selected [Scale]
    /// 
    /// To get unscaled data use `get_unscaled_data()`
    pub fn get_data(&mut self) -> Result<Measurement, BMA400Error<InterfaceError, PinError>> {
        let mut bytes = [0u8; 6];
        self.interface.read_register(AccXLSB, &mut bytes)?;
        Ok(Measurement::from_bytes_scaled(self.config.scale(), &bytes))
    }

    /// Timer reading from the integrated sensor clock. 
    /// 
    /// The timer has a resolution of 21 bits stored across 3 bytes.
    /// The lowest 3 bits are always zero (the value is left-justified for compatibility with 25.6kHz clocks).
    /// This timer is inactive in sleep mode. The clock rolls over to zero after `0xFFFFF8`
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
        Ok(f32::from(self.get_raw_temp()?)*0.5 + 23.0)
    }

    /// Configure sensor-wide settings like [PowerMode] and [OversampleRate]
    pub fn config_accel(&mut self) -> AccConfigBuilder<T> {
        AccConfigBuilder::new(self)
    }

    /// Enable or disable interrupts (except the Auto-Wakeup Interrupt, see `config_autowkup()`) and set interrupt latch mode
    pub fn config_interrupts(&mut self) -> IntConfigBuilder<T> {
        IntConfigBuilder::new(self)
    }

    /// Map interrupts to the INT1 / INT2 hardware interrupt pins
    pub fn config_int_pins(&mut self) -> IntPinConfigBuilder<T> {
        IntPinConfigBuilder::new(self)
    }

    /// Configure the FIFO 
    pub fn config_fifo(&mut self) -> FifoConfigBuilder<T> {
        FifoConfigBuilder::new(self)
    }

    /// Configure Auto Low Power settings
    pub fn config_auto_lp(&mut self) -> AutoLpConfigBuilder<T> {
        AutoLpConfigBuilder::new(self)
    }

    /// Configure Auto Wake-up settings
    pub fn config_autowkup(&mut self) -> AutoWakeupConfigBuilder<T> {
        AutoWakeupConfigBuilder::new(self)
    }

    /// Configure Wakeup Interrupt settings
    pub fn config_wkup_int(&mut self) -> WakeupIntConfigBuilder<T> {
        WakeupIntConfigBuilder::new(self)
    }

    /// Configure Orientation Change Interrupt settings
    pub fn config_orientchg_int(&mut self) -> OrientChgConfigBuilder<T> {
        OrientChgConfigBuilder::new(self)
    }

    /// Configure Generic Interrupt 1 settings
    pub fn config_gen1_int(&mut self) -> GenIntConfigBuilder<T> {
        GenIntConfigBuilder::new_gen1(self)
    }

    /// Configure Generic Interrupt 2 settings
    pub fn config_gen2_int(&mut self) -> GenIntConfigBuilder<T> {
        GenIntConfigBuilder::new_gen2(self)
    }

    /// Configure Activity Change Interrupt settings
    pub fn config_actchg_int(&mut self) -> ActChgConfigBuilder<T> {
        ActChgConfigBuilder::new(self)
    }

    /// Configure Advanced Tap Interrupt Settings
    // Maybe #[cfg(feature = "adv-int-tap")]
    pub fn config_tap(&mut self) -> TapConfigBuilder<T> {
        TapConfigBuilder::new(self)
    }

    /// Perform the self test procedure and return [`Ok`] if passed, [`BMA400Error::SelfTestFailedError`] if failed
    /// 
    /// This will disable all interrupts and FIFO write for the duration
    /// 
    /// See p.48 of the datasheet
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

    /// Resets the device and all settings to default
    pub fn soft_reset(&mut self) -> Result<(), BMA400Error<InterfaceError, PinError>> {
        self.interface.write_register(Command::SoftReset)?;
        self.config = Config::default();
        let mut buffer = [0u8; 1];
        // Clear reset detection bit
        self.interface.read_register(Event, &mut buffer)?;
        Ok(())
    }

    /// Consumes the device instance returning the I2C / SPI Interface
    pub fn destroy(self) -> T {
        self.interface
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{interface::{ReadFromRegister, WriteToRegister}, BMA400};
    pub struct NoOpInterface;
    #[derive(Debug)]
    pub struct NoOpError;
    impl ReadFromRegister for NoOpInterface {
        type Error = BMA400Error<NoOpError, ()>;

        fn read_register<T: crate::registers::ReadReg>(&mut self, _register: T, _buffer: &mut [u8]) -> Result<(), Self::Error> {
            Ok(())
        }
    }
    impl WriteToRegister for NoOpInterface {
        type Error = BMA400Error<NoOpError, ()>;

        fn write_register<T: crate::registers::ConfigReg>(&mut self, _register: T) -> Result<(), Self::Error> {
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
