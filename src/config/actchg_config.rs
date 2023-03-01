use crate::{
    registers::{ActChgConfig0, ActChgConfig1}, 
    interface::WriteToRegister, 
    BMA400, 
    ConfigError, 
    DataSource, 
    ActChgObsPeriod
};

#[derive(Clone, Default)]
pub struct ActChgConfig {
    actchg_config0: ActChgConfig0,
    actchg_config1: ActChgConfig1,
}

impl ActChgConfig {
    pub fn src(&self) -> DataSource {
        self.actchg_config1.dta_src()
    }
}

pub struct ActChgConfigBuilder<'a, Interface: WriteToRegister> {
    config: ActChgConfig,
    device: &'a mut BMA400<Interface>,
}

impl<'a, Interface, E> ActChgConfigBuilder<'a, Interface>
where
    Interface: WriteToRegister<Error = E>,
    E: From<ConfigError>,
{
    pub fn new(device: &'a mut BMA400<Interface>) -> ActChgConfigBuilder<'a, Interface> {
        ActChgConfigBuilder { config: device.config.actchg_config.clone(), device }
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
        self.config.actchg_config1 = self.config.actchg_config1.with_x_axis(x_en).with_y_axis(y_en).with_z_axis(z_en);
        self
    }
    /// Select the data source used for evaluating the activity changed interrupt condition
    /// 
    /// Cannot use [DataSource::AccFilt2Lp]. If passed, this will default to AccFilt2
    pub fn with_src(mut self, src: DataSource) -> Self {
        self.config.actchg_config1 = match src {
            DataSource::AccFilt2Lp => self.config.actchg_config1.with_dta_src(DataSource::AccFilt2),
            _ => self.config.actchg_config1.with_dta_src(src),
        };
        self
    }
    /// Select the number of samples to observe (observation period) when evaluating the activity type
    pub fn with_obs_period(mut self, obs_period: ActChgObsPeriod) -> Self {
        self.config.actchg_config1 = self.config.actchg_config1.with_observation_period(obs_period);
        self
    }
    pub fn write(self) -> Result<(), E> {

        let has_config0_changes = self.device.config.actchg_config.actchg_config0.bits() != self.config.actchg_config0.bits();
        let has_config1_changes = self.device.config.actchg_config.actchg_config1.bits() != self.config.actchg_config1.bits();
        let has_changes = has_config0_changes || has_config1_changes;

        let mut tmp_int_config1 = self.device.config.int_config.get_config1().clone();

        // Temporarily disable the interrupt, if active
        if tmp_int_config1.actch_int() && has_changes {
            tmp_int_config1 = tmp_int_config1.with_actch_int(false);
            self.device.interface.write_register(tmp_int_config1)?;
        }

        // Write the changes
        if has_config0_changes {
            self.device.interface.write_register(self.config.actchg_config0)?;
            self.device.config.actchg_config.actchg_config0 = self.config.actchg_config0;
        }
        if has_config1_changes {
            self.device.interface.write_register(self.config.actchg_config1)?;
            self.device.config.actchg_config.actchg_config1 = self.config.actchg_config1;
        }

        // Re-enable the interrupt, if it was disabled
        if self.device.config.int_config.get_config1().bits() != tmp_int_config1.bits() {
            self.device.interface.write_register(self.device.config.int_config.get_config0())?;
        }
        Ok(())
    }
}