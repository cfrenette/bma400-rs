use crate::{
    Debug,
    //registers::{GenInt1Config0, GenInt2Config0},
    interface::WriteToRegister,
    BMA400,
    ConfigError, DataSource, 
};

pub struct Gen1IntConfig {
    //config0: GenInt1Config0,
    /* 
    config1: u8,
    config2: u8,
    config3: u8,
    config31: u8,
    config4: u8,
    config5: u8,
    config6: u8,
    config7: u8,
    config8: u8,
    config9: u8,
    */
}

pub struct Gen2IntConfig {
    //config0: GenInt2Config0
    /* 
    config1: u8,
    config2: u8,
    config3: u8,
    config31: u8,
    config4: u8,
    config5: u8,
    config6: u8,
    config7: u8,
    config8: u8,
    config9: u8,
    */
}

pub enum GenericInterruptType {
    Gen1Int(Gen1IntConfig),
    Gen2Int(Gen2IntConfig),
}

impl GenericInterruptType {
    pub fn src(&self) -> DataSource {
        todo!()
    }
}

pub struct GenIntConfig<GenericInterruptType> {
    int_type: GenericInterruptType,
}

impl GenIntConfig<GenericInterruptType> {
    pub fn src(&self) -> DataSource {
        self.int_type.src()
    }
}

pub struct GenIntConfigBuilder<'a, Interface: WriteToRegister> {
    config: GenIntConfig<GenericInterruptType>,
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