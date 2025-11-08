use crate::{
    BMA400, BMA400Error, Config,
    embedded_hal::spi::{Operation, SpiDevice},
    embedded_hal_async::spi::SpiDevice as AsyncSpiDevice,
    interface::{AsyncReadFromRegister, AsyncWriteToRegister, ReadFromRegister, WriteToRegister},
    registers::{ChipId, ConfigReg, InterfaceConfig, ReadReg},
};

/// SPI Interface wrapper
// Wrapper class to instantiate BMA400 with an SPI interface
// (extending the Write and WriteRead traits to WriteToRegister and ReadFromRegister)
#[derive(Debug)]
pub struct SPIInterface<SPI> {
    spi: SPI,
}

impl<SPI: SpiDevice> SPIInterface<SPI> {
    /// Consumes the Interface returning underlying SPI peripheral and the pin
    pub fn destroy(self) -> SPI {
        self.spi
    }
}

impl<SPI> WriteToRegister for SPIInterface<SPI>
where
    SPI: SpiDevice,
{
    type Error = BMA400Error<SPI::Error>;

    fn write_register<T: ConfigReg>(&mut self, register: T) -> Result<(), Self::Error> {
        self.spi
            .write(&[register.addr(), register.to_byte()])
            .map_err(BMA400Error::IOError)?;
        Ok(())
    }
}

impl<SPI> ReadFromRegister for SPIInterface<SPI>
where
    SPI: SpiDevice,
{
    type Error = BMA400Error<SPI::Error>;

    fn read_register<T: ReadReg>(
        &mut self,
        register: T,
        buffer: &mut [u8],
    ) -> Result<(), Self::Error> {
        self.spi
            .transaction(&mut [
                Operation::Write(&[register.addr() | 1 << 7, 0]),
                Operation::Read(buffer),
            ])
            .map_err(BMA400Error::IOError)?;
        Ok(())
    }
}

impl<SPI> BMA400<SPIInterface<SPI>>
where
    SPI: SpiDevice,
{
    /// Create a new instance of the BMA400 using 4-wire SPI
    ///
    /// # Examples
    /// ```
    /// # use embedded_hal_mock::eh1::spi::{Mock, Transaction};
    /// use bma400::BMA400;
    /// # let expected_io = vec![
    /// #   Transaction::transaction_start(),
    /// #   Transaction::write_vec(vec![0x80, 0x00]),
    /// #   Transaction::read_vec(vec![0x00]),
    /// #   Transaction::transaction_end(),
    /// #   Transaction::transaction_start(),
    /// #   Transaction::write_vec(vec![0x80, 0x00]),
    /// #   Transaction::read_vec(vec![0x90]),
    /// #   Transaction::transaction_end(),
    /// # ];
    /// # let mut spi = Mock::new(&expected_io);
    /// // spi implements embedded-hal spi::SpiDevice
    /// let mut accelerometer = BMA400::new_spi(&mut spi);
    /// assert!(accelerometer.is_ok());
    /// # spi.done();
    /// ```
    pub fn new_spi(spi: SPI) -> Result<BMA400<SPIInterface<SPI>>, BMA400Error<SPI::Error>> {
        let mut interface = SPIInterface { spi };
        let config = Config::default();
        // Initialize SPI Mode by doing a dummy read
        interface.read_register(ChipId, &mut [0u8; 1])?;
        // Validate Chip ID
        let mut chip_id = [0u8; 1];
        interface.read_register(ChipId, &mut chip_id)?;
        if chip_id[0] != 0x90 {
            Err(BMA400Error::ChipIdReadFailed)
        } else {
            Ok(BMA400 { interface, config })
        }
    }
    /// Create a new instance of the BMA400 using 3-wire SPI
    ///
    /// # Examples
    /// ```
    /// # use embedded_hal_mock::eh1::spi::{Mock, Transaction};
    /// use bma400::BMA400;
    /// # let expected_io = vec![
    /// #   Transaction::transaction_start(),
    /// #   Transaction::write_vec(vec![0x80, 0x00]),
    /// #   Transaction::read_vec(vec![0x00]),
    /// #   Transaction::transaction_end(),
    /// #   Transaction::transaction_start(),
    /// #   Transaction::write_vec(vec![0x80, 0x00]),
    /// #   Transaction::read_vec(vec![0x90]),
    /// #   Transaction::transaction_end(),
    /// #   Transaction::transaction_start(),
    /// #   Transaction::write_vec(vec![0x7C, 0x01]),
    /// #   Transaction::transaction_end(),
    /// # ];
    /// # let mut spi = Mock::new(&expected_io);
    /// // spi implements embedded-hal spi::SpiDevice
    /// let mut accelerometer = BMA400::new_spi_3wire(&mut spi);
    /// assert!(accelerometer.is_ok());
    /// # spi.done();
    /// ```
    pub fn new_spi_3wire(spi: SPI) -> Result<BMA400<SPIInterface<SPI>>, BMA400Error<SPI::Error>> {
        let mut interface = SPIInterface { spi };
        let config = Config::default();
        // Initialize SPI Mode by doing a dummy read
        interface.read_register(ChipId, &mut [0u8; 1])?;
        let mut chip_id = [0u8; 1];
        interface.read_register(ChipId, &mut chip_id)?;
        let if_config = InterfaceConfig::default().with_spi_3wire_mode(true);
        interface.write_register(if_config)?;
        if chip_id[0] != 0x90 {
            Err(BMA400Error::ChipIdReadFailed)
        } else {
            Ok(BMA400 { interface, config })
        }
    }
}
