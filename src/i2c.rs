use embedded_hal::i2c::SevenBitAddress;

use crate::{
    hal::i2c::I2c,
    interface::{ReadFromRegister, WriteToRegister},
    registers::{ChipId, ConfigReg, ReadReg},
    BMA400Error, Config, BMA400,
};

#[cfg(feature = "async")]
use crate::{
    hal_async::i2c::I2c as AsyncI2c,
    interface::{AsyncReadFromRegister, AsyncWriteToRegister},
    AsyncBMA400,
};

#[cfg(all(not(docsrs), feature = "i2c-default", feature = "i2c-alt"))]
compile_error!("Only one of the features `i2c-default` and `i2c-alt` can be enabled");

// This is set by the SDO Pin level. (p. 108 of datasheet)
#[cfg(any(feature = "i2c-default", test))]
pub const ADDR: u8 = 0b00010100;
#[cfg(all(feature = "i2c-alt", not(docsrs), not(feature = "i2c-default")))]
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

#[cfg(feature = "async")]
impl<I2C, E> AsyncWriteToRegister for I2CInterface<I2C>
where
    I2C: AsyncI2c<SevenBitAddress, Error = E>,
{
    type Error = BMA400Error<E, ()>;

    async fn write_register<T: ConfigReg>(&mut self, register: T) -> Result<(), Self::Error> {
        self.i2c
            .write(ADDR, &[register.addr(), register.to_byte()])
            .await
            .map_err(BMA400Error::IOError)
    }
}

#[cfg(feature = "async")]
impl<I2C, E> AsyncReadFromRegister for I2CInterface<I2C>
where
    I2C: AsyncI2c<SevenBitAddress, Error = E>,
{
    type Error = BMA400Error<E, ()>;

    async fn read_register<T: ReadReg>(
        &mut self,
        register: T,
        buffer: &mut [u8],
    ) -> Result<(), Self::Error> {
        self.i2c
            .write_read(ADDR, &[register.addr()], buffer)
            .await
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

#[cfg(any(docsrs, feature = "async"))]
#[cfg_attr(docsrs, doc(cfg(feature = "async")))]
impl<I2C, E> AsyncBMA400<I2CInterface<I2C>>
where
    I2C: AsyncI2c<SevenBitAddress, Error = E>,
{
    /// async equivalent of [`BMA400::new_i2c`].
    pub async fn new_i2c(i2c: I2C) -> Result<Self, BMA400Error<E, ()>> {
        let mut interface = I2CInterface { i2c };
        let config = Config::default();
        let mut chip_id = [0u8; 1];
        interface.read_register(ChipId, &mut chip_id).await?;
        if chip_id[0] != 0x90 {
            Err(BMA400Error::ChipIdReadFailed)
        } else {
            Ok(Self { interface, config })
        }
    }
}
