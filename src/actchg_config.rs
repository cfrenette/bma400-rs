use crate::{
    Debug,
    registers::{ActChgConfig0, ActChgConfig1}, 
    interface::WriteToRegister, 
    BMA400, 
    ConfigError, 
    DataSource, 
    OutputDataRate
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

pub struct ActChgBuilder<'a, Interface: WriteToRegister> {
    config: ActChgConfig,
    device: &'a mut BMA400<Interface>,
}

impl<'a, Interface, E> ActChgBuilder<'a, Interface>
where
    Interface: WriteToRegister<Error = E>,
    E: From<ConfigError> + Debug,
{
    pub fn write(self) -> Result<(), E> {
        if let DataSource::AccFilt1 = self.config.src() {
            match self.device.config.acc_config.odr() {
                OutputDataRate::Hz100 => {},
                _ => return Err(ConfigError::Filt1InterruptInvalidODR.into()),
            }
        }
        todo!()
    }
}