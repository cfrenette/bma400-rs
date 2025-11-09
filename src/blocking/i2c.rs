use crate::{
    BMA400, BMA400Error, Config, I2CAddr, I2CInterface,
    blocking::{ReadFromRegister, WriteToRegister},
    embedded_hal::i2c::{I2c, SevenBitAddress},
    registers::{ChipId, ConfigReg, ReadReg},
};

impl<I2C> WriteToRegister for I2CInterface<I2C>
where
    I2C: I2c<SevenBitAddress>,
{
    type Error = BMA400Error<I2C::Error>;

    fn write_register<T: ConfigReg>(&mut self, register: T) -> Result<(), Self::Error> {
        self.i2c
            .write(Self::ADDR, &[register.addr(), register.to_byte()])
            .map_err(BMA400Error::IOError)
    }
}

impl<I2C> ReadFromRegister for I2CInterface<I2C>
where
    I2C: I2c<SevenBitAddress>,
{
    type Error = BMA400Error<I2C::Error>;

    fn read_register<T: ReadReg>(
        &mut self,
        register: T,
        buffer: &mut [u8],
    ) -> Result<(), Self::Error> {
        self.i2c
            .write_read(Self::ADDR, &[register.addr()], buffer)
            .map_err(BMA400Error::IOError)
    }
}

impl<I2C> BMA400<I2CInterface<I2C>>
where
    I2C: I2c<SevenBitAddress>,
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
    pub fn new_i2c(i2c: I2C) -> Result<BMA400<I2CInterface<I2C>>, BMA400Error<I2C::Error>> {
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
