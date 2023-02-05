use crate::registers::Register;

pub trait WriteToRegister {
    type Error;
    fn write_register(&mut self, register: Register, data: u8) -> Result<(), Self::Error>;
    fn write_registers(&mut self, registers: &[Register], data: &[u8]) -> Result<(), Self::Error>;
}

pub trait ReadFromRegister {
    type Error;
    fn read_register(&mut self, register: Register, buffer: &mut [u8]) -> Result<(), Self::Error>;
    fn read_registers(&mut self, registers: &[u8], buffer: &mut [u8]) -> Result<(), Self::Error>;
}


//TODO
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}