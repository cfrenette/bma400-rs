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
    pub fn new_spi_3wire(spi: SPI, csb: CSBPin) -> Result<BMA400<SPIInterface<SPI, CSBPin>>, BMA400Error<InterfaceError, PinError>> {
        let mut interface = SPIInterface { spi, csb };
        let config = Config::default();
        // Initialize SPI Mode by doing a dummy read
        interface.read_register(ChipId, &mut [0u8; 1])?;
        let mut if_config = InterfaceConfig::default();
        if_config = if_config.with_spi_3wire_mode(true);
        interface.write_register(if_config)?;
        let mut chip_id = [0u8; 1];
        interface.read_register(ChipId, &mut chip_id)?;
        if chip_id[0] != 0x90 {
            Err(BMA400Error::ChipIdReadFailed)
        } else {
            Ok(BMA400 { interface, config })
        }
    }
}
