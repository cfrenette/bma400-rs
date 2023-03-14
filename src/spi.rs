use embedded_hal::digital::v2::OutputPin;

use crate::{
    hal::blocking::spi::{
        Transfer,
        Write,
    },
    interface::{
        ReadFromRegister,
        WriteToRegister,
    },
    registers::{
        ChipId,
        ConfigReg,
        InterfaceConfig,
        ReadReg,
    },
    BMA400Error,
    Config,
    BMA400,
};

/// SPI Interface wrapper
// Wrapper class to instantiate BMA400 with an SPI interface
// (extending the Write and WriteRead traits to WriteToRegister and ReadFromRegister)
#[derive(Debug)]
pub struct SPIInterface<SPI, CSBPin> {
    spi: SPI,
    csb: CSBPin,
}

impl<SPI, CSBPin, InterfaceError, PinError> WriteToRegister for SPIInterface<SPI, CSBPin>
where
    SPI: Write<u8, Error = InterfaceError>,
    CSBPin: OutputPin<Error = PinError>,
{
    type Error = BMA400Error<InterfaceError, PinError>;

    fn write_register<T: ConfigReg>(&mut self, register: T) -> Result<(), Self::Error> {
        self.csb.set_low().map_err(BMA400Error::ChipSelectPinError)?;
        self.spi.write(&[register.addr(), register.to_byte()]).map_err(BMA400Error::IOError)?;
        self.csb.set_high().map_err(BMA400Error::ChipSelectPinError)?;
        Ok(())
    }
}

impl<SPI, CSBPin, InterfaceError, PinError> ReadFromRegister for SPIInterface<SPI, CSBPin>
where
    SPI: Transfer<u8, Error = InterfaceError>,
    CSBPin: OutputPin<Error = PinError>,
{
    type Error = BMA400Error<InterfaceError, PinError>;

    fn read_register<T: ReadReg>(&mut self, register: T, buffer: &mut [u8]) -> Result<(), Self::Error> {
        self.csb.set_low().map_err(BMA400Error::ChipSelectPinError)?;
        self.spi.transfer(&mut [register.addr() | 1 << 7, 0]).map_err(BMA400Error::IOError)?;
        self.spi.transfer(buffer).map_err(BMA400Error::IOError)?;
        self.csb.set_high().map_err(BMA400Error::ChipSelectPinError)?;
        Ok(())
    }
}

impl<SPI, CSBPin, InterfaceError, PinError> BMA400<SPIInterface<SPI, CSBPin>>
where
    SPI: Transfer<u8, Error = InterfaceError> + Write<u8, Error = InterfaceError>,
    CSBPin: OutputPin<Error = PinError>,
{
    /// Create a new instance of the BMA400 using 4-wire SPI
    /// 
    /// # Examples
    /// ```
    /// # use embedded_hal_mock::{
    /// # spi::{Mock, Transaction},
    /// # pin::{Mock as MockPin, Transaction as PinTransaction, State},
    /// # };
    /// use bma400::BMA400;
    /// # let expected_io = vec![
    /// #   Transaction::transfer(vec![0x80, 0x00], vec![0x00,0x00]),
    /// #   Transaction::transfer(vec![0x00], vec![0x00]),
    /// #   Transaction::transfer(vec![0x80, 0x00], vec![0x00, 0x00]),
    /// #   Transaction::transfer(vec![0x00], vec![0x90]),
    /// # ];
    /// # let expected_pin = vec![
    /// #   PinTransaction::set(State::Low),
    /// #   PinTransaction::set(State::High),
    /// #   PinTransaction::set(State::Low),
    /// #   PinTransaction::set(State::High),
    /// # ];
    /// # let spi = Mock::new(&expected_io);
    /// # let csb_pin = MockPin::new(&expected_pin);
    /// // spi implements embedded-hal spi::Transfer and spi::Write
    /// // csb_pin implements embedded-hal digital::v2::OutputPin
    /// let mut accelerometer = BMA400::new_spi(spi, csb_pin);
    /// assert!(accelerometer.is_ok());
    /// ```
    pub fn new_spi(spi: SPI, csb: CSBPin) -> Result<BMA400<SPIInterface<SPI, CSBPin>>, BMA400Error<InterfaceError, PinError>> {
        let mut interface = SPIInterface { spi, csb };
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
    /// # use embedded_hal_mock::{
    /// # spi::{Mock, Transaction},
    /// # pin::{Mock as MockPin, Transaction as PinTransaction, State},
    /// # };
    /// use bma400::BMA400;
    /// # let expected_io = vec![
    /// #   Transaction::transfer(vec![0x80, 0x00], vec![0x00,0x00]),
    /// #   Transaction::transfer(vec![0x00], vec![0x00]),
    /// #   Transaction::transfer(vec![0x80, 0x00], vec![0x00, 0x00]),
    /// #   Transaction::transfer(vec![0x00], vec![0x90]),
    /// #   Transaction::write(vec![0x7C, 0x01]),
    /// # ];
    /// # let expected_pin = vec![
    /// #   PinTransaction::set(State::Low),
    /// #   PinTransaction::set(State::High),
    /// #   PinTransaction::set(State::Low),
    /// #   PinTransaction::set(State::High),
    /// #   PinTransaction::set(State::Low),
    /// #   PinTransaction::set(State::High),
    /// # ];
    /// # let spi = Mock::new(&expected_io);
    /// # let csb_pin = MockPin::new(&expected_pin);
    /// // spi implements embedded-hal spi::Transfer and spi::Write
    /// // csb_pin implements embedded-hal digital::v2::OutputPin
    /// let mut accelerometer = BMA400::new_spi_3wire(spi, csb_pin);
    /// assert!(accelerometer.is_ok());
    /// ```
    pub fn new_spi_3wire(spi: SPI, csb: CSBPin) -> Result<BMA400<SPIInterface<SPI, CSBPin>>, BMA400Error<InterfaceError, PinError>> {
        let mut interface = SPIInterface { spi, csb };
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
