use crate::{
    interface::WriteToRegister,
    registers::{AutoLowPow0, AutoLowPow1},
    AutoLPTimeoutTrigger, ConfigError, BMA400,
};

#[derive(Clone, Default)]
pub struct AutoLpConfig {
    auto_low_pow0: AutoLowPow0,
    auto_low_pow1: AutoLowPow1,
}

/// Configure Auto Low Power settings
///
/// - Set the timeout counter for low power mode using [`with_timeout()`](AutoLpConfigBuilder::with_timeout)
/// - [AutoLPTimeoutTrigger] (trigger and timer reset condition) using [`with_auto_lp_trigger()`](AutoLpConfigBuilder::with_auto_lp_trigger)
/// - Set Generic Interrupt 1 as a trigger condition for auto low power using [`with_gen1_int_trigger()`](AutoLpConfigBuilder::with_gen1_int_trigger)
/// - Set Data Ready as a trigger condition for auto low power using [`with_drdy_trigger()`](AutoLpConfigBuilder::with_drdy_trigger)
pub struct AutoLpConfigBuilder<'a, Interface: WriteToRegister> {
    config: AutoLpConfig,
    device: &'a mut BMA400<Interface>,
}

impl<'a, Interface, E> AutoLpConfigBuilder<'a, Interface>
where
    Interface: WriteToRegister<Error = E>,
    E: From<ConfigError>,
{
    pub(crate) fn new(device: &'a mut BMA400<Interface>) -> AutoLpConfigBuilder<'a, Interface> {
        AutoLpConfigBuilder {
            config: device.config.auto_lp_config.clone(),
            device,
        }
    }
    // AutoLowPow0 + AutoLowPow1

    /// Set the timeout counter for auto low power mode. This value is 12-bits, and is incremented
    /// every 2.5ms
    ///
    /// This value is clamped to \[0, 4095\]
    pub fn with_timeout(mut self, count: u16) -> Self {
        let timeout = count.clamp(0, 4095);
        self.config.auto_low_pow0 = self.config.auto_low_pow0.with_auto_lp_timeout_msb(timeout);
        self.config.auto_low_pow1 = self.config.auto_low_pow1.with_auto_lp_timeout_lsb(timeout);
        self
    }
    // AutoLowPow1

    /// Set the auto low power trigger condition
    pub fn with_auto_lp_trigger(mut self, trigger: AutoLPTimeoutTrigger) -> Self {
        self.config.auto_low_pow1 = self.config.auto_low_pow1.with_auto_lp_timeout_mode(trigger);
        self
    }
    /// Set Generic Interrupt 1 as a trigger condition for auto low power
    pub fn with_gen1_int_trigger(mut self, enabled: bool) -> Self {
        self.config.auto_low_pow1 = self.config.auto_low_pow1.with_gen1_int_trigger(enabled);
        self
    }
    /// Set new data ready as a trigger condition for auto low power
    pub fn with_drdy_trigger(mut self, enabled: bool) -> Self {
        self.config.auto_low_pow1 = self.config.auto_low_pow1.with_drdy_trigger(enabled);
        self
    }
    /// Write the configuration to device registers
    pub fn write(self) -> Result<(), E> {
        if self.device.config.auto_lp_config.auto_low_pow0.bits()
            != self.config.auto_low_pow0.bits()
        {
            self.device
                .interface
                .write_register(self.config.auto_low_pow0)?;
            self.device.config.auto_lp_config.auto_low_pow0 = self.config.auto_low_pow0;
        }
        if self.device.config.auto_lp_config.auto_low_pow1.bits()
            != self.config.auto_low_pow1.bits()
        {
            self.device
                .interface
                .write_register(self.config.auto_low_pow1)?;
            self.device.config.auto_lp_config.auto_low_pow1 = self.config.auto_low_pow1;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::get_test_device;
    #[test]
    fn test_timeout() {
        let mut device = get_test_device();
        let builder = device.config_auto_lp();
        let builder = builder.with_timeout(4098);
        assert_eq!(builder.config.auto_low_pow0.bits(), 0xFF);
        assert_eq!(builder.config.auto_low_pow1.bits(), 0xF0);
        let builder = builder.with_timeout(0);
        assert_eq!(builder.config.auto_low_pow0.bits(), 0x00);
        assert_eq!(builder.config.auto_low_pow1.bits(), 0x00);
    }
    #[test]
    fn test_auto_lp_trigger() {
        let mut device = get_test_device();
        let builder = device.config_auto_lp();
        let builder = builder.with_auto_lp_trigger(AutoLPTimeoutTrigger::TimeoutEnabledNoReset);
        assert_eq!(builder.config.auto_low_pow1.bits(), 0x04);
        let builder =
            builder.with_auto_lp_trigger(AutoLPTimeoutTrigger::TimeoutEnabledGen2IntReset);
        assert_eq!(builder.config.auto_low_pow1.bits(), 0x08);
        let builder = builder.with_auto_lp_trigger(AutoLPTimeoutTrigger::TimeoutDisabled);
        assert_eq!(builder.config.auto_low_pow1.bits(), 0x00);
    }
    #[test]
    fn test_gen1_int_trigger() {
        let mut device = get_test_device();
        let builder = device.config_auto_lp();
        let builder = builder.with_gen1_int_trigger(true);
        assert_eq!(builder.config.auto_low_pow1.bits(), 0x02);
        let builder = builder.with_gen1_int_trigger(false);
        assert_eq!(builder.config.auto_low_pow1.bits(), 0x00);
    }
    #[test]
    fn test_drdy_trigger() {
        let mut device = get_test_device();
        let builder = device.config_auto_lp();
        let builder = builder.with_drdy_trigger(true);
        assert_eq!(builder.config.auto_low_pow1.bits(), 0x01);
        let builder = builder.with_drdy_trigger(false);
        assert_eq!(builder.config.auto_low_pow1.bits(), 0x00);
    }
}
