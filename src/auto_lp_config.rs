use crate::{
    Debug,
    registers::{AutoLowPow0, AutoLowPow1},
    interface::WriteToRegister,
    BMA400,
    ConfigError, AutoLPTimeoutTrigger, 
};

#[derive(Clone, Default)]
pub struct AutoLpConfig {
    auto_low_pow0: AutoLowPow0,
    auto_low_pow1: AutoLowPow1,
}

pub struct AutoLpConfigBuilder<'a, Interface:WriteToRegister> {
    config: AutoLpConfig,
    device: &'a mut BMA400<Interface>
}

impl<'a, Interface, E> AutoLpConfigBuilder<'a, Interface> 
where
    Interface: WriteToRegister<Error = E>,
    E: From<ConfigError> + Debug,
{
    // AutoLowPow0 + AutoLowPow1

    /// Set the timeout counter for auto low power mode. This value is 12-bits, and is incremented every 2.5ms
    /// 
    /// This value is clamped to \[0, 4095\]
    pub fn with_timeout(mut self, count: u16) -> Self {
        self.config.auto_low_pow0 = self.config.auto_low_pow0.with_auto_lp_timeout_msb(count);
        self.config.auto_low_pow1 = self.config.auto_low_pow1.with_auto_lp_timeout_lsb(count);
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
}