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
    T: ReadFromRegister<Error = E> + WriteToRegister<Error = E>,
    E: From<ConfigError> + Debug,
{
    pub fn get_id(&mut self) -> Result<u8, E> {
        let mut id = [0u8; 1];
        self.interface.read_register(ChipId, &mut id)?;
        Ok(id[0])
    }

    pub fn get_status(&mut self) -> Result<types::Status, E> {
        let mut status_byte = [0u8; 1];
        self.interface.read_register(registers::Status, &mut status_byte)?;
        Ok(types::Status::new(status_byte[0]))
    }

    pub fn get_int_status1(&mut self) -> Result<IntStatus1, E> {
        let mut status_byte = [0u8; 1];
        self.interface.read_register(InterruptStatus1, &mut status_byte)?;
        Ok(IntStatus1::new(status_byte[0]))
    }

    pub fn get_error(&mut self) -> Result<bool, E> {
        let mut err_byte = [0u8; 1];
        self.interface.read_register(ErrReg, &mut err_byte)?;
        Ok(err_byte[0] & 0b00000010 != 0)
    }

    /// Returns 3-axis data as a [Measurement], with no adjustment for the selected [Scale]
    /// 
    ///
    pub fn get_unscaled_data(&mut self) -> Result<Measurement, E> {
        let mut bytes = [0u8; 6];
        self.interface.read_register(AccXLSB, &mut bytes)?;
        Ok(Measurement::from_bytes_unscaled(&bytes))
    }

    /// Returns data as a [Measurement] adjusted for the selected [Scale]
    /// 
    /// To get unscaled data use `get_unscaled_data()`
    pub fn get_data(&mut self) -> Result<Measurement, E> {
        let mut bytes = [0u8; 6];
        self.interface.read_register(AccXLSB, &mut bytes)?;
        Ok(Measurement::from_bytes_scaled(self.config.acc_config.scale(), &bytes))
    }

    /// Returns the number of unread bytes currently in the FIFO Buffer
    pub fn get_fifo_len(&mut self) -> Result<u16, E> {
        let mut buffer = [0u8, 2];
        self.interface.read_register(FifoLength0, &mut buffer)?;
        let bytes = [buffer[0], buffer[1] & 0b0000_0111];
        Ok(u16::from_le_bytes(bytes))
    }

    pub fn read_fifo_frames(&mut self, buffer: &mut [u8]) -> Result<(), E> {
        if self.config.fifo_config.is_read_disabled() {
            return Err(ConfigError::FifoReadWhilePwrDisable.into());
        }
        todo!()
    }

    /// Timer reading from the integrated sensor clock. The timer has a resolution of 21 bits stored across 3 bytes.
    /// The lowest 3 bits are always zero (the value is left-justified for compatibility with 25.6kHz clocks).
    /// This timer is inactive in sleep mode. The clock rolls over to zero after `0xFFFFF8`
    pub fn get_sensor_clock(&mut self) -> Result<u32, E> {
        let mut buffer = [0u8; 3];
        self.interface.read_register(SensorTime0, &mut buffer)?;
        let bytes = [buffer[0], buffer[1], buffer[2], 0];
        Ok(u32::from_le_bytes(bytes))
    }

    /// Chip temperature represented as an i8 with 0.5℃ resolution
    /// 
    /// -128 (-40.0℃) to 
    /// 127 (87.5℃)
    pub fn get_raw_temp(&mut self) -> Result<i8, E> {
        let mut temp = [0u8; 1];
        self.interface.read_register(TempData, &mut temp)?;
        let t = i8::from_le_bytes(temp);
        Ok(t)
    }

    #[cfg(feature = "float")]
    /// Chip temperature in degrees celsius with 0.5℃ resolution
    pub fn get_temp_celsius(&mut self) -> Result<f32, E> {
        Ok(f32::from(self.get_raw_temp()?)*0.5 + 23.0)
    }

    /// Set Power Mode ... TODO
    pub fn config_accel(&mut self) -> AccConfigBuilder<T> {
        AccConfigBuilder::new(self.config.acc_config.clone(), self)
    }

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

    pub fn perform_self_test<Timer: CountDown>(&mut self, timer: &mut Timer) -> Result<(), E> {
        let int_config = self.disable_interrupts()?;
        let acc_config = self.config.acc_config.clone();
        self.config_accel()
            .with_osr(OversampleRate::OSR3)
            .with_scale(Scale::Range4G)
            .with_odr(OutputDataRate::Hz100)
            .write()?;

        // TODO wait 2ms
        // timer.wait();

        //TODO Write positive test parameters to SelfTest register

        // TODO wait 50ms

        // TODO Read acceleration and excitation values

        // TODO Write negative test parameters to SelfTest register

        // TODO wait 50ms

        // Read and store acceleration and excitation values

        // Calculate difference

        // Disable self test

        // Wait 50ms

        // Re-enable interrupts

        todo!();

        Ok(())
    }

    fn disable_interrupts(&mut self) -> Result<IntConfig, E> {
        let int_config0 = self.config.int_config.get_config0();
        let int_config1 = self.config.int_config.get_config1();
        self.interface.write_register(int_config0 ^ int_config0)?;
        self.interface.write_register(int_config1 ^ int_config1)?;
        Ok(self.config.int_config.clone())
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

