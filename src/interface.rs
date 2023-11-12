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
    fn write_register<T: ConfigReg>(&mut self, register: T) -> impl core::future::Future<Output = Result<(), Self::Error>>;
}

#[cfg(feature = "async")]
pub trait AsyncReadFromRegister {
    type Error;
    fn read_register<T: ReadReg>(
        &mut self,
        register: T,
        buffer: &mut [u8],
    ) -> impl core::future::Future<Output = Result<(), Self::Error>>;
}
