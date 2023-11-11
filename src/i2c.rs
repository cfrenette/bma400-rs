use embedded_hal::i2c::SevenBitAddress;

use crate::{
    hal::i2c::I2c,
    interface::{ReadFromRegister, WriteToRegister},
    registers::{ChipId, ConfigReg, ReadReg},
    BMA400Error, Config, BMA400,
};

// This is set by the SDO Pin level. (p. 108 of datasheet)
#[cfg(any(feature = "i2c-default", test))]
pub const ADDR: u8 = 0b00010100;
#[cfg(feature = "i2c-alt")]
pub const ADDR: u8 = 0b00010101;

/// I²C Interface wrapper
// Wrapper class to instantiate BMA400 with an I²C interface
// (extending the Write and WriteRead traits to WriteToRegister and ReadFromRegister)
#[derive(Debug)]
pub struct I2CInterface<I2C> {
    i2c: I2C,
}

impl<I2C> I2CInterface<I2C> {
    /// Consumes the Interface returning underlying I²C peripheral
    pub fn destroy(self) -> I2C {
        self.i2c
    }
}

impl<I2C, E> WriteToRegister for I2CInterface<I2C>
where
    I2C: I2c<SevenBitAddress, Error = E>,
{
    type Error = BMA400Error<E, ()>;

    fn write_register<T: ConfigReg>(&mut self, register: T) -> Result<(), Self::Error> {
        self.i2c
            .write(ADDR, &[register.addr(), register.to_byte()])
            .map_err(BMA400Error::IOError)
    }
}

impl<I2C, E> ReadFromRegister for I2CInterface<I2C>
where
    I2C: I2c<SevenBitAddress, Error = E>,
{
    type Error = BMA400Error<E, ()>;

    fn read_register<T: ReadReg>(
        &mut self,
        register: T,
        buffer: &mut [u8],
    ) -> Result<(), Self::Error> {
        self.i2c
            .write_read(ADDR, &[register.addr()], buffer)
            .map_err(BMA400Error::IOError)
    }
}

impl<I2C, E> BMA400<I2CInterface<I2C>>
where
    I2C: I2c<SevenBitAddress, Error = E>,
{
    /// Create a new instance of the BMA400 using I²C
    ///
    /// # Examples
    /// ```
    /// # use embedded_hal_mock::eh1::i2c::{Mock, Transaction};
    /// use bma400::BMA400;
    /// # let expected = vec![Transaction::write_read(0b10100, vec![0x00], vec![0x90])];
    /// # let mut i2c = Mock::new(&expected);
    /// // i2c implements embedded-hal i2c::WriteRead and i2c::Write
    /// let mut accelerometer = BMA400::new_i2c(&mut i2c);
    /// assert!(accelerometer.is_ok());
    /// # i2c.done();
    /// ```
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
