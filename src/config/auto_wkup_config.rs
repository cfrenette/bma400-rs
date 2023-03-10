use crate::{
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

impl AutoWakeupConfig {
    pub fn get_config1(&self) -> AutoWakeup1 {
        self.auto_wakeup1
    }
}

pub struct AutoWakeupConfigBuilder<'a, Interface> {
    config: AutoWakeupConfig,
    device: &'a mut BMA400<Interface>,
}

impl<'a, Interface, E> AutoWakeupConfigBuilder<'a, Interface> 
where 
    Interface: WriteToRegister<Error = E>,
    E: From<ConfigError>,
{
    pub(crate) fn new(device: &'a mut BMA400<Interface>) -> AutoWakeupConfigBuilder<'a, Interface> {
        AutoWakeupConfigBuilder { config: device.config.auto_wkup_config.clone(), device }
    }

    /// Set the timer counter threshold for periodic auto wake-up. The counter is 12-bits and is incremented every 2.5ms,
    /// so this value is clamped to \[0, 4095\]
    pub fn with_wakeup_period(mut self, count: u16) -> Self {
        let timeout = count.clamp(0, 4095);
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

#[cfg(test)]
mod tests {
    use super::*;
    use embedded_hal_mock::i2c::{Mock, Transaction};
    use crate::{
        i2c::I2CInterface,
    };
    const ADDR: u8 = crate::i2c::ADDR;
    fn device_no_write() -> BMA400<I2CInterface<Mock>> {
        let expected = [
            Transaction::write_read(ADDR, [0x00].into_iter().collect(), [0x90].into_iter().collect())
        ];
        BMA400::new_i2c(Mock::new(&expected)).unwrap()
    }
    #[test]
    fn test_wakeup_period() {
        let mut device = device_no_write();
        let builder = device.config_autowkup();
        let builder = builder.with_wakeup_period(4097);
        assert_eq!(builder.config.auto_wakeup0.bits(), 0xFF);
        assert_eq!(builder.config.auto_wakeup1.bits(), 0xF0);
        let builder = builder.with_wakeup_period(0);
        assert_eq!(builder.config.auto_wakeup0.bits(), 0x00);
        assert_eq!(builder.config.auto_wakeup1.bits(), 0x00);
    }
    #[test]
    fn test_periodic_wakeup() {
        let mut device = device_no_write();
        let builder = device.config_autowkup();
        let builder = builder.with_periodic_wakeup(true);
        assert_eq!(builder.config.auto_wakeup1.bits(), 0x04);
        let builder = builder.with_periodic_wakeup(false);
        assert_eq!(builder.config.auto_wakeup1.bits(), 0x00);
    }
    #[test]
    fn test_activity_int() {
        let mut device = device_no_write();
        let builder = device.config_autowkup();
        let builder = builder.with_activity_int(true);
        assert_eq!(builder.config.auto_wakeup1.bits(), 0x02);
        let builder = builder.with_activity_int(false);
        assert_eq!(builder.config.auto_wakeup1.bits(), 0x00);
    }
}
