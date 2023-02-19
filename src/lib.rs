#![no_std]

use core::fmt::Debug;
pub(crate) use embedded_hal as hal;
#[cfg(feature = "float")]
use accelerometer::{Accelerometer, vector::F32x3, Error as AccError};
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

pub struct ConfigBuilder {
    config: Config,
}

impl ConfigBuilder {
    // AccConfig0
    /// Set Power Mode
    /// 
    /// Note: Other settings can result in the power automatically changing,
    /// for example auto wakeup and auto low-power mode.
    /// 
    /// To read the current power mode use `get_status()`
    pub fn with_power_mode(&mut self, power_mode: PowerMode) -> &mut Self {
        self.config.acc_config0 = self.config.acc_config0.with_power_mode(power_mode);
        self
    }
    /// Set the [OversampleRate] used in [PowerMode::LowPower] power mode
    pub fn with_osr_lp(&mut self, osr: OversampleRate) -> &mut Self {
        self.config.acc_config0 = self.config.acc_config0.with_osr_lp(osr);
        self
    }
    /// Set the [Filter1Bandwidth] for [DataSource::AccFilt1] 
    pub fn with_filt1_bw(&mut self, bandwidth: Filter1Bandwidth) -> &mut Self {
        self.config.acc_config0 = self.config.acc_config0.with_filt1_bw(bandwidth);
        self
    }
    // AccConfig1
    /// Output Data Rate for [DataSource::AccFilt1] 
    pub fn with_odr(&mut self, odr: OutputDataRate) -> &mut Self {
        self.config.acc_config1 = self.config.acc_config1.with_odr(odr);
        self
    }
    /// Set the [OversampleRate] used in [PowerMode::Normal] power mode
    pub fn with_osr(&mut self, osr: OversampleRate) -> &mut Self {
        self.config.acc_config1 = self.config.acc_config1.with_osr(osr);
        self
    }
    /// Set the [Scale] (resolution) for [Measurement]s
    pub fn with_scale(&mut self, scale: Scale) -> &mut Self {
        self.config.acc_config1 = self.config.acc_config1.with_scale(scale);
        self
    }
    // AccConfig2
    /// Set the [DataSource] for the data registers
    pub fn with_dta_reg_src(&mut self, src: DataSource) -> &mut Self {
        self.config.acc_config2 = self.config.acc_config2.with_dta_reg_src(src);
        self
    }
    // IntConfig0
    /// Enable/Disable the Data Ready Interrupt
    pub fn with_dta_rdy_int(&mut self, enabled: bool) -> &mut Self {
        self.config.int_config0 = self.config.int_config0.with_dta_rdy_int(enabled);
        self
    }
    /// Enable/Disable the Fifo Watermark Interrupt
    pub fn with_fwm_int(&mut self, enabled: bool) -> &mut Self {
        self.config.int_config0 = self.config.int_config0.with_fwm_int(enabled);
        self
    }
    /// Enable/Disable the Fifo Full Interrupt
    pub fn with_ffull_int(&mut self, enabled: bool) -> &mut Self {
        self.config.int_config0 = self.config.int_config0.with_ffull_int(enabled);
        self
    }
    /// Enable/Disable Generic Interrupt 2
    pub fn with_gen2_int(&mut self, enabled: bool) -> &mut Self {
        self.config.int_config0 = self.config.int_config0.with_gen2_int(enabled);
        self
    }
    /// Enable/Disable Generic Interrupt 1
    pub fn with_gen1_int(&mut self, enabled: bool) -> &mut Self {
        self.config.int_config0 = self.config.int_config0.with_gen1_int(enabled);
        self
    }
    /// Enable/Disable the Orientation Change Interrupt
    pub fn with_orientch_int(&mut self, enabled: bool) -> &mut Self {
        self.config.int_config0 = self.config.int_config0.with_orientch_int(enabled);
        self
    }
    // IntConfig1
    /// Enable/Disable Latched interrupt mode
    pub fn with_latch_int(&mut self, enabled: bool) -> &mut Self {
        self.config.int_config1 = self.config.int_config1.with_latch_int(enabled);
        self
    }
    /// Enable/Disable Activity Changed Interrupt
    pub fn with_actch_int(&mut self, enabled: bool) -> &mut Self {
        self.config.int_config1 = self.config.int_config1.with_actch_int(enabled);
        self
    }
    /// Enable/Disable Double Tap Interrupt
    pub fn with_d_tap_int(&mut self, enabled: bool) -> &mut Self {
        self.config.int_config1 = self.config.int_config1.with_d_tap_int(enabled);
        self
    }
    /// Enable/Disable Single Tap Interrupt
    pub fn with_s_tap_int(&mut self, enabled: bool) -> &mut Self {
        self.config.int_config1 = self.config.int_config1.with_s_tap_int(enabled);
        self
    }
    /// Enable/Disable Step Interrupt
    pub fn with_step_int(&mut self, enabled: bool) -> &mut Self {
        self.config.int_config1 = self.config.int_config1.with_step_int(enabled);
        self
    }
    // Int1Map
    /// Map Data Ready Interrupt to [InterruptPin::INT1]
    pub fn with_dta_rdy_map_int1(&mut self, mapped: bool) -> &mut Self {
        self.config.int1_map = self.config.int1_map.with_drdy(mapped);
        self
    }
    /// Map Fifo Watermark Interrupt to [InterruptPin::INT1]
    pub fn with_fifo_wm_map_int1(&mut self, mapped: bool) -> &mut Self {
        self.config.int1_map = self.config.int1_map.with_fwm(mapped);
        self
    }

    // Int12Map
    /// Map Tap Interrupt to [InterruptPin::INT1]
    pub fn with_tap_map_int1(&mut self, mapped: bool) -> &mut Self {
        self.config.int12_map = self.config.int12_map.with_tap1(mapped);
        self
    }
    /// Map Tap Interrupt to [InterruptPin::INT2]
    pub fn with_tap_map_int2(&mut self, mapped: bool) -> &mut Self {
        self.config.int12_map = self.config.int12_map.with_tap2(mapped);
        self
    }

    //Int12IOCtrl
    pub fn with_int1_cfg(&mut self, config: PinOutputConfig) -> &mut Self {
        self.config.int12_io_ctrl = self.config.int12_io_ctrl.with_int1_cfg(config);
        self
    }
    pub fn with_int2_cfg(&mut self, config: PinOutputConfig) -> &mut Self {
        self.config.int12_io_ctrl = self.config.int12_io_ctrl.with_int2_cfg(config);
        self
    }

    /// Configure Fifo Data Source
    /// 
    /// Cannot use [DataSource::AccFilt2Lp]. If passed, this will default to AccFilt2
    pub fn with_fifo_src(&mut self, src: DataSource) -> &mut Self {
        let fifo_source = match src {
            DataSource::AccFilt2Lp => DataSource::AccFilt2,
            _ => src,
        };
        self.config.fifo_config0 = self.config.fifo_config0.with_fifo_src(fifo_source);
        self
    }

    pub fn with_wkup_int_num_samples(&mut self, num_samples: u8) -> &mut Self {
        self.config.wkup_int_config0 = self.config.wkup_int_config0.with_num_samples(num_samples.clamp(1, 8) - 1);
        self
    }

    pub(crate) fn build(&mut self) -> Result<Config, ConfigError> {
        self.validate()?;
        Ok(self.config.clone())
    }
    fn is_filt1_used_for_interrupt(&self) -> bool {
        // TODO:
        // Generic Interrupt 1
        // Generic Interrupt 2
        // Activity Change
        false
    }
    fn is_tap_int_enabled(&self) -> bool {
        self.config.int_config1.s_tap_int() || self.config.int_config1.d_tap_int()
    }
    fn is_fifo_enabled(&self) -> bool {
        self.config.fifo_config0.fifo_en()
    }
    fn validate(&self) -> Result<(), ConfigError> {
        if self.is_filt1_used_for_interrupt() {
            match self.config.acc_config1.odr() {
                OutputDataRate::Hz100 => {},
                // Interrupt data source ODR must be 100Hz
                _ => return Err(ConfigError::Filt1InterruptInvalidODR),
            }
        }
        if self.is_tap_int_enabled() {
            match self.config.acc_config1.odr() {
                OutputDataRate::Hz200 => {},
                // Tap Interrupt data source ODR must be 200Hz
                _ => return Err(ConfigError::TapIntEnabledInvalidODR),
            }
        }
        if self.is_fifo_enabled() {
            if self.config.fifo_pwr_config.fifo_pwr_disable() {
                return Err(ConfigError::FifoPwrDisableWhileEnabled)
            }
        }
        Ok(())
    }
}

#[derive(Default, Clone)]
struct Config {
    acc_config0: AccConfig0,
    acc_config1: AccConfig1,
    acc_config2: AccConfig2,
    int_config0: IntConfig0,
    int_config1: IntConfig1,
    int1_map: Int1Map,
    int2_map: Int2Map,
    int12_map: Int12Map,
    int12_io_ctrl: Int12IOCtrl,
    fifo_config0: FifoConfig0,
    fifo_config1: FifoConfig1,
    fifo_config2: FifoConfig2,
    fifo_pwr_config: FifoPwrConfig,
    auto_low_pow0: AutoLowPow0,
    auto_low_pow1: AutoLowPow1,
    auto_wakeup0: AutoWakeup0,
    auto_wakeup1: AutoWakeup1,
    wkup_int_config0: WakeupIntConfig0,
    wkup_int_config1: WakeupIntConfig1,
    wkup_int_config2: WakeupIntConfig2,
    wkup_int_config3: WakeupIntConfig3,
    wkup_int_config4: WakeupIntConfig4,
    /*
    TODO
    orientch_config0: u8,
    orientch_config1: u8,
    orientch_config2: u8,
    orientch_config3: u8,
    orientch_config4: u8,
    orientch_cofnig5: u8,
    orientch_config6: u8,
    orientch_config7: u8,
    orientch_config8: u8,
    orientch_config9: u8,
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
    actch_config0: u8,
    actch_config1: u8,
    */
    tap_config0: TapConfig0,
    tap_config1: TapConfig1,
    /*
    if_conf: u8,
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
    E: From<ConfigError> + Debug
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
        Ok(Measurement::from_bytes_scaled(self.config.acc_config1.scale(), &bytes))
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

    pub fn configure(&mut self) -> ConfigBuilder {
        ConfigBuilder { config: self.config.clone() }
    }

    pub fn set_config(&mut self, config_builder: &mut ConfigBuilder) -> Result<(), E> {
        let config = config_builder.build()?;

        

        if !self.config.acc_config0.symmetric_difference(config.acc_config0).is_empty() {
            self.interface.write_register(config.acc_config0)?;
            self.config.acc_config0 = config.acc_config0;
        }

        if self.config.acc_config1.bits() != config.acc_config1.bits() {
            self.interface.write_register(config.acc_config1)?;
            self.config.acc_config1 = config.acc_config1;
        }
        
        if self.config.acc_config2.bits() != config.acc_config2.bits() {
            self.interface.write_register(config.acc_config2)?;
            self.config.acc_config2 = config.acc_config2;
        }

        // Disable interrupts with config changes, write the changes and then re-enable them
        self.process_interrupt_config_changes(&config)?;
        Ok(())
    }

    pub fn destroy(self) -> T {
        self.interface
    }

    fn process_interrupt_config_changes(&mut self, config: &Config) -> Result<(), E> {
        // Any change of an interrupt configuration must be executed when the corresponding interrupt is
        // disabled. (Datasheet p. 40)
        
        // IntConfig0
        let mut tmp_int_config0 = self.config.int_config0;

        // Data Ready
        let has_drdy_changes = config.int1_map.drdy_int() != self.config.int1_map.drdy_int() || config.int2_map.drdy_int() != self.config.int2_map.drdy_int();
        if tmp_int_config0.dta_rdy_int() && has_drdy_changes {
            tmp_int_config0 = tmp_int_config0.with_dta_rdy_int(false);
        }

        // FIFO Watermark
        let has_fifo0_changes = config.fifo_config0.bits() != self.config.fifo_config0.bits();
        let has_fifo1_changes = config.fifo_config1.bits() != self.config.fifo_config1.bits();
        let has_fifo2_changes = config.fifo_config2.bits() != self.config.fifo_config2.bits();
        let has_int1_map_changes = config.int1_map.fwm_int() != self.config.int1_map.fwm_int();
        let has_int2_map_changes = config.int2_map.fwm_int() != self.config.int1_map.fwm_int();
        let has_fifowm_changes = has_fifo0_changes || has_fifo1_changes || has_fifo2_changes || has_int1_map_changes || has_int2_map_changes;
        if tmp_int_config0.fwm_int() && has_fifowm_changes {
            tmp_int_config0 = tmp_int_config0.with_fwm_int(false);
        }

        // FIFO Full
        let has_ffull_changes = config.int1_map.ffull_int() != self.config.int1_map.ffull_int() || config.int2_map.ffull_int() != self.config.int2_map.ffull_int();
        if tmp_int_config0.ffull_int() && has_ffull_changes {
            tmp_int_config0 = tmp_int_config0.with_ffull_int(false);
        }

        // Generic Int 2

        // Generic Int 1

        // Orientation Change

        // IntConfig1
        let mut tmp_int_config1 = self.config.int_config1;

        // Activity Change Interrupt

        // Double Tap Interrupt
        let has_tap_map_changes = config.int12_map.tap_int1() != self.config.int12_map.tap_int1() || config.int12_map.tap_int2() != self.config.int12_map.tap_int2();
        let has_tap_changes = has_tap_map_changes || config.tap_config0.bits() != self.config.tap_config0.bits() || config.tap_config1.bits() != config.tap_config1.bits();
        if tmp_int_config1.d_tap_int() && has_tap_changes {
            tmp_int_config1 = tmp_int_config1.with_d_tap_int(false);
        }

        // Single Tap Interrupt
        if tmp_int_config1.s_tap_int() && has_tap_changes {
            tmp_int_config1 = tmp_int_config1.with_s_tap_int(false);
        }

        // Step Interrupt
        // TODO: Config in Registers 0x59-0x70 per p.43 of datasheet


        // WakeupIntConfig0
        let mut tmp_wkup_config0 = self.config.wkup_int_config0;

        // Wakeup Interrupt
        let has_wkup_config0_changes = config.wkup_int_config0.bits() != self.config.wkup_int_config0.bits();
        let has_wkup_config1_changes = config.wkup_int_config1.bits() != self.config.wkup_int_config1.bits();
        let has_wkup_config2_changes = config.wkup_int_config2.bits() != self.config.wkup_int_config2.bits();
        let has_wkup_config3_changes = config.wkup_int_config3.bits() != self.config.wkup_int_config3.bits();
        let has_wkup_config4_changes = config.wkup_int_config4.bits() != self.config.wkup_int_config4.bits();
        let has_wkup_map_changes = config.int1_map.wkup_int() != self.config.int1_map.wkup_int() || config.int2_map.wkup_int() != self.config.int2_map.wkup_int();
        let has_wkup_int_changes = has_wkup_map_changes || has_wkup_config0_changes || has_wkup_config1_changes || has_wkup_config2_changes || has_wkup_config3_changes || has_wkup_config4_changes;
        if self.config.wkup_int_config0.wkup_int_en() && has_wkup_int_changes {
            tmp_wkup_config0 = tmp_wkup_config0.with_x_axis(false).with_y_axis(false).with_z_axis(false);
        }

        // Temporarily disable interrupts with config changes
        self.config.int_config0.write_update(&mut self.interface, tmp_int_config0)?;
        self.config.int_config1.write_update(&mut self.interface, tmp_int_config1)?;
        self.config.wkup_int_config0.write_update(&mut self.interface, tmp_wkup_config0)?;

        // Update Config
        self.config.fifo_config0.write_update(&mut self.interface, config.fifo_config0)?;
        self.config.fifo_config1.write_update(&mut self.interface, config.fifo_config1)?;
        self.config.fifo_config2.write_update(&mut self.interface, config.fifo_config2)?;

        self.config.tap_config0.write_update(&mut self.interface, config.tap_config0)?;
        self.config.tap_config1.write_update(&mut self.interface, config.tap_config1)?;

        self.config.wkup_int_config1.write_update(&mut self.interface, config.wkup_int_config1)?;
        self.config.wkup_int_config2.write_update(&mut self.interface, config.wkup_int_config2)?;
        self.config.wkup_int_config3.write_update(&mut self.interface, config.wkup_int_config3)?;
        self.config.wkup_int_config4.write_update(&mut self.interface, config.wkup_int_config4)?;

        self.config.int12_io_ctrl.write_update(&mut self.interface, config.int12_io_ctrl)?;
        self.config.int1_map.write_update(&mut self.interface, config.int1_map)?;
        self.config.int2_map.write_update(&mut self.interface, config.int2_map)?;
        self.config.int12_map.write_update(&mut self.interface, config.int12_map)?;

        // Re-enable disabled interrupts
        self.config.int_config0.write_update(&mut self.interface, config.int_config0)?;
        self.config.int_config1.write_update(&mut self.interface, config.int_config1)?;
        self.config.wkup_int_config0.write_update(&mut self.interface, config.wkup_int_config0)?;
        Ok(())
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

