use crate::{
    Debug,
    registers::{GenInt1Config0, GenInt2Config0},
    interface::WriteToRegister,
    BMA400,
    ConfigError, 
};

pub struct GenIntConfig {
    gen_config0: GenInt1Config0,
}

impl GenIntConfig {
    pub fn src(&self) -> DataSource {
        self.gen_config0.data_src()
    }
}

pub struct GenIntConfigBuilder<'a, Interface: WriteToRegister> {
    config: GenIntConfig,
    device: &'a mut BMA400<Interface>,
}

impl <'a, Interface, E> GenIntConfigBuilder<'a, Interface>
where
    Interface: WriteToRegister<Error = E>,
    E: From<ConfigError> + Debug,
{
    pub fn write(self) -> Result<(), E> {
        if let DataSource::AccFilt1 = self.config.src() {

        }
        // If Gen1 or Gen2 changes, need to disable interrupt before writing changes
        todo!()
    }
}