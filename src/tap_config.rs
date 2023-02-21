use core::fmt::Debug;
use crate::{registers::{TapConfig0, TapConfig1}, interface::WriteToRegister, BMA400, ConfigError};

#[derive(Clone, Default)]
pub struct TapConfig {
    tap_config0: TapConfig0,
    tap_config1: TapConfig1,
}

pub struct TapConfigBuilder<'a, Interface: WriteToRegister> 
{
    config: TapConfig,
    device: &'a mut BMA400<Interface>,
}

impl<'a, Interface, E> TapConfigBuilder<'a, Interface> 
where
    Interface: WriteToRegister<Error = E>,
    E: From<ConfigError> + Debug,
{
    pub fn write(self) -> Result<(), E> {
        let tap1_changes = self.device.config.tap_config.tap_config0.bits() != self.config.tap_config0.bits();
        let tap2_changes = self.device.config.tap_config.tap_config1.bits() != self.config.tap_config1.bits();
        let tap_changes = tap1_changes || tap2_changes;
        let mut tmp_int_config = self.device.config.int_config.get_config1().clone();

        // If enabled, temporarily disable the FIFO Watermark Interrupt to change the config
        if (self.device.config.int_config.get_config1().d_tap_int() || self.device.config.int_config.get_config1().d_tap_int()) && tap_changes {
            tmp_int_config = tmp_int_config.with_s_tap_int(false).with_d_tap_int(false);
            self.device.interface.write_register(tmp_int_config)?;
        }
        if tap1_changes {
            self.device.interface.write_register(self.config.tap_config0)?;
            self.device.config.tap_config.tap_config0 = self.config.tap_config0;
        }
        if tap2_changes {
            self.device.interface.write_register(self.config.tap_config1)?;
            self.device.config.tap_config.tap_config1 = self.config.tap_config1;
        }
        // Re-enable the interrupt if it was changed
        if self.device.config.int_config.get_config1().bits() != tmp_int_config.bits() {
            self.device.interface.write_register(self.device.config.int_config.get_config1())?;
        }
        Ok(())
    }
}