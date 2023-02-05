use embedded_hal as hal;
use hal::blocking::i2c::{Write, WriteRead};

use crate::{
    interface::{WriteToRegister, ReadFromRegister},
    registers::Register, 
    BMA400, ConfigBuilder, Config,
};

// TODO: Depends on state of SDO Pin
pub const ADDR: u8 = 0b00010100;

// Wrapper class to instantiate BMA400 with an I2C interface 
// (extending the Write and WriteRead traits to WriteToRegister and ReadFromRegister)
pub struct I2CInterface<I2C> {
    i2c: I2C,
}

impl<I2C, E> WriteToRegister for I2CInterface<I2C>
where
    I2C: Write<Error = E>
{
    type Error = E;

    fn write_register(&mut self, register: Register, data: u8) -> Result<(), Self::Error> {
        // TODO sequential write
        self.i2c.write(ADDR, &[u8::from(register), data])
    }

    fn write_registers(&mut self, registers: &[Register], data: &[u8]) -> Result<(), Self::Error> {
        for (register, &data) in registers.iter().map(|r| u8::from(*r)).zip(data.iter()) {
            self.i2c.write(ADDR, &[register, data])?;
        }
        Ok(())
    }
    
}

impl<I2C, E> ReadFromRegister for I2CInterface<I2C>
where
    I2C: WriteRead<Error = E>
{
    type Error = E;

    fn read_register(&mut self, register: Register, buffer: &mut [u8]) -> Result<(), Self::Error> {
        self.i2c.write_read(ADDR, &[u8::from(register)], buffer)
    }

    fn read_registers(&mut self, registers: &[u8], buffer: &mut [u8]) -> Result<(), Self::Error> {
        self.i2c.write_read(ADDR, registers, buffer)
    }
    
}

impl<I2C, E> BMA400<I2C>
where
    I2C: WriteRead + Write,
    I2CInterface<I2C>: ReadFromRegister<Error = E> + WriteToRegister<Error = E>,
{
    pub fn new_i2c(i2c: I2C) -> Result<BMA400<I2CInterface<I2C>>, E> {
        let mut interface = I2CInterface { i2c };
        let mut config = ConfigBuilder::default().build();
        Self::init(&mut interface, &mut config)?;
        Ok(BMA400 { interface, config })
    }
    pub fn new_i2c_with_config(i2c: I2C, config_builder: ConfigBuilder) -> Result<BMA400<I2CInterface<I2C>>, E> {
        let mut interface = I2CInterface { i2c };
        let mut config = config_builder.build();
        Self::init(&mut interface, &mut config)?;
        Ok(BMA400 {interface, config})
    }
    fn init(interface: &mut I2CInterface<I2C>, config: &mut Config) -> Result<(), E> {
        interface.write_registers(&[Register::AccConfig0, Register::AccConfig1, Register::AccConfig2], config.to_bytes())?;
        Ok(())
    }
}