use crate::hal::blocking::i2c::{Write, WriteRead};

use crate::{
    interface::{WriteToRegister, ReadFromRegister},
    registers::{ReadReg, ConfigReg}, 
    BMA400, Config, BMA400Error,
};

// This is set by the SDO Pin level. (p. 108 of datasheet)
//#[cfg(feature = "i2c-default")]
pub const ADDR: u8 = 0b00010100;
#[cfg(feature = "i2c-alt")]
pub const ADDR: u8 = 0b00010101;

// Wrapper class to instantiate BMA400 with an I2C interface 
// (extending the Write and WriteRead traits to WriteToRegister and ReadFromRegister)
pub struct I2CInterface<I2C> {
    i2c: I2C,
}

impl<I2C> WriteToRegister for I2CInterface<I2C>
where
    I2C: Write
{
    type Error = BMA400Error<I2C::Error>;

    fn write_register<T: ConfigReg>(&mut self, register: T) -> Result<(), Self::Error> {
        self.i2c.write(ADDR, &[T::ADDR, register.to_byte()]).map_err(|e| BMA400Error::IOError(e))
    }
}

impl<I2C> ReadFromRegister for I2CInterface<I2C>
where
    I2C: WriteRead
{
    type Error = BMA400Error<I2C::Error>;
    fn read_register<T: ReadReg>(&mut self, register: T, buffer: &mut [u8]) -> Result<(), Self::Error> {
        self.i2c.write_read(ADDR, &[register.addr()], buffer).map_err(|e| BMA400Error::IOError(e))
    }
}

impl<I2C, E> BMA400<I2C>
where
    I2C: WriteRead<Error = E> + Write<Error = E>,
    I2CInterface<I2C>: ReadFromRegister + WriteToRegister,
{
    pub fn new_i2c(i2c: I2C) -> Result<BMA400<I2CInterface<I2C>>, BMA400Error<E>> {
        let interface = I2CInterface { i2c };
        let config = Config::default();
        Ok(BMA400 { interface, config })
    }
}