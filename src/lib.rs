#![no_std]

use core::fmt::Debug;
pub(crate) use embedded_hal as hal;
use hal::prelude::_embedded_hal_timer_CountDown;
use hal::timer::CountDown;
#[cfg(feature = "float")]
use accelerometer::{Accelerometer, vector::F32x3, Error as AccError};
//#[cfg(feature = "advanced-actchg")]
mod actchg_config;
use actchg_config::ActChgConfig;
pub use actchg_config::ActChgBuilder;
//#[cfg(feature = "tap")]
mod tap_config;
use tap_config::TapConfig;
pub use tap_config::TapConfigBuilder;
mod accel_config;
use accel_config::AccConfig;
pub use accel_config::AccConfigBuilder;
mod int_config;
use int_config::IntConfig;
pub use int_config::IntConfigBuilder;
mod int_pin_config;
use int_pin_config::IntPinConfig;
pub use int_pin_config::IntPinConfigBuilder;
mod fifo_config;
use fifo_config::FifoConfig;
pub use fifo_config::FifoConfigBuilder;
mod auto_lp_config;
use auto_lp_config::AutoLpConfig;
pub use auto_lp_config::AutoLpConfigBuilder;
mod auto_wkup_config;
use auto_wkup_config::AutoWakeupConfig;
pub use auto_wkup_config::AutoWakeupConfigBuilder;
mod wkup_int_config;
use wkup_int_config::WakeupIntConfig;
pub use wkup_int_config::WakeupIntConfigBuilder;
mod orientch_config;
use orientch_config::OrientChgConfig;
pub use orientch_config::OrientChgConfigBuilder;
pub mod types;
pub use types::*;
mod interface;
use interface::{ReadFromRegister, WriteToRegister};
pub(crate) mod registers;
use registers::*;

//#[cfg(feature = "i2c")]
pub mod i2c;

#[cfg(feature = "spi")]
pub mod spi;

#[derive(Default, Clone)]
struct Config {
    acc_config: AccConfig,
    int_config: IntConfig,
    int_pin_config: IntPinConfig,
    fifo_config: FifoConfig,
    auto_lp_config: AutoLpConfig,
    auto_wkup_config: AutoWakeupConfig,
    wkup_int_config: WakeupIntConfig,
    orientch_config: OrientChgConfig,
    
    /* TODO
    gen1int_config0: u8,
    gen1int_config1: u8,
    gen1int_config2: u8,
    gen1int_config3: u8,
    gen1int_config31: u8,
    gen1int_config4: u8,
    gen1int_config5: u8,
    gen1int_config6: u8,
    gen1int_config7: u8,
    gen1int_config8: u8,
    gen1int_config9: u8,
    gen2int_config0: u8,
    gen2int_config1: u8,
    gen2int_config2: u8,
    gen2int_config3: u8,
    gen2int_config31: u8,
    gen2int_config4: u8,
    gen2int_config5: u8,
    gen2int_config6: u8,
    gen2int_config7: u8,
    gen2int_config8: u8,
    gen2int_config9: u8,
    */

    //#[cfg(feature = "advanced-actchg")]
    actch_config: ActChgConfig,

    //#[cfg(feature = "advanced-tap")]
    tap_config: TapConfig,

    /* TODO
    #[cfg(feature = "spi")]
    if_conf: InterfaceConfig,
    self_test: u8,
    cmd: u8,
    */
}

pub struct BMA400<T> {
    interface: T,
    config: Config,
}

impl<T, E> BMA400<T> 
where
    T: ReadFromRegister<Error = BMA400Error<E>> + WriteToRegister<Error = BMA400Error<E>>,
    E: Debug,
{
    /// Returns the chip ID (0x90)
    pub fn get_id(&mut self) -> Result<u8, BMA400Error<E>> {
        let mut id = [0u8; 1];
        self.interface.read_register(ChipId, &mut id)?;
        Ok(id[0])
    }

    /// Reads and returns the status of the command error register
    /// 
    /// Errors are cleared on read
    pub fn get_cmd_error(&mut self) -> Result<bool, BMA400Error<E>> {
        let mut err_byte = [0u8; 1];
        self.interface.read_register(ErrReg, &mut err_byte)?;
        Ok(err_byte[0] & 0b00000010 != 0)
    }

    /// Reads and returns the sensor [Status] register
    pub fn get_status(&mut self) -> Result<Status, BMA400Error<E>> {
        let mut status_byte = [0u8; 1];
        self.interface.read_register(StatusReg, &mut status_byte)?;
        Ok(Status::new(status_byte[0]))
    }

    /// Returns 3-axis data as a [Measurement], with no adjustment for the selected [Scale]
    pub fn get_unscaled_data(&mut self) -> Result<Measurement, BMA400Error<E>> {
        let mut bytes = [0u8; 6];
        self.interface.read_register(AccXLSB, &mut bytes)?;
        Ok(Measurement::from_bytes_unscaled(&bytes))
    }

    /// Returns data as a [Measurement] adjusted for the selected [Scale]
    /// 
    /// To get unscaled data use `get_unscaled_data()`
    pub fn get_data(&mut self) -> Result<Measurement, BMA400Error<E>> {
        let mut bytes = [0u8; 6];
        self.interface.read_register(AccXLSB, &mut bytes)?;
        Ok(Measurement::from_bytes_scaled(self.config.acc_config.scale(), &bytes))
    }

    /// Timer reading from the integrated sensor clock. 
    /// 
    /// The timer has a resolution of 21 bits stored across 3 bytes.
    /// The lowest 3 bits are always zero (the value is left-justified for compatibility with 25.6kHz clocks).
    /// This timer is inactive in sleep mode. The clock rolls over to zero after `0xFFFFF8`
    pub fn get_sensor_clock(&mut self) -> Result<u32, BMA400Error<E>> {
        let mut buffer = [0u8; 3];
        self.interface.read_register(SensorTime0, &mut buffer)?;
        let bytes = [buffer[0], buffer[1], buffer[2], 0];
        Ok(u32::from_le_bytes(bytes))
    }

    /// Returns `true` if a power reset has been detected
    pub fn get_reset_status(&mut self) -> Result<bool, BMA400Error<E>> {
        let mut buffer = [0u8; 1];
        self.interface.read_register(Event, &mut buffer)?;
        Ok(buffer[0] & 0x01 != 0)
    }

    /// Reads and returns the [IntStatus0] interrupt status register
    pub fn get_int_status0(&mut self) -> Result<IntStatus0, BMA400Error<E>> {
        let mut status_byte = [0u8; 1];
        self.interface.read_register(InterruptStatus0, &mut status_byte)?;
        Ok(IntStatus0::new(status_byte[0]))
    }

    /// Reads and returns the [IntStatus1] interrupt status register
    pub fn get_int_status1(&mut self) -> Result<IntStatus1, BMA400Error<E>> {
        let mut status_byte = [0u8; 1];
        self.interface.read_register(InterruptStatus1, &mut status_byte)?;
        Ok(IntStatus1::new(status_byte[0]))
    }

    /// Reads and returns the [IntStatus2] interrupt status register
    pub fn get_int_status2(&mut self) -> Result<IntStatus2, BMA400Error<E>> {
        let mut status_byte = [0u8; 1];
        self.interface.read_register(InterruptStatus2, &mut status_byte)?;
        Ok(IntStatus2::new(status_byte[0]))
    }

    /// Returns the number of unread bytes currently in the FIFO Buffer
    pub fn get_fifo_len(&mut self) -> Result<u16, BMA400Error<E>> {
        let mut buffer = [0u8, 2];
        self.interface.read_register(FifoLength0, &mut buffer)?;
        let bytes = [buffer[0], buffer[1] & 0b0000_0111];
        Ok(u16::from_le_bytes(bytes))
    }

    /// Reads a number of data frames from the FIFO Buffer
    pub fn read_fifo_frames(&mut self, buffer: &mut [u8]) -> Result<(), BMA400Error<E>> {
        if self.config.fifo_config.is_read_disabled() {
            return Err(ConfigError::FifoReadWhilePwrDisable.into());
        }
        todo!()
    }

    /// Flush all data from the FIFO Buffer
    pub fn flush_fifo(&mut self) -> Result<(), BMA400Error<E>> {
        self.interface.write_register(Command::FlushFifo)?;
        Ok(())
    }

    /// Get the step count 
    /// 
    /// (the counter only increments if the step interrupt is enabled)
    pub fn get_step_count(&mut self) -> Result<u32, BMA400Error<E>> {
        let mut buffer = [0u8; 3];
        self.interface.read_register(StepCount0, &mut buffer)?;
        Ok(u32::from_le_bytes([buffer[0], buffer[1], buffer[2], 0]))
    }

    /// Reset the step count to 0
    pub fn clear_step_count(&mut self) -> Result<(), BMA400Error<E>> {
        self.interface.write_register(Command::ClearStepCount)?;
        Ok(())
    }

    /// Chip temperature represented as an i8 with 0.5℃ resolution
    /// 
    /// -128 (-40.0℃) to 
    /// 127 (87.5℃)
    pub fn get_raw_temp(&mut self) -> Result<i8, BMA400Error<E>> {
        let mut temp = [0u8; 1];
        self.interface.read_register(TempData, &mut temp)?;
        let t = i8::from_le_bytes(temp);
        Ok(t)
    }

    #[cfg(feature = "float")]
    /// Chip temperature in degrees celsius with 0.5℃ resolution
    pub fn get_temp_celsius(&mut self) -> Result<f32, BMA400Error<E>> {
        Ok(f32::from(self.get_raw_temp()?)*0.5 + 23.0)
    }

    /// Configure sensor-wide settings like [PowerMode] and [OversampleRate]
    pub fn config_accel(&mut self) -> AccConfigBuilder<T> {
        AccConfigBuilder::new(self.config.acc_config.clone(), self)
    }

    /// Enable or disable interrupts (except //TODO) and set interrupt latch mode
    pub fn config_interrupts(&mut self) -> IntConfigBuilder<T> {
        IntConfigBuilder::new(self.config.int_config.clone(), self)
    }

    pub fn config_int_pins(&mut self) -> IntPinConfigBuilder<T> {
        IntPinConfigBuilder::new(self.config.int_pin_config.clone(), self)
    }

    #[cfg(feature = "tap")]
    pub fn adv_tap_cfg(&mut self) -> TapConfigBuilder {
        TapConfigBuilder { config: config.tap_config }
    }

    /// Perform the self test procedure and return [`Ok`] if passed, [`BMA400Error::SelfTestFailedError`] if failed
    /// 
    /// This will disable all interrupts and FIFO write for the duration
    /// 
    /// See p.48 of the Datasheet
    pub fn perform_self_test<Timer: CountDown>(&mut self, timer: &mut Timer) -> Result<(), BMA400Error<E>> {
        let int_config0 = self.config.int_config.get_config0();
        let int_config1 = self.config.int_config.get_config1();
        // TODO: Other interrupts
        self.interface.write_register(int_config0 ^ int_config0)?;
        self.interface.write_register(int_config1 ^ int_config1)?;
        // Disable FIFO
        let fifo_config0 = self.config.fifo_config.get_config0();
        self.interface.write_register(fifo_config0.with_fifo_x(false).with_fifo_y(false).with_fifo_z(false))?;

        self.config_accel()
            .with_osr(OversampleRate::OSR3)
            .with_scale(Scale::Range4G)
            .with_odr(OutputDataRate::Hz100)
            .write()?;

        // TODO wait 2ms
        // timer.wait();

        //TODO Write positive test parameters to SelfTest register
        self.interface.write_register(SelfTest::from_bits_truncate(0x07))?;

        // TODO wait 50ms

        // TODO Read acceleration and excitation values
        let m_pos = self.get_unscaled_data()?;

        // TODO Write negative test parameters to SelfTest register
        self.interface.write_register(SelfTest::from_bits_truncate(0x0F))?;

        // TODO wait 50ms

        // Read and store acceleration and excitation values
        let m_neg = self.get_unscaled_data()?;

        // Calculate difference
        let (x, y, z) = (m_pos.x - m_neg.x, m_pos.y - m_neg.y, m_pos.z - m_neg.z);

        // Disable self test
        self.interface.write_register(SelfTest::default())?;

        // TODO Wait 50ms

        // Re-enable interrupts
        AccConfigBuilder::new(self.config.acc_config.clone(), self).write()?;
        FifoConfigBuilder::new(self.config.fifo_config.clone(), self).write()?;
        IntConfigBuilder::new(self.config.int_config.clone(), self).write()?;

        if x > 1500 && y > 1200 && z > 250 {
            Ok(())
        } else {
            Err(BMA400Error::SelfTestFailedError)
        }
    }

    /// Resets the device and all settings to default
    pub fn soft_reset(&mut self) -> Result<(), BMA400Error<E>> {
        self.interface.write_register(Command::SoftReset)?;
        self.config = Config::default();
        let mut buffer = [0u8; 1];
        // Clear reset detection bit
        self.interface.read_register(Event, &mut buffer)?;
        Ok(())
    }

    pub fn destroy(self) -> T {
        self.interface
    }
}

#[cfg(feature = "float")]
impl<T, E> Accelerometer for BMA400<T> 
where
    T: ReadFromRegister<Error = E> + WriteToRegister<Error = E>,
    E: Debug,
{
    type Error = AccError<E>;

    fn accel_norm(&mut self) -> Result<F32x3, AccError<Self::Error>> {
        todo!()
    }

    fn sample_rate(&mut self) -> Result<f32, AccError<Self::Error>> {
        todo!()
    }
}

