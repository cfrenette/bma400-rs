use crate::{
    registers::{OrientChgConfig0, OrientChgConfig1, OrientChgConfig3, OrientChgConfig4, OrientChgConfig5, OrientChgConfig6, OrientChgConfig7, OrientChgConfig8, OrientChgConfig9},
    interface::WriteToRegister,
    BMA400,
    ConfigError, DataSource, OrientIntRefMode,
};

#[derive(Clone, Default)]
pub struct OrientChgConfig {
    orientch_config0: OrientChgConfig0,
    orientch_config1: OrientChgConfig1,
    orientch_config3: OrientChgConfig3,
    orientch_config4: OrientChgConfig4,
    orientch_config5: OrientChgConfig5,
    orientch_config6: OrientChgConfig6,
    orientch_config7: OrientChgConfig7,
    orientch_config8: OrientChgConfig8,
    orientch_config9: OrientChgConfig9,
}

pub struct OrientChgConfigBuilder<'a, Interface> {
    config: OrientChgConfig,
    device: &'a mut BMA400<Interface>,
}

impl<'a, Interface, E> OrientChgConfigBuilder<'a, Interface>
where 
    Interface: WriteToRegister<Error = E>,
    E: From<ConfigError>,
{
    pub(crate) fn new(device: &'a mut BMA400<Interface>) -> OrientChgConfigBuilder<'a, Interface> {
        OrientChgConfigBuilder { config: device.config.orientch_config.clone(), device }
    }

    // OrientChgConfig0

    /// Enable/Disable the axes evaluated for the interrupt trigger condition
    pub fn with_axes(mut self, x_en: bool, y_en: bool, z_en: bool) -> Self {
        self.config.orientch_config0 = self.config.orientch_config0.with_x_axis(x_en).with_y_axis(y_en).with_z_axis(z_en);
        self
    }
    /// Set the data source used for evaluating the interrupt trigger condition
    /// 
    /// Cannot use [DataSource::AccFilt1]. If passed, this will default to AccFilt2
    pub fn with_src(mut self, src: DataSource) -> Self {
        self.config.orientch_config0 = match src {
            DataSource::AccFilt1 => self.config.orientch_config0.with_data_src(DataSource::AccFilt2),
            _ => self.config.orientch_config0.with_data_src(src),
        };
        self
    }
    /// Set reference update mode for the interrupt
    pub fn with_ref_mode(mut self, mode: OrientIntRefMode) -> Self {
        self.config.orientch_config0 = self.config.orientch_config0.with_update_mode(mode);
        self
    }

    // OrientChgConfig1
    /// Set threshold above reference acceleration for the interrupt evaluation (8 milli-g / lsb)
    pub fn with_threshold(mut self, threshold: u8) -> Self {
        self.config.orientch_config1 = self.config.orientch_config1.with_orient_thresh(threshold);
        self
    }

    // OrientChgConfig3
    /// Set the duration (in number of samples) that a new detected orientation must be in effect before the interrupt is triggered.
    /// 
    /// The ODR of the data source is 100Hz, so this value is in multiples of 10ms
    pub fn with_duration(mut self, duration: u8) -> Self {
        self.config.orientch_config3 = self.config.orientch_config3.with_orient_dur(duration);
        self
    }

    // OrientChgConfig4-9
    /// Manually set the reference acceleration for the x,y,z axes (use with [OrientIntRefMode::Manual])
    /// 
    /// In order for an axis to be evaluated it must be enabled using `with_axes()`
    pub fn with_ref_accel(mut self, ref_x: i16, ref_y: i16, ref_z: i16) -> Self {
        let (ref_x, ref_y, ref_z) = (ref_x.clamp(-2048, 2047), ref_y.clamp(-2048, 2047), ref_z.clamp(-2048, 2047));

        self.config.orientch_config4 = self.config.orientch_config4.with_refx_lsb(ref_x);
        self.config.orientch_config5 = self.config.orientch_config5.with_refx_msb(ref_x);

        self.config.orientch_config6 = self.config.orientch_config6.with_refy_lsb(ref_y);
        self.config.orientch_config7 = self.config.orientch_config7.with_refy_msb(ref_y);

        self.config.orientch_config8 = self.config.orientch_config8.with_refz_lsb(ref_z);
        self.config.orientch_config9 = self.config.orientch_config9.with_refz_msb(ref_z);

        self
    }
    pub fn write(self) -> Result<(), E> {

        let has_config0_changes = self.device.config.orientch_config.orientch_config0.bits() != self.config.orientch_config0.bits();
        let has_config1_changes = self.device.config.orientch_config.orientch_config1.bits() != self.config.orientch_config1.bits();
        let has_config3_changes = self.device.config.orientch_config.orientch_config3.bits() != self.config.orientch_config3.bits();
        let has_config4_changes = self.device.config.orientch_config.orientch_config4.bits() != self.config.orientch_config4.bits();
        let has_config5_changes = self.device.config.orientch_config.orientch_config5.bits() != self.config.orientch_config5.bits();
        let has_config6_changes = self.device.config.orientch_config.orientch_config6.bits() != self.config.orientch_config6.bits();
        let has_config7_changes = self.device.config.orientch_config.orientch_config7.bits() != self.config.orientch_config7.bits();
        let has_config8_changes = self.device.config.orientch_config.orientch_config8.bits() != self.config.orientch_config8.bits();
        let has_config9_changes = self.device.config.orientch_config.orientch_config9.bits() != self.config.orientch_config9.bits();
        let has_changes = has_config0_changes || has_config1_changes || has_config3_changes || has_config4_changes || has_config5_changes ||
                                has_config6_changes || has_config7_changes || has_config8_changes || has_config9_changes;

        let mut tmp_int_config0 = self.device.config.int_config.get_config0();

        // Temporarily disable interrupt, if active
        if tmp_int_config0.orientch_int() && has_changes {
            tmp_int_config0 = tmp_int_config0.with_orientch_int(false);
            self.device.interface.write_register(tmp_int_config0)?;
        }
        // Write the changes
        if has_config0_changes {
            self.device.interface.write_register(self.config.orientch_config0)?;
            self.device.config.orientch_config.orientch_config0 = self.config.orientch_config0;
        }
        if has_config1_changes {
            self.device.interface.write_register(self.config.orientch_config1)?;
            self.device.config.orientch_config.orientch_config1 = self.config.orientch_config1;
        }
        if has_config3_changes {
            self.device.interface.write_register(self.config.orientch_config3)?;
            self.device.config.orientch_config.orientch_config3 = self.config.orientch_config3;
        }
        if has_config4_changes {
            self.device.interface.write_register(self.config.orientch_config4)?;
            self.device.config.orientch_config.orientch_config4 = self.config.orientch_config4;
        }
        if has_config5_changes {
            self.device.interface.write_register(self.config.orientch_config5)?;
            self.device.config.orientch_config.orientch_config5 = self.config.orientch_config5;
        }
        if has_config6_changes {
            self.device.interface.write_register(self.config.orientch_config6)?;
            self.device.config.orientch_config.orientch_config6 = self.config.orientch_config6;
        }
        if has_config7_changes {
            self.device.interface.write_register(self.config.orientch_config7)?;
            self.device.config.orientch_config.orientch_config7 = self.config.orientch_config7;
        }
        if has_config8_changes {
            self.device.interface.write_register(self.config.orientch_config8)?;
            self.device.config.orientch_config.orientch_config8 = self.config.orientch_config8;
        }
        if has_config9_changes {
            self.device.interface.write_register(self.config.orientch_config9)?;
            self.device.config.orientch_config.orientch_config9 = self.config.orientch_config9;
        }
        // Re-enable interrupt, if disabled
        if self.device.config.int_config.get_config0().bits() != tmp_int_config0.bits() {
            self.device.interface.write_register(self.device.config.int_config.get_config0())?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::get_test_device;
    #[test]
    fn test_axes() {
        let mut device = get_test_device();
        let builder = device.config_orientchg_int();
        let builder = builder.with_axes(false, false, true);
        assert_eq!(builder.config.orientch_config0.bits(), 0x80);
        let builder = builder.with_axes(false, true, false);
        assert_eq!(builder.config.orientch_config0.bits(), 0x40);
        let builder = builder.with_axes(true, false, false);
        assert_eq!(builder.config.orientch_config0.bits(), 0x20);
    }
    #[test]
    fn test_src() {
        let mut device = get_test_device();
        let builder = device.config_orientchg_int();
        let builder = builder.with_src(DataSource::AccFilt1);
        assert_eq!(builder.config.orientch_config0.bits(), 0x00);
        let builder = builder.with_src(DataSource::AccFilt2Lp);
        assert_eq!(builder.config.orientch_config0.bits(), 0x10);
        let builder = builder.with_src(DataSource::AccFilt2);
        assert_eq!(builder.config.orientch_config0.bits(), 0x00);
    }
    #[test]
    fn test_ref_mode() {
        let mut device = get_test_device();
        let builder = device.config_orientchg_int();
        let builder = builder.with_ref_mode(OrientIntRefMode::AccFilt2);
        assert_eq!(builder.config.orientch_config0.bits(), 0x04);
        let builder = builder.with_ref_mode(OrientIntRefMode::AccFilt2Lp);
        assert_eq!(builder.config.orientch_config0.bits(), 0x08);
        let builder = builder.with_ref_mode(OrientIntRefMode::Manual);
        assert_eq!(builder.config.orientch_config0.bits(), 0x00);
    }
    #[test]
    fn test_threshold() {
        let mut device = get_test_device();
        let builder = device.config_orientchg_int();
        let builder = builder.with_threshold(255);
        assert_eq!(builder.config.orientch_config1.bits(), 0xFF);
        let builder = builder.with_threshold(0);
        assert_eq!(builder.config.orientch_config1.bits(), 0x00);
    }
    #[test]
    fn test_duration() {
        let mut device = get_test_device();
        let builder = device.config_orientchg_int();
        let builder = builder.with_duration(255);
        assert_eq!(builder.config.orientch_config3.bits(), 0xFF);
        let builder = builder.with_duration(0);
        assert_eq!(builder.config.orientch_config3.bits(), 0x00);
    }
    #[test]
    fn test_ref_accel() {
        let mut device = get_test_device();
        let builder = device.config_orientchg_int();
        let builder = builder.with_ref_accel(-256,240, 15);
        assert_eq!(builder.config.orientch_config4.bits(), 0x00);
        assert_eq!(builder.config.orientch_config5.bits(), 0x0F);
        assert_eq!(builder.config.orientch_config6.bits(), 0xF0);
        assert_eq!(builder.config.orientch_config7.bits(), 0x00);
        assert_eq!(builder.config.orientch_config8.bits(), 0x0F);
        assert_eq!(builder.config.orientch_config9.bits(), 0x00);
    }
}
