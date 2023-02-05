#![no_std]

use core::fmt::Debug;
use embedded_hal as hal;
use accelerometer::{Accelerometer, vector::F32x3, Error as AccError};
mod interface;
use interface::{ReadFromRegister, WriteToRegister};
pub(crate) mod registers;
use registers::Register;

#[cfg(feature = "i2c")]
pub mod i2c;

#[cfg(feature = "spi")]
pub mod spi;

pub struct ConfigBuilder {
    acc_config0: u8,
    acc_config1: u8,
    acc_config2: u8,
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        Self { 
            acc_config0: 0b0000_0000,
            acc_config1: 0b0100_1001, 
            acc_config2: 0b0000_0000, 
        }
    }
}

impl ConfigBuilder {
    pub fn with_odr(mut self, odr: OutputDataRate) -> Self {
        match odr {
            OutputDataRate::Hz100 => {},
            _ => {
                // Need to switch to Filter1
                self.acc_config2 = self.acc_config2 & 0b1111_0011;
            }
        }
        self.acc_config1 = self.acc_config1 & 0b1111_0000 | match odr {
            OutputDataRate::Hz12_5 => 0x05,
            OutputDataRate::Hz25 => 0x06,
            OutputDataRate::Hz50 => 0x07,
            OutputDataRate::Hz100 => 0x08,
            OutputDataRate::Hz200 => 0x09,
            OutputDataRate::Hz400 => 0x0A,
            OutputDataRate::Hz800 => 0x0B,
        };
        self
        
    }
    pub fn with_scale(mut self, sensitivity: Scale) -> Self {
        self.acc_config1 = self.acc_config1 & 0b0011_1111 | (match sensitivity {
            Scale::Range2G => 0x00,
            Scale::Range4G => 0x01,
            Scale::Range8G => 0x02,
            Scale::Range16G => 0x03,
        }) << 6;
        self
    }
    pub fn with_power_mode(mut self, mode: PowerMode) -> Self {
        self.acc_config0 = (self.acc_config0 & 0b1111_1100) | match mode {
            PowerMode::Sleep => 0x00,
            PowerMode::LowPower => 0x01,
            PowerMode::Normal => 0x02,
        };
        self
    }
    pub(crate) fn build(self) -> Config {
        Config {
            acc_config: [self.acc_config0, self.acc_config1, self.acc_config2],
        }
    }
}

struct Config {
    acc_config: [u8; 3],
}

impl Config {
    pub fn to_bytes(&self) -> &[u8] {
        &self.acc_config
    }
}

#[derive(Debug)]
pub enum PowerMode {
    Sleep,
    LowPower,
    Normal,
}

pub enum InterruptPin {
    INT1,
    INT2,
}

/// The Measurement scale of the accelerometer
/// 
/// 2g/4g/8g/16g
pub enum Scale {
    /// -2g to 2g
    Range2G = 0x00,
    /// -4g to 4g
    Range4G = 0x01,
    /// -8g to 8g
    Range8G = 0x02,
    /// -16g to 16g
    Range16G = 0x03,
}

/// Data Source Configuration
/// 
/// [Filter::Filter1] allows a selectable [OutputDataRate] (12.5 to 800 Hz) and a choice of two bandwidth settings: [Bandwidth::High] and [Bandwidth::Low]
/// 
/// [Filter::Filter2] allows only a fixed [OutputDataRate] of 100Hz. An additional low pass filter with a bandwidth of 1Hz can be applied but this will disable the Data Buffer
#[derive(Debug)]
pub enum Filter {
    /// Selectable ODR, Selectable Bandwidth
    Filter1,
    /// Fixed ODR of 100Hz, Enable addl low-pass filter at the cost of disabling Data Buffering
    Filter2,
}

pub enum Bandwidth {
    /// 0.48 x [OutputDataRate] Hz
    High,
    /// 0.24 x [OutputDataRate] Hz
    Low,
}

/// Output Data Rate in Hz
pub enum OutputDataRate {
    /// 12.5 Hz
    Hz12_5 = 0x05,
    /// 25 Hz
    Hz25 = 0x06,
    Hz50 = 0x07,
    Hz100 = 0x08,
    Hz200 = 0x09,
    Hz400 = 0x0A,
    Hz800 = 0x0B,
}

#[derive(Debug)]
/// Oversample Rate
/// 
/// Higher values reduce data noise at the cost of power consumption
pub enum OversampleRate {
    /// Lowest Precision / Power Draw
    /// 
    /// [PowerMode::LowPower] 0.85uA
    /// 
    /// [PowerMode::Normal]  3.5uA
    OSR0,
    /// [PowerMode::LowPower] 0.93uA 
    /// 
    /// [PowerMode::Normal] 5.8uA 
    OSR1,
    /// [PowerMode::LowPower] 1.1uA
    /// 
    /// [PowerMode::Normal] 9.5uA 
    OSR2,
    /// Highest Precision / Power Draw
    /// 
    /// [PowerMode::LowPower] 1.35uA 
    /// 
    /// [PowerMode::Normal] 14.5uA 
    OSR3,
}

pub struct Status {
    pub int_active: bool,
    pub power_mode_stat: PowerMode,
    pub cmd_rdy: bool,
    pub drdy_stat: bool,
}

impl Status {
    pub fn new(status_byte: u8) -> Self {
        let int_mask = 0b00000001;
        let power_mode_mask = 0b00000110;
        let cmd_rdy_mask = 0b00010000;
        let drdy_mask = 0b10000000;
        Status {
            int_active: (status_byte & int_mask) != 0,
            power_mode_stat: match status_byte & power_mode_mask >> 1 {
                0 => PowerMode::Sleep,
                1 => PowerMode::LowPower,
                _ => PowerMode::Normal,
            },
            cmd_rdy: (status_byte & cmd_rdy_mask) != 0,
            drdy_stat: (status_byte & drdy_mask) != 0,
        }
    }
}

/// The behavior of the BMA400's circular buffer
pub enum BufferMode {
    /// Last-In, First-Out
    LIFO,
    /// First-In, First-Out
    FIFO,
}

#[derive(Debug)]
pub struct DataBuffer {

}

pub enum Axis {
    X,
    Y,
    Z,
}

pub enum FrameType {
    Data(Axis),
    Time,
    Empty,
}

#[derive(Debug)]
pub struct BufferFrame {

}


/// A 3-axis acceleration measurement
#[derive(Debug)]
pub struct Measurement {
    /// x-axis data
    pub x: i16,
    /// y-axis data
    pub y: i16,
    /// z-axis data
    pub z: i16,
}

impl Measurement {
    pub fn new(x_lsb: u8, x_msb: u8, y_lsb: u8, y_msb: u8, z_lsb: u8, z_msb: u8) -> Self {
        let twelve_to_sixteen = |lsb, msb| {
            let msb_no_garbage = msb & 0b0000_1111;
            i16::from_le_bytes([lsb, if msb_no_garbage >> 3 == 0u8 { msb_no_garbage } else { msb_no_garbage | (0b1111_0000) }])
        };
        Measurement {
            x: twelve_to_sixteen(x_lsb, x_msb),
            y: twelve_to_sixteen(y_lsb, y_msb),
            z: twelve_to_sixteen(z_lsb, z_msb),
        }
    }
}

pub struct BMA400<T> {
    interface: T,
    config: Config,
}

impl<T, E> BMA400<T> 
where
    T: ReadFromRegister<Error = E> + WriteToRegister<Error = E>,
    E: Debug,
{
    pub fn get_id(&mut self) -> Result<u8, E> {
        let mut id = [0u8; 1];
        self.interface.read_register(Register::ChipId, &mut id)?;
        Ok(id[0])
    }

    pub fn get_config(&self) -> ConfigBuilder {
        ConfigBuilder {
            acc_config0: self.config.acc_config[0],
            acc_config1: self.config.acc_config[1],
            acc_config2: self.config.acc_config[2],
        }
    }

    pub fn get_status(&mut self) -> Result<Status, E> {
        let mut status_byte = [0u8; 1];
        self.interface.read_register(Register::Status, &mut status_byte)?;
        Ok(Status::new(status_byte[0]))
    }

    pub fn get_error(&mut self) -> Result<bool, E> {
        let mut err_byte = [0u8; 1];
        self.interface.read_register(Register::ErrReg, &mut err_byte)?;
        Ok(err_byte[0] & 0b00000010 != 0)
    }

    pub fn get_data(&mut self) -> Result<Measurement, E> {
        let mut bytes = [0u8; 6];
        self.interface.read_register(Register::AccXLSB, &mut bytes)?;
        Ok(Measurement::new(bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5]))
    }

    /// Time from the integrated sensor timer. The timer has a resolution of 21 bits stored across 3 bytes.
    /// The lowest 3 bits are always zero (the value is left-justified for compatibility with 25.6kHz clocks).
    /// This timer is inactive in sleep mode. The clock resets to zero after 0xFFFFF8
    pub fn get_sensor_clock(&mut self) -> Result<u32, E> {
        let mut buffer = [0u8; 3];
        self.interface.read_register(Register::SensorTime0, &mut buffer)?;
        let bytes = [buffer[0], buffer[1], buffer[2], 0];
        Ok(u32::from_le_bytes(bytes))
    }

    /// Chip Temperature represented as an i8 with 0.5℃ precision.
    /// 
    /// -128 (-40.0℃) to 
    /// 127 (87.5℃)
    pub fn get_raw_temp(&mut self) -> Result<i8, E> {
        let mut temp = [0u8; 1];
        self.interface.read_register(Register::TempData, &mut temp)?;
        let t = i8::from_le_bytes(temp);
        Ok(t)
    }

    /// Chip Temperature with 0.5℃ Precision
    pub fn get_temp_celsius(&mut self) -> Result<f32, E> {
        Ok(f32::from(self.get_raw_temp()?)*0.5 + 23.0)
    }

    pub fn set_config(&mut self, config_builder: ConfigBuilder) -> Result<(), E> {
        let config = config_builder.build();
        self.interface.write_registers(&[Register::AccConfig0, Register::AccConfig1, Register::AccConfig2], config.to_bytes())?;
        self.config = config;
        Ok(())
    }

    pub fn destroy(self) -> T {
        self.interface
    }
}

impl<T, E> Accelerometer for BMA400<T> 
where
    T: ReadFromRegister<Error = E> + WriteToRegister<Error = E>,
    E: Debug,
{
    type Error = AccError<E>;

    fn accel_norm(&mut self) -> Result<F32x3, AccError<Self::Error>> {
        todo!()
    }

    fn sample_rate(&mut self) -> Result<f32, AccError<Self::Error>> {
        todo!()
    }
}

