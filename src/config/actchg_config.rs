use crate::{
    ActChgObsPeriod, BMA400, ConfigError, DataSource, OutputDataRate,
    registers::{ActChgConfig0, ActChgConfig1},
};

#[derive(Clone, Default)]
pub struct ActChgConfig {
    actchg_config0: ActChgConfig0,
    actchg_config1: ActChgConfig1,
}

impl ActChgConfig {
    pub fn src(&self) -> DataSource {
        self.actchg_config1.src()
    }
}

/// Configure Activity Change Interrupt settings
///
/// - Set the interrupt trigger threshold using [`with_threshold()`](ActChgConfigBuilder::with_threshold)
/// - Enable / Disable the axes evaluated for the interrupt trigger condition using [`with_axes()`](ActChgConfigBuilder::with_axes)
/// - [DataSource] used for evaluating the trigger condition using [`with_src()`](ActChgConfigBuilder::with_src)
/// - [ActChgObsPeriod] (number of samples) using [`with_obs_period()`](ActChgConfigBuilder::with_obs_period)
pub struct ActChgConfigBuilder<'a, Interface> {
    config: ActChgConfig,
    device: &'a mut BMA400<Interface>,
}

#[cfg(not(feature = "embedded-hal-async"))]
impl<'a, Interface, E> ActChgConfigBuilder<'a, Interface>
where
    Interface: crate::blocking::WriteToRegister<Error = E>,
    E: From<ConfigError>,
{
    /// Write this configuration to device registers
    pub fn write(self) -> Result<(), E> {
        let has_config0_changes = self.device.config.actchg_config.actchg_config0.bits()
            != self.config.actchg_config0.bits();
        let has_config1_changes = self.device.config.actchg_config.actchg_config1.bits()
            != self.config.actchg_config1.bits();
        let has_changes = has_config0_changes || has_config1_changes;

        // If there are no changes, return early
        if !has_changes {
            return Ok(());
        }

        let mut tmp_int_config1 = self.device.config.int_config.get_config1();
        let int_enabled = tmp_int_config1.actch_int();

        // If the interrupt is enabled and we're trying to change the Data Source to AccFilt1, the ODR must be 100Hz
        if int_enabled
            && matches!(self.config.actchg_config1.src(), DataSource::AccFilt1)
            && !matches!(self.device.config.acc_config.odr(), OutputDataRate::Hz100)
        {
            return Err(ConfigError::Filt1InterruptInvalidODR.into());
        }

        // Temporarily disable the interrupt, if active
        if int_enabled {
            tmp_int_config1 = tmp_int_config1.with_actch_int(false);
            self.device.interface.write_register(tmp_int_config1)?;
        }

        // Write the changes
        if has_config0_changes {
            self.device
                .interface
                .write_register(self.config.actchg_config0)?;
            self.device.config.actchg_config.actchg_config0 = self.config.actchg_config0;
        }
        if has_config1_changes {
            self.device
                .interface
                .write_register(self.config.actchg_config1)?;
            self.device.config.actchg_config.actchg_config1 = self.config.actchg_config1;
        }

        // Re-enable the interrupt, if it was disabled
        if self.device.config.int_config.get_config1().bits() != tmp_int_config1.bits() {
            self.device
                .interface
                .write_register(self.device.config.int_config.get_config0())?;
        }
        Ok(())
    }
}

#[cfg(feature = "embedded-hal-async")]
impl<'a, Interface, E> ActChgConfigBuilder<'a, Interface>
where
    Interface: crate::asynch::WriteToRegister<Error = E>,
    E: From<ConfigError>,
{
    /// Write this configuration to device registers
    pub async fn write(self) -> Result<(), E> {
        let has_config0_changes = self.device.config.actchg_config.actchg_config0.bits()
            != self.config.actchg_config0.bits();
        let has_config1_changes = self.device.config.actchg_config.actchg_config1.bits()
            != self.config.actchg_config1.bits();
        let has_changes = has_config0_changes || has_config1_changes;

        // If there are no changes, return early
        if !has_changes {
            return Ok(());
        }

        let mut tmp_int_config1 = self.device.config.int_config.get_config1();
        let int_enabled = tmp_int_config1.actch_int();

        // If the interrupt is enabled and we're trying to change the Data Source to AccFilt1, the ODR must be 100Hz
        if int_enabled
            && matches!(self.config.actchg_config1.src(), DataSource::AccFilt1)
            && !matches!(self.device.config.acc_config.odr(), OutputDataRate::Hz100)
        {
            return Err(ConfigError::Filt1InterruptInvalidODR.into());
        }

        // Temporarily disable the interrupt, if active
        if int_enabled {
            tmp_int_config1 = tmp_int_config1.with_actch_int(false);
            self.device
                .interface
                .write_register(tmp_int_config1)
                .await?;
        }

        // Write the changes
        if has_config0_changes {
            self.device
                .interface
                .write_register(self.config.actchg_config0)
                .await?;
            self.device.config.actchg_config.actchg_config0 = self.config.actchg_config0;
        }
        if has_config1_changes {
            self.device
                .interface
                .write_register(self.config.actchg_config1)
                .await?;
            self.device.config.actchg_config.actchg_config1 = self.config.actchg_config1;
        }

        // Re-enable the interrupt, if it was disabled
        if self.device.config.int_config.get_config1().bits() != tmp_int_config1.bits() {
            self.device
                .interface
                .write_register(self.device.config.int_config.get_config0())
                .await?;
        }
        Ok(())
    }
}

impl<'a, Interface> ActChgConfigBuilder<'a, Interface> {
    pub(crate) fn new(device: &'a mut BMA400<Interface>) -> ActChgConfigBuilder<'a, Interface> {
        ActChgConfigBuilder {
            config: device.config.actchg_config.clone(),
            device,
        }
    }
    // ActChgConfig0
    /// Set the threshold used when evaluating the activity changed interrupt condition
    pub fn with_threshold(mut self, threshold: u8) -> Self {
        self.config.actchg_config0 = self.config.actchg_config0.with_actch_thres(threshold);
        self
    }

    // ActChgConfig1
    /// Select the axes to be used when evaluating the activity changed interrupt condition
    pub fn with_axes(mut self, x_en: bool, y_en: bool, z_en: bool) -> Self {
        self.config.actchg_config1 = self
            .config
            .actchg_config1
            .with_x_axis(x_en)
            .with_y_axis(y_en)
            .with_z_axis(z_en);
        self
    }
    /// Select the data source used for evaluating the activity changed interrupt condition
    ///
    /// Cannot use [DataSource::AccFilt2Lp]. If passed, this will default to AccFilt2
    pub fn with_src(mut self, src: DataSource) -> Self {
        self.config.actchg_config1 = match src {
            DataSource::AccFilt2Lp => self
                .config
                .actchg_config1
                .with_dta_src(DataSource::AccFilt2),
            _ => self.config.actchg_config1.with_dta_src(src),
        };
        self
    }
    /// Select the number of samples to observe (observation period) when evaluating the activity
    /// type
    pub fn with_obs_period(mut self, obs_period: ActChgObsPeriod) -> Self {
        self.config.actchg_config1 = self
            .config
            .actchg_config1
            .with_observation_period(obs_period);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BMA400Error, tests::get_test_device};
    #[test]
    fn test_threshold() {
        let mut device = get_test_device();
        let builder = device.config_actchg_int();
        let builder = builder.with_threshold(255);
        assert_eq!(builder.config.actchg_config0.bits(), 0xFF);
        let builder = builder.with_threshold(0);
        assert_eq!(builder.config.actchg_config0.bits(), 0x00);
    }
    #[test]
    fn test_axes() {
        let mut device = get_test_device();
        let builder = device.config_actchg_int();
        let builder = builder.with_axes(false, false, true);
        assert_eq!(builder.config.actchg_config1.bits(), 0x80);
        let builder = builder.with_axes(false, true, false);
        assert_eq!(builder.config.actchg_config1.bits(), 0x40);
        let builder = builder.with_axes(true, false, false);
        assert_eq!(builder.config.actchg_config1.bits(), 0x20);
    }
    #[test]
    fn test_src() {
        let mut device = get_test_device();
        let builder = device.config_actchg_int();
        let builder = builder.with_src(DataSource::AccFilt2Lp);
        assert_eq!(builder.config.actchg_config1.bits(), 0x10);
        let builder = builder.with_src(DataSource::AccFilt1);
        assert_eq!(builder.config.actchg_config1.bits(), 0x00);
        let builder = builder.with_src(DataSource::AccFilt2);
        assert_eq!(builder.config.actchg_config1.bits(), 0x10);
    }
    #[test]
    fn test_obs_period() {
        let mut device = get_test_device();
        let builder = device.config_actchg_int();
        let builder = builder.with_obs_period(ActChgObsPeriod::Samples64);
        assert_eq!(builder.config.actchg_config1.bits(), 0x01);
        let builder = builder.with_obs_period(ActChgObsPeriod::Samples128);
        assert_eq!(builder.config.actchg_config1.bits(), 0x02);
        let builder = builder.with_obs_period(ActChgObsPeriod::Samples256);
        assert_eq!(builder.config.actchg_config1.bits(), 0x03);
        let builder = builder.with_obs_period(ActChgObsPeriod::Samples512);
        assert_eq!(builder.config.actchg_config1.bits(), 0x04);
        let builder = builder.with_obs_period(ActChgObsPeriod::Samples32);
        assert_eq!(builder.config.actchg_config1.bits(), 0x00);
    }
    #[test]
    fn test_config_err() {
        let mut device = get_test_device();
        // Change the data source to AccFilt2
        assert!(matches!(
            device
                .config_actchg_int()
                .with_src(DataSource::AccFilt2)
                .write(),
            Ok(())
        ));
        // Enable the interrupt
        assert!(matches!(
            device.config_interrupts().with_actch_int(true).write(),
            Ok(())
        ));
        // Try to change the data source back to AccFilt1 while the interrupt is enabled
        let result = device
            .config_actchg_int()
            .with_src(DataSource::AccFilt1)
            .write();
        assert!(matches!(
            result,
            Err(BMA400Error::ConfigBuildError(
                ConfigError::Filt1InterruptInvalidODR
            ))
        ));
    }
}
