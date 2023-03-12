use crate::{
    config::Config,
    interface::WriteToRegister,
    registers::{
        Gen1IntConfig0,
        Gen1IntConfig1,
        Gen1IntConfig2,
        Gen1IntConfig3,
        Gen1IntConfig31,
        Gen1IntConfig4,
        Gen1IntConfig5,
        Gen1IntConfig6,
        Gen1IntConfig7,
        Gen1IntConfig8,
        Gen1IntConfig9,
        Gen2IntConfig0,
        Gen2IntConfig1,
        Gen2IntConfig2,
        Gen2IntConfig3,
        Gen2IntConfig31,
        Gen2IntConfig4,
        Gen2IntConfig5,
        Gen2IntConfig6,
        Gen2IntConfig7,
        Gen2IntConfig8,
        Gen2IntConfig9,
    },
    ConfigError,
    DataSource,
    GenIntCriterionMode,
    GenIntLogicMode,
    GenIntRefMode,
    Hysteresis,
    OutputDataRate,
    BMA400,
};

#[derive(Clone, Default)]
pub struct Gen1IntConfig {
    config0: Gen1IntConfig0,
    config1: Gen1IntConfig1,
    config2: Gen1IntConfig2,
    config3: Gen1IntConfig3,
    config31: Gen1IntConfig31,
    config4: Gen1IntConfig4,
    config5: Gen1IntConfig5,
    config6: Gen1IntConfig6,
    config7: Gen1IntConfig7,
    config8: Gen1IntConfig8,
    config9: Gen1IntConfig9,
}

impl Gen1IntConfig {
    pub fn src(&self) -> DataSource {
        self.config0.src()
    }
}

#[derive(Clone, Default)]
pub struct Gen2IntConfig {
    config0: Gen2IntConfig0,
    config1: Gen2IntConfig1,
    config2: Gen2IntConfig2,
    config3: Gen2IntConfig3,
    config31: Gen2IntConfig31,
    config4: Gen2IntConfig4,
    config5: Gen2IntConfig5,
    config6: Gen2IntConfig6,
    config7: Gen2IntConfig7,
    config8: Gen2IntConfig8,
    config9: Gen2IntConfig9,
}

impl Gen2IntConfig {
    pub fn src(&self) -> DataSource {
        self.config0.src()
    }
}

pub enum GenIntConfig {
    Gen1Int(Gen1IntConfig),
    Gen2Int(Gen2IntConfig),
}

impl GenIntConfig {
    pub fn src(&self) -> DataSource {
        match self {
            GenIntConfig::Gen1Int(config) => config.src(),
            GenIntConfig::Gen2Int(config) => config.src(),
        }
    }
}

pub struct GenIntConfigBuilder<'a, Interface: WriteToRegister> {
    config: GenIntConfig,
    device: &'a mut BMA400<Interface>,
}

impl<'a, Interface, E> GenIntConfigBuilder<'a, Interface>
where
    Interface: WriteToRegister<Error = E>,
    E: From<ConfigError>,
{
    pub(crate) fn new_gen1(device: &'a mut BMA400<Interface>) -> GenIntConfigBuilder<'a, Interface> {
        let config = GenIntConfig::Gen1Int(device.config.gen1int_config.clone());
        GenIntConfigBuilder {
            config,
            device,
        }
    }
    pub(crate) fn new_gen2(device: &'a mut BMA400<Interface>) -> GenIntConfigBuilder<'a, Interface> {
        let config = GenIntConfig::Gen2Int(device.config.gen2int_config.clone());
        GenIntConfigBuilder {
            config,
            device,
        }
    }
    // Config0
    /// Select the axes to be considered when evaluating the generic interrupt criterion
    pub fn with_axes(mut self, x: bool, y: bool, z: bool) -> Self {
        match &mut self.config {
            GenIntConfig::Gen1Int(config) => {
                config.config0 = 
                    config.config0
                        .with_x_axis(x)
                        .with_y_axis(y)
                        .with_z_axis(z)
            }
            GenIntConfig::Gen2Int(config) => {
                config.config0 = 
                    config.config0
                        .with_x_axis(x)
                        .with_y_axis(y)
                        .with_z_axis(z)
            }
        }
        self
    }
    /// Set the data source to use when evaluating the generic interrupt criterion
    ///
    /// Cannot use [DataSource::AccFilt2Lp]. If passed, this will default to [DataSource::AccFilt2]
    pub fn with_src(mut self, src: DataSource) -> Self {
        let src = match src {
            DataSource::AccFilt2Lp => DataSource::AccFilt2,
            _ => src,
        };
        match &mut self.config {
            GenIntConfig::Gen1Int(config) => config.config0 = config.config0.with_src(src),
            GenIntConfig::Gen2Int(config) => config.config0 = config.config0.with_src(src),
        }
        self
    }
    /// Set the reference acceleration update mode for the generic interrupt
    pub fn with_reference_mode(mut self, mode: GenIntRefMode) -> Self {
        match &mut self.config {
            GenIntConfig::Gen1Int(config) => config.config0 = config.config0.with_refu_mode(mode),
            GenIntConfig::Gen2Int(config) => config.config0 = config.config0.with_refu_mode(mode),
        }
        self
    }
    /// Set the amplitude of the hysteresis adjustment to the interrupt criteria
    pub fn with_hysteresis(mut self, hysteresis: Hysteresis) -> Self {
        match &mut self.config {
            GenIntConfig::Gen1Int(config) => {
                config.config0 = config.config0.with_act_hysteresis(hysteresis)
            }
            GenIntConfig::Gen2Int(config) => {
                config.config0 = config.config0.with_act_hysteresis(hysteresis)
            }
        }
        self
    }
    // Config1
    /// Set the interrupt trigger condition (on Activity or Inactivity)
    pub fn with_criterion_mode(mut self, mode: GenIntCriterionMode) -> Self {
        match &mut self.config {
            GenIntConfig::Gen1Int(config) => {
                config.config1 = config.config1.with_criterion_sel(mode)
            }
            GenIntConfig::Gen2Int(config) => {
                config.config1 = config.config1.with_criterion_sel(mode)
            }
        }
        self
    }
    /// Set the interrupt trigger behavior when multiple axes are selected
    pub fn with_logic_mode(mut self, mode: GenIntLogicMode) -> Self {
        match &mut self.config {
            GenIntConfig::Gen1Int(config) => config.config1 = config.config1.with_comb_sel(mode),
            GenIntConfig::Gen2Int(config) => config.config1 = config.config1.with_comb_sel(mode),
        }
        self
    }
    // Config2
    /// Set the threshold above or below reference acceleration at which the interrupt criterion
    /// evaluates to true
    ///
    /// This is not adjusted by scale, and is compared against the 8 msb of the acceleration (8
    /// milli-g resolution)
    pub fn with_threshold(mut self, threshold: u8) -> Self {
        match &mut self.config {
            GenIntConfig::Gen1Int(config) => {
                config.config2 = config.config2.with_threshold(threshold)
            }
            GenIntConfig::Gen2Int(config) => {
                config.config2 = config.config2.with_threshold(threshold)
            }
        }
        self
    }
    // Config3 and Config31
    /// Set the number of cycles that the interrupt criterion must evaluate to true before the
    /// interrupt triggers
    ///
    /// Note that the actual time duration depends on the ODR of the [DataSource] used
    pub fn with_duration(mut self, duration: u16) -> Self {
        match &mut self.config {
            GenIntConfig::Gen1Int(config) => {
                config.config3 = config.config3.with_duration_msb(duration.to_le_bytes()[1]);
                config.config31 = config.config31.with_duration_lsb(duration.to_le_bytes()[0]);
            }
            GenIntConfig::Gen2Int(config) => {
                config.config3 = config.config3.with_duration_msb(duration.to_le_bytes()[1]);
                config.config31 = config.config31.with_duration_lsb(duration.to_le_bytes()[0]);
            }
        }
        self
    }
    // Config4-9
    /// Manually set the reference acceleration for the interrupt criterion. This is
    /// automatically overwritten if [`GenIntRefMode::Manual`] is not set.
    ///
    /// 12-bit, clamped to \[-2048, 2047\] and scales with [crate::Scale]
    pub fn with_ref_accel(mut self, ref_x: i16, ref_y: i16, ref_z: i16) -> Self {
        let (ref_x, ref_y, ref_z) =
            (ref_x.clamp(-2048, 2047), ref_y.clamp(-2048, 2047), ref_z.clamp(-2048, 2047));
        match &mut self.config {
            GenIntConfig::Gen1Int(config) => {
                config.config4 = config.config4.with_ref_x_lsb(ref_x.to_le_bytes()[0]);
                config.config5 = config.config5.with_ref_x_msb(ref_x.to_le_bytes()[1]);
                config.config6 = config.config6.with_ref_y_lsb(ref_y.to_le_bytes()[0]);
                config.config7 = config.config7.with_ref_y_msb(ref_y.to_le_bytes()[1]);
                config.config8 = config.config8.with_ref_z_lsb(ref_z.to_le_bytes()[0]);
                config.config9 = config.config9.with_ref_z_msb(ref_z.to_le_bytes()[1]);
            }
            GenIntConfig::Gen2Int(config) => {
                config.config4 = config.config4.with_ref_x_lsb(ref_x.to_le_bytes()[0]);
                config.config5 = config.config5.with_ref_x_msb(ref_x.to_le_bytes()[1]);
                config.config6 = config.config6.with_ref_y_lsb(ref_y.to_le_bytes()[0]);
                config.config7 = config.config7.with_ref_y_msb(ref_y.to_le_bytes()[1]);
                config.config8 = config.config8.with_ref_z_lsb(ref_z.to_le_bytes()[0]);
                config.config9 = config.config9.with_ref_z_msb(ref_z.to_le_bytes()[1]);
            }
        }
        self
    }
    pub fn write(self) -> Result<(), E> {
        let has_config0_changes = self.has_config0_changes_from(&self.device.config);
        let has_config1_changes = self.has_config1_changes_from(&self.device.config);
        let has_config2_changes = self.has_config2_changes_from(&self.device.config);
        let has_config3_changes = self.has_config3_changes_from(&self.device.config);
        let has_config31_changes = self.has_config31_changes_from(&self.device.config);
        let has_config4_changes = self.has_config4_changes_from(&self.device.config);
        let has_config5_changes = self.has_config5_changes_from(&self.device.config);
        let has_config6_changes = self.has_config6_changes_from(&self.device.config);
        let has_config7_changes = self.has_config7_changes_from(&self.device.config);
        let has_config8_changes = self.has_config8_changes_from(&self.device.config);
        let has_config9_changes = self.has_config9_changes_from(&self.device.config);

        let has_changes = has_config0_changes
            || has_config1_changes
            || has_config2_changes
            || has_config3_changes
            || has_config31_changes
            || has_config4_changes
            || has_config5_changes
            || has_config6_changes
            || has_config7_changes
            || has_config8_changes
            || has_config9_changes;

        // If there aren't any changes, return early
        if !has_changes {
            return Ok(());
        }
        // Clone the existing enabled interrupts
        let mut int_config0 = self.device.config.int_config.get_config0();
        let int_enabled = match &self.config {
            GenIntConfig::Gen1Int(_) => int_config0.gen1_int(),
            GenIntConfig::Gen2Int(_) => int_config0.gen2_int(),
        };
        // If the interrupt is enabled and we're changing the data source to AccFilt1 the ODR must
        // be 100Hz
        if int_enabled
            && !matches!(self.device.config.acc_config.odr(), OutputDataRate::Hz100)
            && matches!(self.config.src(), DataSource::AccFilt1)
        {
            return Err(ConfigError::Filt1InterruptInvalidODR.into());
        }
        // If there are changes and the interrupt is active, need to disable interrupt before
        // writing changes
        match &self.config {
            GenIntConfig::Gen1Int(_) => {
                if int_enabled {
                    int_config0 = int_config0.with_gen1_int(false);
                    self.device.interface.write_register(int_config0)?;
                }
            }
            GenIntConfig::Gen2Int(_) => {
                if int_enabled {
                    int_config0 = int_config0.with_gen2_int(false);
                    self.device.interface.write_register(int_config0)?;
                }
            }
        }
        if has_config0_changes {
            match &self.config {
                GenIntConfig::Gen1Int(config) => {
                    self.device.interface.write_register(config.config0)?;
                    self.device.config.gen1int_config.config0 = config.config0;
                }
                GenIntConfig::Gen2Int(config) => {
                    self.device.interface.write_register(config.config0)?;
                    self.device.config.gen2int_config.config0 = config.config0;
                }
            }
        }
        if has_config1_changes {
            match &self.config {
                GenIntConfig::Gen1Int(config) => {
                    self.device.interface.write_register(config.config1)?;
                    self.device.config.gen1int_config.config1 = config.config1;
                }
                GenIntConfig::Gen2Int(config) => {
                    self.device.interface.write_register(config.config1)?;
                    self.device.config.gen2int_config.config1 = config.config1;
                }
            }
        }
        if has_config2_changes {
            match &self.config {
                GenIntConfig::Gen1Int(config) => {
                    self.device.interface.write_register(config.config2)?;
                    self.device.config.gen1int_config.config2 = config.config2;
                }
                GenIntConfig::Gen2Int(config) => {
                    self.device.interface.write_register(config.config2)?;
                    self.device.config.gen2int_config.config2 = config.config2;
                }
            }
        }
        if has_config3_changes {
            match &self.config {
                GenIntConfig::Gen1Int(config) => {
                    self.device.interface.write_register(config.config3)?;
                    self.device.config.gen1int_config.config3 = config.config3;
                }
                GenIntConfig::Gen2Int(config) => {
                    self.device.interface.write_register(config.config3)?;
                    self.device.config.gen2int_config.config3 = config.config3;
                }
            }
        }
        if has_config31_changes {
            match &self.config {
                GenIntConfig::Gen1Int(config) => {
                    self.device.interface.write_register(config.config31)?;
                    self.device.config.gen1int_config.config31 = config.config31;
                }
                GenIntConfig::Gen2Int(config) => {
                    self.device.interface.write_register(config.config31)?;
                    self.device.config.gen2int_config.config31 = config.config31;
                }
            }
        }
        if has_config4_changes {
            match &self.config {
                GenIntConfig::Gen1Int(config) => {
                    self.device.interface.write_register(config.config4)?;
                    self.device.config.gen1int_config.config4 = config.config4;
                }
                GenIntConfig::Gen2Int(config) => {
                    self.device.interface.write_register(config.config4)?;
                    self.device.config.gen2int_config.config4 = config.config4;
                }
            }
        }
        if has_config5_changes {
            match &self.config {
                GenIntConfig::Gen1Int(config) => {
                    self.device.interface.write_register(config.config5)?;
                    self.device.config.gen1int_config.config5 = config.config5;
                }
                GenIntConfig::Gen2Int(config) => {
                    self.device.interface.write_register(config.config5)?;
                    self.device.config.gen2int_config.config5 = config.config5;
                }
            }
        }
        if has_config6_changes {
            match &self.config {
                GenIntConfig::Gen1Int(config) => {
                    self.device.interface.write_register(config.config6)?;
                    self.device.config.gen1int_config.config6 = config.config6;
                }
                GenIntConfig::Gen2Int(config) => {
                    self.device.interface.write_register(config.config6)?;
                    self.device.config.gen2int_config.config6 = config.config6;
                }
            }
        }
        if has_config7_changes {
            match &self.config {
                GenIntConfig::Gen1Int(config) => {
                    self.device.interface.write_register(config.config7)?;
                    self.device.config.gen1int_config.config7 = config.config7;
                }
                GenIntConfig::Gen2Int(config) => {
                    self.device.interface.write_register(config.config7)?;
                    self.device.config.gen2int_config.config7 = config.config7;
                }
            }
        }
        if has_config8_changes {
            match &self.config {
                GenIntConfig::Gen1Int(config) => {
                    self.device.interface.write_register(config.config8)?;
                    self.device.config.gen1int_config.config8 = config.config8;
                }
                GenIntConfig::Gen2Int(config) => {
                    self.device.interface.write_register(config.config8)?;
                    self.device.config.gen2int_config.config8 = config.config8;
                }
            }
        }
        if has_config9_changes {
            match &self.config {
                GenIntConfig::Gen1Int(config) => {
                    self.device.interface.write_register(config.config9)?;
                    self.device.config.gen1int_config.config9 = config.config9;
                }
                GenIntConfig::Gen2Int(config) => {
                    self.device.interface.write_register(config.config9)?;
                    self.device.config.gen2int_config.config9 = config.config9;
                }
            }
        }
        // Re-enable interrupt, if it was disabled
        if int_config0.bits() != self.device.config.int_config.get_config0().bits() {
            self.device.interface.write_register(self.device.config.int_config.get_config0())?;
        }
        Ok(())
    }
    // Detect changes to assess whether to skip writing registers
    fn has_config0_changes_from(&self, device_config: &Config) -> bool {
        match &self.config {
            GenIntConfig::Gen1Int(config) => {
                config.config0.bits() != device_config.gen1int_config.config0.bits()
            }
            GenIntConfig::Gen2Int(config) => {
                config.config0.bits() != device_config.gen2int_config.config0.bits()
            }
        }
    }
    fn has_config1_changes_from(&self, device_config: &Config) -> bool {
        match &self.config {
            GenIntConfig::Gen1Int(config) => {
                config.config1.bits() != device_config.gen1int_config.config1.bits()
            }
            GenIntConfig::Gen2Int(config) => {
                config.config1.bits() != device_config.gen2int_config.config1.bits()
            }
        }
    }
    fn has_config2_changes_from(&self, device_config: &Config) -> bool {
        match &self.config {
            GenIntConfig::Gen1Int(config) => {
                config.config2.bits() != device_config.gen1int_config.config2.bits()
            }
            GenIntConfig::Gen2Int(config) => {
                config.config2.bits() != device_config.gen2int_config.config2.bits()
            }
        }
    }
    fn has_config3_changes_from(&self, device_config: &Config) -> bool {
        match &self.config {
            GenIntConfig::Gen1Int(config) => {
                config.config3.bits() != device_config.gen1int_config.config3.bits()
            }
            GenIntConfig::Gen2Int(config) => {
                config.config3.bits() != device_config.gen2int_config.config3.bits()
            }
        }
    }
    fn has_config31_changes_from(&self, device_config: &Config) -> bool {
        match &self.config {
            GenIntConfig::Gen1Int(config) => {
                config.config31.bits() != device_config.gen1int_config.config31.bits()
            }
            GenIntConfig::Gen2Int(config) => {
                config.config31.bits() != device_config.gen2int_config.config31.bits()
            }
        }
    }
    fn has_config4_changes_from(&self, device_config: &Config) -> bool {
        match &self.config {
            GenIntConfig::Gen1Int(config) => {
                config.config4.bits() != device_config.gen1int_config.config4.bits()
            }
            GenIntConfig::Gen2Int(config) => {
                config.config4.bits() != device_config.gen2int_config.config4.bits()
            }
        }
    }
    fn has_config5_changes_from(&self, device_config: &Config) -> bool {
        match &self.config {
            GenIntConfig::Gen1Int(config) => {
                config.config5.bits() != device_config.gen1int_config.config5.bits()
            }
            GenIntConfig::Gen2Int(config) => {
                config.config5.bits() != device_config.gen2int_config.config5.bits()
            }
        }
    }
    fn has_config6_changes_from(&self, device_config: &Config) -> bool {
        match &self.config {
            GenIntConfig::Gen1Int(config) => {
                config.config6.bits() != device_config.gen1int_config.config6.bits()
            }
            GenIntConfig::Gen2Int(config) => {
                config.config6.bits() != device_config.gen2int_config.config6.bits()
            }
        }
    }
    fn has_config7_changes_from(&self, device_config: &Config) -> bool {
        match &self.config {
            GenIntConfig::Gen1Int(config) => {
                config.config7.bits() != device_config.gen1int_config.config7.bits()
            }
            GenIntConfig::Gen2Int(config) => {
                config.config7.bits() != device_config.gen2int_config.config7.bits()
            }
        }
    }
    fn has_config8_changes_from(&self, device_config: &Config) -> bool {
        match &self.config {
            GenIntConfig::Gen1Int(config) => {
                config.config8.bits() != device_config.gen1int_config.config8.bits()
            }
            GenIntConfig::Gen2Int(config) => {
                config.config8.bits() != device_config.gen2int_config.config8.bits()
            }
        }
    }
    fn has_config9_changes_from(&self, device_config: &Config) -> bool {
        match &self.config {
            GenIntConfig::Gen1Int(config) => {
                config.config9.bits() != device_config.gen1int_config.config9.bits()
            }
            GenIntConfig::Gen2Int(config) => {
                config.config9.bits() != device_config.gen2int_config.config9.bits()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        tests::get_test_device,
        BMA400Error,
    };
    #[test]
    fn test_axes() {
        let mut device = get_test_device();
        let builder = device.config_gen1_int();
        assert!(matches!(builder.config, GenIntConfig::Gen1Int(_)));
        let builder = builder.with_axes(false, false, true);
        if let GenIntConfig::Gen1Int(config) = &builder.config {
            assert_eq!(config.config0.bits(), 0x80);
        }
        let builder = builder.with_axes(false, true, false);
        if let GenIntConfig::Gen1Int(config) = &builder.config {
            assert_eq!(config.config0.bits(), 0x40);
        }
        let builder = builder.with_axes(true, false, false);
        if let GenIntConfig::Gen1Int(config) = &builder.config {
            assert_eq!(config.config0.bits(), 0x20);
        }

        let builder = device.config_gen2_int();
        assert!(matches!(builder.config, GenIntConfig::Gen2Int(_)));
        let builder = builder.with_axes(false, false, true);
        if let GenIntConfig::Gen2Int(config) = &builder.config {
            assert_eq!(config.config0.bits(), 0x80);
        }
        let builder = builder.with_axes(false, true, false);
        if let GenIntConfig::Gen2Int(config) = &builder.config {
            assert_eq!(config.config0.bits(), 0x40);
        }
        let builder = builder.with_axes(true, false, false);
        if let GenIntConfig::Gen2Int(config) = &builder.config {
            assert_eq!(config.config0.bits(), 0x20);
        }
    }
    #[test]
    fn test_src() {
        let mut device = get_test_device();
        let builder = device.config_gen1_int();
        assert!(matches!(builder.config, GenIntConfig::Gen1Int(_)));
        let builder = builder.with_src(DataSource::AccFilt2Lp);
        if let GenIntConfig::Gen1Int(config) = &builder.config {
            assert_eq!(config.config0.bits(), 0x10);
        }
        let builder = builder.with_src(DataSource::AccFilt1);
        if let GenIntConfig::Gen1Int(config) = &builder.config {
            assert_eq!(config.config0.bits(), 0x00);
        }
        let builder = builder.with_src(DataSource::AccFilt2);
        if let GenIntConfig::Gen1Int(config) = &builder.config {
            assert_eq!(config.config0.bits(), 0x10);
        }

        let builder = device.config_gen2_int();
        assert!(matches!(builder.config, GenIntConfig::Gen2Int(_)));
        let builder = builder.with_src(DataSource::AccFilt2Lp);
        if let GenIntConfig::Gen2Int(config) = &builder.config {
            assert_eq!(config.config0.bits(), 0x10);
        }
        let builder = builder.with_src(DataSource::AccFilt1);
        if let GenIntConfig::Gen2Int(config) = &builder.config {
            assert_eq!(config.config0.bits(), 0x00);
        }
        let builder = builder.with_src(DataSource::AccFilt2);
        if let GenIntConfig::Gen2Int(config) = &builder.config {
            assert_eq!(config.config0.bits(), 0x10);
        }
    }
    #[test]
    fn test_reference_mode() {
        let mut device = get_test_device();
        let builder = device.config_gen1_int();
        assert!(matches!(builder.config, GenIntConfig::Gen1Int(_)));
        let builder = builder.with_reference_mode(GenIntRefMode::OneTime);
        if let GenIntConfig::Gen1Int(config) = &builder.config {
            assert_eq!(config.config0.bits(), 0x04);
        }
        let builder = builder.with_reference_mode(GenIntRefMode::EveryTimeFromSrc);
        if let GenIntConfig::Gen1Int(config) = &builder.config {
            assert_eq!(config.config0.bits(), 0x08);
        }
        let builder = builder.with_reference_mode(GenIntRefMode::EveryTimeFromLp);
        if let GenIntConfig::Gen1Int(config) = &builder.config {
            assert_eq!(config.config0.bits(), 0x0C);
        }
        let builder = builder.with_reference_mode(GenIntRefMode::Manual);
        if let GenIntConfig::Gen1Int(config) = &builder.config {
            assert_eq!(config.config0.bits(), 0x00);
        }

        let builder = device.config_gen2_int();
        assert!(matches!(builder.config, GenIntConfig::Gen2Int(_)));
        let builder = builder.with_reference_mode(GenIntRefMode::OneTime);
        if let GenIntConfig::Gen2Int(config) = &builder.config {
            assert_eq!(config.config0.bits(), 0x04);
        }
        let builder = builder.with_reference_mode(GenIntRefMode::EveryTimeFromSrc);
        if let GenIntConfig::Gen2Int(config) = &builder.config {
            assert_eq!(config.config0.bits(), 0x08);
        }
        let builder = builder.with_reference_mode(GenIntRefMode::EveryTimeFromLp);
        if let GenIntConfig::Gen2Int(config) = &builder.config {
            assert_eq!(config.config0.bits(), 0x0C);
        }
        let builder = builder.with_reference_mode(GenIntRefMode::Manual);
        if let GenIntConfig::Gen2Int(config) = &builder.config {
            assert_eq!(config.config0.bits(), 0x00);
        }
    }
    #[test]
    fn test_hysteresis() {
        let mut device = get_test_device();
        let builder = device.config_gen1_int();
        assert!(matches!(builder.config, GenIntConfig::Gen1Int(_)));
        let builder = builder.with_hysteresis(Hysteresis::Hyst96mg);
        if let GenIntConfig::Gen1Int(config) = &builder.config {
            assert_eq!(config.config0.bits(), 0x03);
        }
        let builder = builder.with_hysteresis(Hysteresis::Hyst48mg);
        if let GenIntConfig::Gen1Int(config) = &builder.config {
            assert_eq!(config.config0.bits(), 0x02);
        }
        let builder = builder.with_hysteresis(Hysteresis::Hyst24mg);
        if let GenIntConfig::Gen1Int(config) = &builder.config {
            assert_eq!(config.config0.bits(), 0x01);
        }
        let builder = builder.with_hysteresis(Hysteresis::None);
        if let GenIntConfig::Gen1Int(config) = &builder.config {
            assert_eq!(config.config0.bits(), 0x00);
        }

        let builder = device.config_gen2_int();
        assert!(matches!(builder.config, GenIntConfig::Gen2Int(_)));
        let builder = builder.with_hysteresis(Hysteresis::Hyst96mg);
        if let GenIntConfig::Gen2Int(config) = &builder.config {
            assert_eq!(config.config0.bits(), 0x03);
        }
        let builder = builder.with_hysteresis(Hysteresis::Hyst48mg);
        if let GenIntConfig::Gen2Int(config) = &builder.config {
            assert_eq!(config.config0.bits(), 0x02);
        }
        let builder = builder.with_hysteresis(Hysteresis::Hyst24mg);
        if let GenIntConfig::Gen2Int(config) = &builder.config {
            assert_eq!(config.config0.bits(), 0x01);
        }
        let builder = builder.with_hysteresis(Hysteresis::None);
        if let GenIntConfig::Gen2Int(config) = &builder.config {
            assert_eq!(config.config0.bits(), 0x00);
        }
    }
    #[test]
    fn test_criterion_mode() {
        let mut device = get_test_device();
        let builder = device.config_gen1_int();
        assert!(matches!(builder.config, GenIntConfig::Gen1Int(_)));
        let builder = builder.with_criterion_mode(GenIntCriterionMode::Activity);
        if let GenIntConfig::Gen1Int(config) = &builder.config {
            assert_eq!(config.config1.bits(), 0x02);
        }
        let builder = builder.with_criterion_mode(GenIntCriterionMode::Inactivity);
        if let GenIntConfig::Gen1Int(config) = &builder.config {
            assert_eq!(config.config1.bits(), 0x00);
        }

        let builder = device.config_gen2_int();
        assert!(matches!(builder.config, GenIntConfig::Gen2Int(_)));
        let builder = builder.with_criterion_mode(GenIntCriterionMode::Activity);
        if let GenIntConfig::Gen2Int(config) = &builder.config {
            assert_eq!(config.config1.bits(), 0x02);
        }
        let builder = builder.with_criterion_mode(GenIntCriterionMode::Inactivity);
        if let GenIntConfig::Gen2Int(config) = &builder.config {
            assert_eq!(config.config1.bits(), 0x00);
        }
    }
    #[test]
    fn test_logic_mode() {
        let mut device = get_test_device();
        let builder = device.config_gen1_int();
        assert!(matches!(builder.config, GenIntConfig::Gen1Int(_)));
        let builder = builder.with_logic_mode(GenIntLogicMode::And);
        if let GenIntConfig::Gen1Int(config) = &builder.config {
            assert_eq!(config.config1.bits(), 0x01);
        }
        let builder = builder.with_logic_mode(GenIntLogicMode::Or);
        if let GenIntConfig::Gen1Int(config) = &builder.config {
            assert_eq!(config.config1.bits(), 0x00);
        }

        let builder = device.config_gen2_int();
        assert!(matches!(builder.config, GenIntConfig::Gen2Int(_)));
        let builder = builder.with_logic_mode(GenIntLogicMode::And);
        if let GenIntConfig::Gen2Int(config) = &builder.config {
            assert_eq!(config.config1.bits(), 0x01);
        }
        let builder = builder.with_logic_mode(GenIntLogicMode::Or);
        if let GenIntConfig::Gen2Int(config) = &builder.config {
            assert_eq!(config.config1.bits(), 0x00);
        }
    }
    #[test]
    fn test_threshold() {
        let mut device = get_test_device();
        let builder = device.config_gen1_int();
        assert!(matches!(builder.config, GenIntConfig::Gen1Int(_)));
        let builder = builder.with_threshold(0xFF);
        if let GenIntConfig::Gen1Int(config) = &builder.config {
            assert_eq!(config.config2.bits(), 0xFF);
        }
        let builder = builder.with_threshold(0);
        if let GenIntConfig::Gen1Int(config) = &builder.config {
            assert_eq!(config.config2.bits(), 0);
        }

        let builder = device.config_gen2_int();
        assert!(matches!(builder.config, GenIntConfig::Gen2Int(_)));
        let builder = builder.with_threshold(0xFF);
        if let GenIntConfig::Gen2Int(config) = &builder.config {
            assert_eq!(config.config2.bits(), 0xFF);
        }
        let builder = builder.with_threshold(0);
        if let GenIntConfig::Gen2Int(config) = &builder.config {
            assert_eq!(config.config2.bits(), 0);
        }
    }
    #[test]
    fn test_duration() {
        let mut device = get_test_device();
        let builder = device.config_gen1_int();
        assert!(matches!(builder.config, GenIntConfig::Gen1Int(_)));
        let builder = builder.with_duration(0xFF00);
        if let GenIntConfig::Gen1Int(config) = &builder.config {
            assert_eq!(config.config3.bits(), 0xFF);
            assert_eq!(config.config31.bits(), 0x00);
        }
        let builder = builder.with_duration(0x00FF);
        if let GenIntConfig::Gen1Int(config) = &builder.config {
            assert_eq!(config.config3.bits(), 0x00);
            assert_eq!(config.config31.bits(), 0xFF);
        }

        let builder = device.config_gen2_int();
        assert!(matches!(builder.config, GenIntConfig::Gen2Int(_)));
        let builder = builder.with_duration(0xFF00);
        if let GenIntConfig::Gen2Int(config) = &builder.config {
            assert_eq!(config.config3.bits(), 0xFF);
            assert_eq!(config.config31.bits(), 0x00);
        }
        let builder = builder.with_duration(0x00FF);
        if let GenIntConfig::Gen2Int(config) = &builder.config {
            assert_eq!(config.config3.bits(), 0x00);
            assert_eq!(config.config31.bits(), 0xFF);
        }
    }
    #[test]
    fn test_int1_config_err() {
        let mut device = get_test_device();
        // Change the data source to AccFilt2
        assert!(matches!(device.config_gen1_int().with_src(DataSource::AccFilt2).write(), Ok(())));
        // Enable the interrupt
        assert!(matches!(device.config_interrupts().with_gen1_int(true).write(), Ok(())));
        // Try to change the data source back to AccFilt1 while the interrupt is enabled
        assert!(matches!(
            device.config_gen1_int().with_src(DataSource::AccFilt1).write(),
            Err(BMA400Error::ConfigBuildError(ConfigError::Filt1InterruptInvalidODR))
        ));
    }
    #[test]
    fn test_int2_config_err() {
        let mut device = get_test_device();
        // Change the data source to AccFilt2
        assert!(matches!(device.config_gen2_int().with_src(DataSource::AccFilt2).write(), Ok(())));
        // Enable the interrupt
        assert!(matches!(device.config_interrupts().with_gen2_int(true).write(), Ok(())));
        // Try to change the data source back to AccFilt1 while the interrupt is enabled
        assert!(matches!(
            device.config_gen2_int().with_src(DataSource::AccFilt1).write(),
            Err(BMA400Error::ConfigBuildError(ConfigError::Filt1InterruptInvalidODR))
        ));
    }
}
