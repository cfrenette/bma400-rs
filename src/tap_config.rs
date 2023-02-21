use core::fmt::Debug;
use crate::{registers::{TapConfig0, TapConfig1}, interface::WriteToRegister, BMA400, ConfigError, Axis, TapSensitivity, MinTapDuration, DoubleTapDuration, MaxTapDuration};

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
    // TapConfig0

    /// Select axis to use when evaluating interrupt
    pub fn with_axis(mut self, axis: Axis) -> Self {
        self.config.tap_config0 = self.config.tap_config0.with_axis(axis);
        self
    }
    /// Select the sensitivity level
    pub fn with_sensitivity(mut self, sensitivity: TapSensitivity) -> Self {
        self.config.tap_config0 = self.config.tap_config0.with_sensitivity(sensitivity);
        self
    }

    // TapConfig1

    /// Select the minimum number of samples that must elapse between two peaks for it to be considered as a separate tap
    pub fn with_min_duration_btn_taps(mut self, duration: MinTapDuration) -> Self {
        self.config.tap_config1 = self.config.tap_config1.with_min_tap_duration(duration);
        self
    }
    /// Select the maximum number of samples that can elapse between two peaks for it to be considered as a double tap
    pub fn with_max_double_tap_window(mut self, duration: DoubleTapDuration) -> Self {
        self.config.tap_config1 = self.config.tap_config1.with_double_tap_duration(duration);
        self
    }
    /// Select the maximuim number of samples that can elapse between the high and low peak of a tap for it to be considered a tap
    pub fn with_max_tap_duration(mut self, duration: MaxTapDuration) -> Self {
        self.config.tap_config1 = self.config.tap_config1.with_max_tap_duration(duration);
        self
    }

    pub fn write(self) -> Result<(), E> {

        let tap1_changes = self.device.config.tap_config.tap_config0.bits() != self.config.tap_config0.bits();
        let tap2_changes = self.device.config.tap_config.tap_config1.bits() != self.config.tap_config1.bits();
        let tap_changes = tap1_changes || tap2_changes;
        let mut tmp_int_config = self.device.config.int_config.get_config1().clone();

        // Disable the interrupt, if active
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
        // Re-enable the interrupt, if disabled
        if self.device.config.int_config.get_config1().bits() != tmp_int_config.bits() {
            self.device.interface.write_register(self.device.config.int_config.get_config1())?;
        }
        Ok(())
    }
}