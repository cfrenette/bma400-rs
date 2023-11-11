use crate::registers::{ConfigReg, ReadReg};

pub trait WriteToRegister {
    type Error;
    fn write_register<T: ConfigReg>(&mut self, register: T) -> Result<(), Self::Error>;
}

pub trait ReadFromRegister {
    type Error;
    fn read_register<T: ReadReg>(
        &mut self,
        register: T,
        buffer: &mut [u8],
    ) -> Result<(), Self::Error>;
}

#[cfg(feature = "async")]
pub trait AsyncWriteToRegister {
    type Error;
    async fn write_register<T: ConfigReg>(&mut self, register: T) -> Result<(), Self::Error>;
}

#[cfg(feature = "async")]
pub trait AsyncReadFromRegister {
    type Error;
    async fn read_register<T: ReadReg>(
        &mut self,
        register: T,
        buffer: &mut [u8],
    ) -> Result<(), Self::Error>;
}
