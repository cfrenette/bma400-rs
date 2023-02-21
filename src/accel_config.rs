use crate::{registers::{AccConfig0, AccConfig1, AccConfig2}, Scale, interface::WriteToRegister, ConfigError, PowerMode, OversampleRate, Filter1Bandwidth, OutputDataRate, DataSource, BMA400};
use core::fmt::Debug;


#[derive(Clone, Default)]
pub struct AccConfig {
    acc_config0: AccConfig0,
    acc_config1: AccConfig1,
    acc_config2: AccConfig2,
}

impl AccConfig {
    pub fn odr(&self) -> OutputDataRate {
        self.acc_config1.odr()
    }

    pub fn scale(&self) -> Scale {
        self.acc_config1.scale()
    }
}

/// Configure basic accelerometer settings like Output Data Rate (ODR)
pub struct AccConfigBuilder<'a, Interface: WriteToRegister> 
{
    config: AccConfig,
    device: &'a mut BMA400<Interface>,
}

impl<'a, Interface, E> AccConfigBuilder<'a, Interface> 
where
    Interface: WriteToRegister<Error = E>,
    E: From<ConfigError> + Debug,
{
    pub fn new(config: AccConfig, device: &'a mut BMA400<Interface>) -> AccConfigBuilder<'a, Interface> {
        AccConfigBuilder { config, device }
    }
    // AccConfig0
    /// Set Power Mode
    /// 
    /// Note: Other settings can result in the power automatically changing,
    /// for example auto wakeup and auto low-power mode.
    /// 
    /// To read the current power mode use `get_status()`
    pub fn with_power_mode(mut self, power_mode: PowerMode) -> Self {
        self.config.acc_config0 = self.config.acc_config0.with_power_mode(power_mode);
        self
    }
    /// Set the [OversampleRate] used in [PowerMode::LowPower] power mode
    pub fn with_osr_lp(mut self, osr: OversampleRate) -> Self {
        self.config.acc_config0 = self.config.acc_config0.with_osr_lp(osr);
        self
    }
    /// Set the [Filter1Bandwidth] for [DataSource::AccFilt1] 
    pub fn with_filt1_bw(mut self, bandwidth: Filter1Bandwidth) -> Self {
        self.config.acc_config0 = self.config.acc_config0.with_filt1_bw(bandwidth);
        self
    }
    // AccConfig1
    /// Output Data Rate for [DataSource::AccFilt1] 
    pub fn with_odr(mut self, odr: OutputDataRate) -> Self {
        self.config.acc_config1 = self.config.acc_config1.with_odr(odr);
        self
    }
    /// Set the [OversampleRate] used in [PowerMode::Normal] power mode
    pub fn with_osr(mut self, osr: OversampleRate) -> Self {
        self.config.acc_config1 = self.config.acc_config1.with_osr(osr);
        self
    }
    /// Set the [Scale] (resolution) for [Measurement]s
    pub fn with_scale(mut self, scale: Scale) -> Self {
        self.config.acc_config1 = self.config.acc_config1.with_scale(scale);
        self
    }
    // AccConfig2
    /// Set the [DataSource] for the data registers
    pub fn with_reg_dta_src(mut self, src: DataSource) -> Self {
        self.config.acc_config2 = self.config.acc_config2.with_dta_reg_src(src);
        self
    }
    /// Write the configuration to device registers
    pub fn write(self) -> Result<(), E> {
        /* TODO Gen Int 1 / 2
        let int_config0 = self.device.config.int_config.get_config0();
        */
        let int_config1 = self.device.config.int_config.get_config1();

        // If Gen Int 1 / 2 or Activity Change use filt1 and are enabled ODR must be 100Hz
        let mut filt1_used_for_ints = false;
        if int_config1.actch_int() && matches!(self.device.config.actch_config.src(), DataSource::AccFilt1) {
            filt1_used_for_ints = true;
        }
        // TODO Gen Int 1 / 2

        if filt1_used_for_ints && !matches!(self.config.odr(), OutputDataRate::Hz100) {
            return Err(ConfigError::Filt1InterruptInvalidODR.into());
        }
        // If either Tap Interrupt is enabled, filt1 ODR must be set to 200Hz
        if (int_config1.d_tap_int() || int_config1.s_tap_int()) && !matches!(self.config.odr(), OutputDataRate::Hz200) {
            return Err(ConfigError::Filt1InterruptInvalidODR.into());
        }
        if self.device.config.acc_config.acc_config0.bits() != self.config.acc_config0.bits() {
            self.device.interface.write_register(self.config.acc_config0)?;
            self.device.config.acc_config.acc_config0 = self.config.acc_config0;
        }
        if self.device.config.acc_config.acc_config1.bits() != self.config.acc_config1.bits() {
            self.device.interface.write_register(self.config.acc_config1)?;
            self.device.config.acc_config.acc_config1 = self.config.acc_config1;
        }
        if self.device.config.acc_config.acc_config2.bits() != self.config.acc_config2.bits() {
            self.device.interface.write_register(self.config.acc_config2)?;
            self.device.config.acc_config.acc_config2 = self.config.acc_config2;
        }
        Ok(())
    }
}