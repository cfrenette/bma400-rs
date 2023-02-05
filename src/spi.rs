use embedded_hal as hal;
use hal::blocking::spi::{Write, Transfer};


use crate::{
    interface::{WriteToRegister, ReadFromRegister},
    registers::Register, 
    BMA400,
};

// Wrapper class to instantiate BMA400 with an SPI interface 
// (extending the Write and WriteRead traits to WriteToRegister and ReadFromRegister)
pub struct SPIInterface<SPI> {
    interface: SPI,
}

impl<SPI, E> WriteToRegister for SPIInterface<SPI> 
where
    SPI: Write<u8, Error = E>,

{
    type Error = E;

    fn write_register(&mut self, register: Register, data: u8) -> Result<(), Self::Error> {
        todo!()
    }
    
}

impl<SPI, E> ReadFromRegister for SPIInterface<SPI> 
where
    SPI: Transfer<u8, Error = E>,

{
    type Error = E;

    fn read_register(&mut self, register: Register, buffer: &mut [u8]) -> Result<(), Self::Error> {
        todo!()
    }

    
}

impl<SPI, E> BMA400<SPI>
where
    SPI: Transfer<u8, Error = E> + Write<u8, Error = E>,
    SPIInterface<SPI>: ReadFromRegister + WriteToRegister,
{
    pub fn new_spi(mut interface: SPI) -> Result<BMA400<SPIInterface<SPI>>, E> {
        Self::init(&mut interface)?;
        Ok(BMA400 { interface: SPIInterface { interface } })
    }
    fn init(interface: &mut SPI) -> Result<(), E> {
        // Something involving a dummy read of CHIP_ID
        todo!()
    }
}