use core::fmt::Debug;

use crate::{registers::{IntConfig0, IntConfig1}, interface::WriteToRegister, ConfigError, BMA400, OutputDataRate, DataSource};


#[derive(Clone, Default)]
pub struct IntConfig {
    int_config0: IntConfig0,
    int_config1: IntConfig1,
}

impl IntConfig {
    // API to quickly enable / disable interrupts for config changes
    pub fn get_config0(&self) -> IntConfig0 {
        self.int_config0
    }
    pub fn get_config1(&self) -> IntConfig1 {
        self.int_config1
    }
}

pub struct IntConfigBuilder<'a, Interface: WriteToRegister> 
{
    config: IntConfig,
    device: &'a mut BMA400<Interface>,
}

impl<'a, Interface, E> IntConfigBuilder<'a, Interface> 
where
    Interface: WriteToRegister<Error = E>,
    E: From<ConfigError> + Debug,
{
    pub fn new(config: IntConfig, device: &'a mut BMA400<Interface>) -> IntConfigBuilder<'a, Interface> {
        IntConfigBuilder { config, device }
    }
    // IntConfig0
    /// Enable/Disable the Data Ready Interrupt
    pub fn with_dta_rdy_int(mut self, enabled: bool) -> Self {
        self.config.int_config0 = self.config.int_config0.with_dta_rdy_int(enabled);
        self
    }
    /// Enable/Disable the Fifo Watermark Interrupt
    pub fn with_fwm_int(mut self, enabled: bool) -> Self {
        self.config.int_config0 = self.config.int_config0.with_fwm_int(enabled);
        self
    }
    /// Enable/Disable the Fifo Full Interrupt
    pub fn with_ffull_int(mut self, enabled: bool) -> Self {
        self.config.int_config0 = self.config.int_config0.with_ffull_int(enabled);
        self
    }
    /// Enable/Disable Generic Interrupt 2
    pub fn with_gen2_int(mut self, enabled: bool) -> Self {
        self.config.int_config0 = self.config.int_config0.with_gen2_int(enabled);
        self
    }
    /// Enable/Disable Generic Interrupt 1
    pub fn with_gen1_int(mut self, enabled: bool) -> Self {
        self.config.int_config0 = self.config.int_config0.with_gen1_int(enabled);
        self
    }
    /// Enable/Disable the Orientation Change Interrupt
    pub fn with_orientch_int(mut self, enabled: bool) -> Self {
        self.config.int_config0 = self.config.int_config0.with_orientch_int(enabled);
        self
    }
    // IntConfig1
    /// Enable/Disable Latched interrupt mode
    pub fn with_latch_int(mut self, enabled: bool) -> Self {
        self.config.int_config1 = self.config.int_config1.with_latch_int(enabled);
        self
    }
    /// Enable/Disable Activity Changed Interrupt
    pub fn with_actch_int(mut self, enabled: bool) -> Self {
        self.config.int_config1 = self.config.int_config1.with_actch_int(enabled);
        self
    }
    /// Enable/Disable Double Tap Interrupt
    pub fn with_d_tap_int(mut self, enabled: bool) -> Self {
        self.config.int_config1 = self.config.int_config1.with_d_tap_int(enabled);
        self
    }
    /// Enable/Disable Single Tap Interrupt
    pub fn with_s_tap_int(mut self, enabled: bool) -> Self {
        self.config.int_config1 = self.config.int_config1.with_s_tap_int(enabled);
        self
    }
    /// Enable/Disable Step Interrupt
    pub fn with_step_int(mut self, enabled: bool) -> Self {
        self.config.int_config1 = self.config.int_config1.with_step_int(enabled);
        self
    }
    pub fn write(self) -> Result<(), E> {
        if self.config.int_config1.d_tap_int() || self.config.int_config1.s_tap_int() {
            match self.device.config.acc_config.odr() {
                OutputDataRate::Hz200 => {},
                // Tap Interrupt data source ODR must be 200Hz
                _ => return Err(ConfigError::TapIntEnabledInvalidODR.into()),
            }
        }
        
        // Check DataSource for each enabled interrupt that can use Filt1 and validate

        // Gen 1
        // TODO
        // Gen 2
        // TODO
        // Activity Change
        if self.config.int_config1.actch_int() && !matches!(self.device.config.actch_config.src(), DataSource::AccFilt2) {
            return Err(ConfigError::Filt1InterruptInvalidODR.into())
        }

        if self.device.config.int_config.int_config0.bits() != self.config.int_config0.bits() {
            self.device.interface.write_register(self.config.int_config0)?;
            self.device.config.int_config.int_config0 = self.config.int_config0;
        }
        if self.device.config.int_config.int_config1.bits() != self.config.int_config1.bits() {
            self.device.interface.write_register(self.config.int_config1)?;
            self.device.config.int_config.int_config1 = self.config.int_config1;
        }
        Ok(())
    }
}