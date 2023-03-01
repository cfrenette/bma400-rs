use core::fmt::Debug;
use embedded_hal::digital::v2::OutputPin;
use crate::hal::blocking::spi::{Write, Transfer};
use crate::{
    interface::{WriteToRegister, ReadFromRegister},
    registers::{ReadReg, ConfigReg, ChipId}, 
    BMA400, Config, BMA400Error
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
    InterfaceError: Debug,
    PinError: Debug,
{
    type Error = BMA400Error<InterfaceError, PinError>;

    fn write_register<T: ConfigReg>(&mut self, register: T) -> Result<(), Self::Error> {
        self.csb.set_low().map_err(|e| BMA400Error::ChipSelectPinError(e))?;
        self.spi.write(&[register.addr(), register.to_byte()]).map_err(|e| BMA400Error::IOError(e))?;
        self.csb.set_high().map_err(|e| BMA400Error::ChipSelectPinError(e))?;
        Ok(())
    }
}

impl<SPI, CSBPin, InterfaceError, PinError> ReadFromRegister for SPIInterface<SPI, CSBPin> 
where
    SPI: Transfer<u8, Error = InterfaceError>,
    CSBPin: OutputPin<Error = PinError>,
    InterfaceError: Debug,
    PinError: Debug,
{
    type Error = BMA400Error<InterfaceError, PinError>;

    fn read_register<T: ReadReg>(&mut self, register: T, buffer: &mut [u8]) -> Result<(), Self::Error> {
        let first_byte = register.addr() | 1 << 7;
        buffer[0] = first_byte;
        self.csb.set_low().map_err(|e| BMA400Error::ChipSelectPinError(e))?;
        self.spi.transfer(buffer).map_err(|e| BMA400Error::IOError(e))?;
        self.csb.set_high().map_err(|e| BMA400Error::ChipSelectPinError(e))?;
        Ok(())
    }
}

impl<SPI, CSBPin, InterfaceError, PinError> BMA400<SPIInterface<SPI, CSBPin>>
where
    SPI: Transfer<u8, Error = InterfaceError> + Write<u8, Error = InterfaceError>,
    CSBPin: OutputPin<Error = PinError>,
    SPIInterface<SPI, CSBPin>: ReadFromRegister<Error = BMA400Error<InterfaceError, PinError>> + WriteToRegister<Error = BMA400Error<InterfaceError, PinError>>,
    InterfaceError: Debug,
    PinError: Debug,
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
}
