use crate::hal::blocking::i2c::{Write, WriteRead};
use crate::registers::ChipId;
use crate::{
    interface::{WriteToRegister, ReadFromRegister},
    registers::{ReadReg, ConfigReg}, 
    BMA400, Config, BMA400Error,
};

// This is set by the SDO Pin level. (p. 108 of datasheet)
#[cfg(any(feature = "i2c-default", test))]
pub const ADDR: u8 = 0b00010100;
#[cfg(feature = "i2c-alt")]
pub const ADDR: u8 = 0b00010101;

// Wrapper class to instantiate BMA400 with an I2C interface 
// (extending the Write and WriteRead traits to WriteToRegister and ReadFromRegister)
#[derive(Debug)]
pub struct I2CInterface<I2C> {
    i2c: I2C,
}

impl<I2C, E> WriteToRegister for I2CInterface<I2C>
where
    I2C: Write<Error = E>,
{
    type Error = BMA400Error<E, ()>;

    fn write_register<T: ConfigReg>(&mut self, register: T) -> Result<(), Self::Error> {
        self.i2c.write(ADDR, &[register.addr(), register.to_byte()]).map_err(BMA400Error::IOError)
    }
}

impl<I2C, E> ReadFromRegister for I2CInterface<I2C> 
where
    I2C: WriteRead<Error = E>,
{
    type Error = BMA400Error<E, ()>;

    fn read_register<T: ReadReg>(&mut self, register: T, buffer: &mut [u8]) -> Result<(), Self::Error> {
        self.i2c.write_read(ADDR, &[register.addr()], buffer).map_err(BMA400Error::IOError)
    }
}

impl<I2C, E> BMA400<I2CInterface<I2C>>
where
    I2C: WriteRead<Error = E> + Write<Error = E>,
{
    pub fn new_i2c(i2c: I2C) -> Result<BMA400<I2CInterface<I2C>>, BMA400Error<E, ()>> {
        let mut interface = I2CInterface { i2c };
        let config = Config::default();
        let mut chip_id = [0u8; 1];
        interface.read_register(ChipId, &mut chip_id)?;
        if chip_id[0] != 0x90 {
            Err(BMA400Error::ChipIdReadFailed)
        } else {
            Ok(BMA400 { interface, config })
        }
    }
}
