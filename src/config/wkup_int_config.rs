use crate::{
    interface::WriteToRegister,
    registers::{
        WakeupIntConfig0,
        WakeupIntConfig1,
        WakeupIntConfig2,
        WakeupIntConfig3,
        WakeupIntConfig4,
    },
    ConfigError,
    WakeupIntRefMode,
    BMA400,
};

#[derive(Clone, Default)]
pub struct WakeupIntConfig {
    wkup_int_config0: WakeupIntConfig0,
    wkup_int_config1: WakeupIntConfig1,
    wkup_int_config2: WakeupIntConfig2,
    wkup_int_config3: WakeupIntConfig3,
    wkup_int_config4: WakeupIntConfig4,
}

impl WakeupIntConfig {
    pub fn is_int_en(&self) -> bool {
        self.wkup_int_config0.wkup_int_en()
    }
    pub fn get_config0(&self) -> WakeupIntConfig0 {
        self.wkup_int_config0
    }
}

/// Configure Wake-up Interrupt settings
/// 
/// - [WakeupIntRefMode] using [`with_ref_mode()`](WakeupIntConfigBuilder::with_ref_mode)
/// - Set the number of consecutive samples that must satisfy the condition before the interrupt is triggered using [`with_num_samples()`](WakeupIntConfigBuilder::with_num_samples)
/// - Enable / Disable axes to be evaluated against the condition using [`with_axes()`](WakeupIntConfigBuilder::with_axes)
/// - Set the interrupt trigger threshold using [`with_threshold()`](WakeupIntConfigBuilder::with_threshold)
/// - Set the reference acceleration using [`with_ref_accel()`](WakeupIntConfigBuilder::with_ref_accel)
pub struct WakeupIntConfigBuilder<'a, Interface: WriteToRegister> {
    config: WakeupIntConfig,
    device: &'a mut BMA400<Interface>,
}

impl<'a, Interface, E> WakeupIntConfigBuilder<'a, Interface>
where
    Interface: WriteToRegister<Error = E>,
    E: From<ConfigError>,
{
    pub(crate) fn new(device: &'a mut BMA400<Interface>) -> WakeupIntConfigBuilder<'a, Interface> {
        WakeupIntConfigBuilder {
            config: device.config.wkup_int_config.clone(),
            device,
        }
    }
    // WkupIntConfig0
    /// Set Reference mode for the Wake-up Interrupt
    pub fn with_ref_mode(mut self, mode: WakeupIntRefMode) -> Self {
        self.config.wkup_int_config0 = self.config.wkup_int_config0.with_reference_mode(mode);
        self
    }
    /// Number of consecutive samples that must exceed reference acceleration + / - threshold before
    /// interrupt is triggered.
    ///
    /// This value is clamped to \[1, 8\]
    pub fn with_num_samples(mut self, num_samples: u8) -> Self {
        self.config.wkup_int_config0 =
            self.config.wkup_int_config0.with_num_samples(num_samples.clamp(1, 8) - 1);
        self
    }
    /// Select the axes to be used in evaluating the wake-up interrupt condition ()
    pub fn with_axes(mut self, x_en: bool, y_en: bool, z_en: bool) -> Self {
        self.config.wkup_int_config0 =
            self.config.wkup_int_config0.with_x_axis(x_en).with_y_axis(y_en).with_z_axis(z_en);
        self
    }
    // WkupIntConfig1

    /// Set the amount by which the measured acceleration must exceed the reference acceleration
    /// before the interrupt is triggered.
    ///
    /// This threshold has unsigned 8-bit resolution corresponding to the upper 8 bits of a 12bit
    /// acceleration (<< 4).
    ///
    /// The evaluated condition is abs(measured - reference) > (threshold << 4) for _any_ enabled
    /// axis (logic OR).
    pub fn with_threshold(mut self, threshold: u8) -> Self {
        self.config.wkup_int_config1 = self.config.wkup_int_config1.with_threshold(threshold);
        self
    }

    // WkupIntConfig2 / WkupIntConfig3 / WkupIntConfig4

    /// Manually set the reference acceleration for the x,y,z axes (use with
    /// [WakeupIntRefMode::Manual])
    ///
    /// these values have signed 8-bit resolution corresponding to the upper 8 bits of a 12-bit
    /// acceleration (<< 4)
    ///
    /// In order for an axis to be evaluated it must be enabled using `with_axes()`
    pub fn with_ref_accel(mut self, x_ref: i8, y_ref: i8, z_ref: i8) -> Self {
        self.config.wkup_int_config2 =
            self.config.wkup_int_config2.with_x_ref(x_ref.to_le_bytes()[0]);
        self.config.wkup_int_config3 =
            self.config.wkup_int_config3.with_y_ref(y_ref.to_le_bytes()[0]);
        self.config.wkup_int_config4 =
            self.config.wkup_int_config4.with_z_ref(z_ref.to_le_bytes()[0]);
        self
    }
    /// Write this configuration to device registers
    pub fn write(self) -> Result<(), E> {
        let has_wkup_config0_changes = self.device.config.wkup_int_config.wkup_int_config0.bits()
            != self.config.wkup_int_config0.bits();
        let has_wkup_config1_changes = self.device.config.wkup_int_config.wkup_int_config1.bits()
            != self.config.wkup_int_config1.bits();
        let has_wkup_config2_changes = self.device.config.wkup_int_config.wkup_int_config2.bits()
            != self.config.wkup_int_config2.bits();
        let has_wkup_config3_changes = self.device.config.wkup_int_config.wkup_int_config3.bits()
            != self.config.wkup_int_config3.bits();
        let has_wkup_config4_changes = self.device.config.wkup_int_config.wkup_int_config4.bits()
            != self.config.wkup_int_config4.bits();
        let has_wkup_config_changes = has_wkup_config0_changes
            || has_wkup_config1_changes
            || has_wkup_config2_changes
            || has_wkup_config3_changes
            || has_wkup_config4_changes;

        // Disable the interrupt
        if self.device.config.wkup_int_config.is_int_en() && has_wkup_config_changes {
            self.device.interface.write_register(
                self.device
                    .config
                    .wkup_int_config
                    .wkup_int_config0
                    .with_x_axis(false)
                    .with_y_axis(false)
                    .with_z_axis(false),
            )?;
        }
        // Write the config changes
        if self.device.config.wkup_int_config.wkup_int_config1.bits()
            != self.config.wkup_int_config1.bits()
        {
            self.device.interface.write_register(self.config.wkup_int_config1)?;
            self.device.config.wkup_int_config.wkup_int_config1 = self.config.wkup_int_config1;
        }
        if self.device.config.wkup_int_config.wkup_int_config2.bits()
            != self.config.wkup_int_config2.bits()
        {
            self.device.interface.write_register(self.config.wkup_int_config2)?;
            self.device.config.wkup_int_config.wkup_int_config2 = self.config.wkup_int_config2;
        }
        if self.device.config.wkup_int_config.wkup_int_config3.bits()
            != self.config.wkup_int_config3.bits()
        {
            self.device.interface.write_register(self.config.wkup_int_config3)?;
            self.device.config.wkup_int_config.wkup_int_config3 = self.config.wkup_int_config3;
        }
        if self.device.config.wkup_int_config.wkup_int_config4.bits()
            != self.config.wkup_int_config4.bits()
        {
            self.device.interface.write_register(self.config.wkup_int_config4)?;
            self.device.config.wkup_int_config.wkup_int_config4 = self.config.wkup_int_config4;
        }
        // (Re)-enable the interrupt
        if self.device.config.wkup_int_config.wkup_int_config0.bits()
            != self.config.wkup_int_config0.bits()
        {
            self.device.interface.write_register(self.config.wkup_int_config0)?;
            self.device.config.wkup_int_config.wkup_int_config0 = self.config.wkup_int_config0;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::get_test_device;
    #[test]
    fn test_ref_mode() {
        let mut device = get_test_device();
        let builder = device.config_wkup_int();
        let builder = builder.with_ref_mode(WakeupIntRefMode::OneTime);
        assert_eq!(builder.config.wkup_int_config0.bits(), 0x01);
        let builder = builder.with_ref_mode(WakeupIntRefMode::EveryTime);
        assert_eq!(builder.config.wkup_int_config0.bits(), 0x02);
        let builder = builder.with_ref_mode(WakeupIntRefMode::Manual);
        assert_eq!(builder.config.wkup_int_config0.bits(), 0x00);
    }
    #[test]
    fn test_num_samples() {
        let mut device = get_test_device();
        let builder = device.config_wkup_int();
        let builder = builder.with_num_samples(9);
        assert_eq!(builder.config.wkup_int_config0.bits(), 0x1C);
        let builder = builder.with_num_samples(0);
        assert_eq!(builder.config.wkup_int_config0.bits(), 0x00);
    }
    #[test]
    fn test_axes() {
        let mut device = get_test_device();
        let builder = device.config_wkup_int();
        let builder = builder.with_axes(false, false, true);
        assert_eq!(builder.config.wkup_int_config0.bits(), 0x80);
        let builder = builder.with_axes(false, true, false);
        assert_eq!(builder.config.wkup_int_config0.bits(), 0x40);
        let builder = builder.with_axes(true, false, false);
        assert_eq!(builder.config.wkup_int_config0.bits(), 0x20);
    }
    #[test]
    fn test_threshold() {
        let mut device = get_test_device();
        let builder = device.config_wkup_int();
        let builder = builder.with_threshold(255);
        assert_eq!(builder.config.wkup_int_config1.bits(), 0xFF);
        let builder = builder.with_threshold(0);
        assert_eq!(builder.config.wkup_int_config1.bits(), 0x00);
    }
    #[test]
    fn test_ref_accel() {
        let mut device = get_test_device();
        let builder = device.config_wkup_int();
        let builder = builder.with_ref_accel(-128, 127, 1);
        assert_eq!(builder.config.wkup_int_config2.bits(), 0x80);
        assert_eq!(builder.config.wkup_int_config3.bits(), 0x7F);
        assert_eq!(builder.config.wkup_int_config4.bits(), 0x01);
        let builder = builder.with_ref_accel(127, -1, -2);
        assert_eq!(builder.config.wkup_int_config2.bits(), 0x7F);
        assert_eq!(builder.config.wkup_int_config3.bits(), 0xFF);
        assert_eq!(builder.config.wkup_int_config4.bits(), 0xFE);
    }
}
