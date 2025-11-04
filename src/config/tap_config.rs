use crate::{
    Axis, BMA400, ConfigError, DoubleTapDuration, MaxTapDuration, MinTapDuration, TapSensitivity,
    interface::WriteToRegister,
    registers::{TapConfig0, TapConfig1},
};

#[derive(Clone, Default)]
pub struct TapConfig {
    tap_config0: TapConfig0,
    tap_config1: TapConfig1,
}

/// Configure Advanced Tap Interrupt Settings
///
/// - Set the axis evaluated for the interrupt trigger condition using [`with_axis()`](TapConfigBuilder::with_axis)
/// - [TapSensitivity] using [`with_sensitivity()`](TapConfigBuilder::with_sensitivity)
/// - [MinTapDuration] using [`with_min_duration_btn_taps()`](TapConfigBuilder::with_min_duration_btn_taps)
/// - [DoubleTapDuration] using [`with_max_double_tap_window()`](TapConfigBuilder::with_max_double_tap_window)
/// - [MaxTapDuration] using [`with_max_tap_duration()`](TapConfigBuilder::with_max_tap_duration)
pub struct TapConfigBuilder<'a, Interface: WriteToRegister> {
    config: TapConfig,
    device: &'a mut BMA400<Interface>,
}

impl<'a, Interface, E> TapConfigBuilder<'a, Interface>
where
    Interface: WriteToRegister<Error = E>,
    E: From<ConfigError>,
{
    pub(crate) fn new(device: &mut BMA400<Interface>) -> TapConfigBuilder<Interface> {
        TapConfigBuilder {
            config: device.config.tap_config.clone(),
            device,
        }
    }
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

    /// Select the minimum number of samples that must elapse between two peaks for it to be
    /// considered as a separate tap
    pub fn with_min_duration_btn_taps(mut self, duration: MinTapDuration) -> Self {
        self.config.tap_config1 = self.config.tap_config1.with_min_tap_duration(duration);
        self
    }
    /// Select the maximum number of samples that can elapse between two peaks for it to be
    /// considered as a double tap
    pub fn with_max_double_tap_window(mut self, duration: DoubleTapDuration) -> Self {
        self.config.tap_config1 = self.config.tap_config1.with_double_tap_duration(duration);
        self
    }
    /// Select the maximuim number of samples that can elapse between the high and low peak of a tap
    /// for it to be considered a tap
    pub fn with_max_tap_duration(mut self, duration: MaxTapDuration) -> Self {
        self.config.tap_config1 = self.config.tap_config1.with_max_tap_duration(duration);
        self
    }
    /// Write this configuration to device registers
    pub fn write(self) -> Result<(), E> {
        let tap1_changes =
            self.device.config.tap_config.tap_config0.bits() != self.config.tap_config0.bits();
        let tap2_changes =
            self.device.config.tap_config.tap_config1.bits() != self.config.tap_config1.bits();
        let tap_changes = tap1_changes || tap2_changes;
        let mut tmp_int_config = self.device.config.int_config.get_config1();

        // Disable the interrupt, if active
        if (self.device.config.int_config.get_config1().d_tap_int()
            || self.device.config.int_config.get_config1().d_tap_int())
            && tap_changes
        {
            tmp_int_config = tmp_int_config.with_s_tap_int(false).with_d_tap_int(false);
            self.device.interface.write_register(tmp_int_config)?;
        }
        if tap1_changes {
            self.device
                .interface
                .write_register(self.config.tap_config0)?;
            self.device.config.tap_config.tap_config0 = self.config.tap_config0;
        }
        if tap2_changes {
            self.device
                .interface
                .write_register(self.config.tap_config1)?;
            self.device.config.tap_config.tap_config1 = self.config.tap_config1;
        }
        // Re-enable the interrupt, if disabled
        if self.device.config.int_config.get_config1().bits() != tmp_int_config.bits() {
            self.device
                .interface
                .write_register(self.device.config.int_config.get_config1())?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::get_test_device;
    #[test]
    fn test_axis() {
        let mut device = get_test_device();
        let builder = device.config_tap();
        let builder = builder.with_axis(Axis::Y);
        assert_eq!(builder.config.tap_config0.bits(), 0x08);
        let builder = builder.with_axis(Axis::X);
        assert_eq!(builder.config.tap_config0.bits(), 0x10);
        let builder = builder.with_axis(Axis::Z);
        assert_eq!(builder.config.tap_config0.bits(), 0x00);
    }
    #[test]
    fn test_sensitivity() {
        let mut device = get_test_device();
        let builder = device.config_tap();
        let builder = builder.with_sensitivity(TapSensitivity::SENS1);
        assert_eq!(builder.config.tap_config0.bits(), 0x01);
        let builder = builder.with_sensitivity(TapSensitivity::SENS2);
        assert_eq!(builder.config.tap_config0.bits(), 0x02);
        let builder = builder.with_sensitivity(TapSensitivity::SENS3);
        assert_eq!(builder.config.tap_config0.bits(), 0x03);
        let builder = builder.with_sensitivity(TapSensitivity::SENS4);
        assert_eq!(builder.config.tap_config0.bits(), 0x04);
        let builder = builder.with_sensitivity(TapSensitivity::SENS5);
        assert_eq!(builder.config.tap_config0.bits(), 0x05);
        let builder = builder.with_sensitivity(TapSensitivity::SENS6);
        assert_eq!(builder.config.tap_config0.bits(), 0x06);
        let builder = builder.with_sensitivity(TapSensitivity::SENS7);
        assert_eq!(builder.config.tap_config0.bits(), 0x07);
        let builder = builder.with_sensitivity(TapSensitivity::SENS0);
        assert_eq!(builder.config.tap_config0.bits(), 0x00);
    }
    #[test]
    fn test_min_duration() {
        let mut device = get_test_device();
        let builder = device.config_tap();
        let builder = builder.with_min_duration_btn_taps(MinTapDuration::Samples8);
        assert_eq!(builder.config.tap_config1.bits(), 0x16);
        let builder = builder.with_min_duration_btn_taps(MinTapDuration::Samples12);
        assert_eq!(builder.config.tap_config1.bits(), 0x26);
        let builder = builder.with_min_duration_btn_taps(MinTapDuration::Samples16);
        assert_eq!(builder.config.tap_config1.bits(), 0x36);
        let builder = builder.with_min_duration_btn_taps(MinTapDuration::Samples4);
        assert_eq!(builder.config.tap_config1.bits(), 0x06);
    }
    #[test]
    fn test_double_tap_duration() {
        let mut device = get_test_device();
        let builder = device.config_tap();
        let builder = builder.with_max_double_tap_window(DoubleTapDuration::Samples80);
        assert_eq!(builder.config.tap_config1.bits(), 0x06);
        let builder = builder.with_max_double_tap_window(DoubleTapDuration::Samples100);
        assert_eq!(builder.config.tap_config1.bits(), 0x0A);
        let builder = builder.with_max_double_tap_window(DoubleTapDuration::Samples120);
        assert_eq!(builder.config.tap_config1.bits(), 0x0E);
        let builder = builder.with_max_double_tap_window(DoubleTapDuration::Samples60);
        assert_eq!(builder.config.tap_config1.bits(), 0x02);
    }
    #[test]
    fn test_max_tap_duration() {
        let mut device = get_test_device();
        let builder = device.config_tap();
        let builder = builder.with_max_tap_duration(MaxTapDuration::Samples9);
        assert_eq!(builder.config.tap_config1.bits(), 0x05);
        let builder = builder.with_max_tap_duration(MaxTapDuration::Samples12);
        assert_eq!(builder.config.tap_config1.bits(), 0x06);
        let builder = builder.with_max_tap_duration(MaxTapDuration::Samples18);
        assert_eq!(builder.config.tap_config1.bits(), 0x07);
        let builder = builder.with_max_tap_duration(MaxTapDuration::Samples6);
        assert_eq!(builder.config.tap_config1.bits(), 0x04);
    }
}
