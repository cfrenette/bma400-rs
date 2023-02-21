use crate::{
    Debug,
    registers::{AutoWakeup0, AutoWakeup1},
    interface::WriteToRegister,
    BMA400,
    ConfigError,
};

#[derive(Clone, Default)]
pub struct AutoWakeupConfig {
    auto_wakeup0: AutoWakeup0,
    auto_wakeup1: AutoWakeup1,
}

pub struct AutoWakeupConfigBuilder<'a, Interface> {
    config: AutoWakeupConfig,
    device: &'a mut BMA400<Interface>,
}

impl<'a, Interface, E> AutoWakeupConfigBuilder<'a, Interface> 
where 
    Interface: WriteToRegister<Error = E>,
    E: From<ConfigError> + Debug,
{
    pub fn new(config: AutoWakeupConfig, device: &'a mut BMA400<Interface>) -> AutoWakeupConfigBuilder<'a, Interface> {
        AutoWakeupConfigBuilder { config, device }
    }

    /// Set the timer counter for periodic auto wake-up. This value is 12-bits and is incremented every 2.5ms
    pub fn with_wakeup_period(mut self, count: u16) -> Self {
        let timeout = count.clamp(1, 4096) - 1;
        self.config.auto_wakeup0 = self.config.auto_wakeup0.with_wakeup_timeout_msb(timeout);
        self.config.auto_wakeup1 = self.config.auto_wakeup1.with_wakeup_timeout_lsb(timeout);
        self
    }
    /// Enable/Disable periodic auto wake-up
    pub fn with_periodic_wakeup(mut self, enabled: bool) -> Self {
        self.config.auto_wakeup1 = self.config.auto_wakeup1.with_wakeup_timeout(enabled);
        self
    }
    /// Enable/Disable activity wake-up interrupt
    pub fn with_activity_int(mut self, enabled: bool) -> Self {
        self.config.auto_wakeup1 = self.config.auto_wakeup1.with_wakeup_int(enabled);
        self
    }
    pub fn write(self) -> Result<(), E> {
        if self.device.config.auto_wkup_config.auto_wakeup0.bits() != self.config.auto_wakeup0.bits() {
            self.device.interface.write_register(self.config.auto_wakeup0)?;
            self.device.config.auto_wkup_config.auto_wakeup0 = self.config.auto_wakeup0;
        }
        if self.device.config.auto_wkup_config.auto_wakeup1.bits() != self.config.auto_wakeup1.bits() {
            self.device.interface.write_register(self.config.auto_wakeup1)?;
            self.device.config.auto_wkup_config.auto_wakeup1 = self.config.auto_wakeup1;
        }
        Ok(())
    }
}