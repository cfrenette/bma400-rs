
/// Error types
#[derive(Debug)]
pub enum BMA400Error<InterfaceError> {
    /// I2C / SPI Error
    IOError(InterfaceError),
    /// Incorrect configuration
    ConfigBuildError(ConfigError),
    /// Self-Test Failure
    SelfTestFailedError,
}

impl<E> From<ConfigError> for BMA400Error<E> {
    fn from(value: ConfigError) -> Self {
        Self::ConfigBuildError(value)
    }
}

/// Errors building Config
#[derive(Debug)]
pub enum ConfigError {
    /// Interrupt data source ODR must be 100Hz
    Filt1InterruptInvalidODR,
    /// Tap Interrupt data source (Filt1) ODR must be 200Hz
    TapIntEnabledInvalidODR,
    /// FIFO Read attempted with read circuit disabled
    FifoReadWhilePwrDisable,
}

/// A sensor Status reading
pub struct Status {
    bits: u8,
}

impl Status {
    pub fn new(status_byte: u8) -> Self {
        Status {
            bits: status_byte,
        }
    }
    pub fn drdy_stat(&self) -> bool {
        (self.bits & 0b1000_0000) != 0
    }
    pub fn cmd_rdy(&self) -> bool {
        (self.bits & 0b0001_0000) != 0
    }
    pub fn power_mode(&self) -> PowerMode {
        match self.bits & 0b0000_0110 >> 1 {
            0 => PowerMode::Sleep,
            1 => PowerMode::LowPower,
            _ => PowerMode::Normal,
        }
    }
    pub fn int_active(&self) -> bool {
        (self.bits & 0b0000_0001) != 0
    }
}

pub enum StepIntStatus {
    None,
    OneStepDetect,
    ManyStepDetect,
}

pub struct IntStatus0 {
    bits: u8,
}

impl IntStatus0 {
    pub fn new(status_byte: u8) -> Self {
        IntStatus0 {bits: status_byte }
    }
    pub fn drdy_stat(&self) -> bool {
        (self.bits & 0b1000_0000) != 0
    }
    pub fn fwm_stat(&self) -> bool {
        (self.bits & 0b0100_0000) != 0
    }
    pub fn ffull_stat(&self) -> bool {
        (self.bits & 0b0010_0000) != 0
    }
    pub fn ieng_overrun_stat(&self) -> bool {
        (self.bits & 0b0001_0000) != 0
    }
    pub fn gen2_stat(&self) -> bool {
        (self.bits & 0b0000_1000) != 0
    }
    pub fn gen1_stat(&self) -> bool {
        (self.bits & 0b0000_0100) != 0
    }
    pub fn orientch_stat(&self) -> bool {
        (self.bits & 0b0000_0010) != 0
    }
    pub fn wkup_stat(&self) -> bool {
        (self.bits & 0b0000_0001) != 0
    }
}

pub struct IntStatus1 {
    bits: u8,
}

impl IntStatus1 {
    pub fn new(status_byte: u8) -> Self {
        IntStatus1 { bits: status_byte }
    }
    pub fn ieng_overrun_stat(&self) -> bool {
        (self.bits & 0b0001_0000) != 0
    }
    pub fn d_tap_stat(&self) -> bool {
        (self.bits & 0b0000_1000) != 0
    }
    pub fn s_tap_stat(&self) -> bool {
        (self.bits & 0b0000_0100) != 0
    }
    pub fn step_int_stat(&self) -> StepIntStatus {
        match self.bits & 0b0000_0011 {
            0x00 => StepIntStatus::None,
            0x01 => StepIntStatus::OneStepDetect,
            _ => StepIntStatus::ManyStepDetect,
        }
    }
}

pub struct IntStatus2 {
    bits: u8,
}

impl IntStatus2 {
    pub fn new(status_byte: u8) -> Self {
        IntStatus2 { bits: status_byte }
    }
    pub fn ieng_overrun_stat(&self) -> bool {
        (self.bits & 0b0001_0000) != 0
    }
    pub fn actch_z_stat(&self) -> bool {
        (self.bits & 0b0000_0100) != 0
    }
    pub fn actch_y_stat(&self) -> bool {
        (self.bits & 0b0000_0010) != 0
    }
    pub fn actch_x_stat(&self) -> bool {
        (self.bits & 0b0000_0001) != 0
    }
}

/// A 3-axis acceleration measurement with 3 fields
/// 
/// x: x-axis data,
/// 
/// y: y-axis data,
/// 
/// z: z-axis data
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
    fn new(x: i16, y: i16, z: i16) -> Self {
        Measurement { x, y, z }
    }
    pub fn from_bytes_unscaled(bytes: &[u8]) -> Self {
        Self::new(
            Self::to_i16(bytes[0], bytes[1]),
            Self::to_i16(bytes[2], bytes[3]),
            Self::to_i16(bytes[4], bytes[5])
        )
    }
    pub fn from_bytes_scaled(scale: Scale, bytes: &[u8]) -> Self {
        let shift = match scale {
            Scale::Range2G => 0,
            Scale::Range4G => 1,
            Scale::Range8G => 2,
            Scale::Range16G => 3,
        };
        Self::new(
            Self::to_i16(bytes[0], bytes[1]) << shift,
            Self::to_i16(bytes[2], bytes[3]) << shift,
            Self::to_i16(bytes[4], bytes[5]) << shift
        )
    }
    fn to_i16(lsb: u8, msb: u8) -> i16 {
        let clear_rsvd_bits = msb & 0b0000_1111;
        i16::from_le_bytes([lsb, if clear_rsvd_bits >> 3 == 0u8 { clear_rsvd_bits } else { clear_rsvd_bits | (0b1111_0000) }])
    }
}


/// The BMA400's Hardware Interrupt Pins, Int1 and Int2
pub enum InterruptPins {
    None,
    Int1,
    Int2,
    Both,
}

/// Defines which state represents active
pub enum PinOutputLevel {
    /// Gnd
    ActiveLow,
    /// VDDIO / Hi-Z
    ActiveHigh,
}

/// Defines the interrupt pin configuration
pub enum PinOutputConfig {
    /// Gnd / VDDIO
    PushPull(PinOutputLevel),
    /// Gnd / Hi-Z
    OpenDrain(PinOutputLevel),
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
/// Select one of three possible data sources to feed the data registers and the interrupt engine. 
/// 
/// The FIFO buffer can only use either [DataSource::AccFilt1] or [DataSource::AccFilt2]
#[derive(Debug)]
pub enum DataSource {
    /// Selectable [OutputDataRate], choice of two low pass filter bandwidths 
    /// 
    /// Recommended to feed data registers / FIFO buffer
    /// (See: [Filter1Bandwidth])
    AccFilt1,
    /// Fixed [OutputDataRate] of 100Hz, fixed low pass filter bandwidth (48Hz)
    /// 
    /// Recommended for interrupts except Tap Sensing Interrupt (which requires 200Hz ODR)
    AccFilt2,
    /// Fixed [OutputDataRate] of 100Hz, fixed low pass filter bandwidth (1Hz)
    /// 
    /// Cannot be used by the FIFO buffer
    AccFilt2Lp,
}

/// Bandwidth setting for the low pass filter for AccFilt1 data source
pub enum Filter1Bandwidth {
    /// 0.48 x [OutputDataRate] Hz
    High,
    /// 0.24 x [OutputDataRate] Hz
    Low,
}

/// Output Data Rate in Hz
pub enum OutputDataRate {
    /// 12.5 Hz
    Hz12_5,
    /// 25 Hz
    Hz25,
    /// 50 Hz
    Hz50,
    /// 100 Hz
    Hz100,
    /// 200 Hz
    Hz200,
    /// 400 Hz
    Hz400,
    /// 800 Hz
    Hz800,
}

#[derive(Debug)]
/// Oversample Rate
/// 
/// Higher values reduce data noise at the cost of power consumption
/// 
/// See p. 21 of datasheet
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


pub enum PowerMode {
    Sleep,
    LowPower,
    Normal,
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

/// Automatically enter low power mode after a defined timeout
/// 
/// Non-timed triggers are still supported if timeout is disabled
/// 
/// See datasheet p.25
pub enum AutoLPTimeoutTrigger {
    /// Timed trigger to enter low power mode disabled
    TimeoutDisabled,
    /// Timed trigger to enter low power mode enabled
    TimeoutEnabledNoReset,
    /// Timed trigger is enabled, but reset on activation of Generic Interrupt 2
    TimeoutEnabledGen2IntReset,
}

/// Wake-up interrupt activity reference update mode
/// 
/// [WakeupIntRefMode::Manual] - The reference acceleration is set manually by the host MCU
/// 
/// [WakeupIntRefMode::OneTime] - A snapshot of the acceleration each time the device enters
///  low power mode is used as reference
/// 
/// [WakeupIntRefMode::EveryTime] - The reference acceleration is continuously updated in 
/// low power mode (25Hz) waking up on changes in acceleration samples larger than threshold
pub enum WakeupIntRefMode {
    /// Manually set reference acceleration
    Manual,
    /// Automatically snapshot acceleration upon entering low power mode
    OneTime,
    /// Continuously update reference acceleration in low power mode
    EveryTime
}

/// Orientation Changed reference update mode
/// 

pub enum OrientIntRefMode {
    /// Manually set reference acceleration
    Manual,
    /// Automatically snapshot acceleration from AccFilt2
    AccFilt2,
    /// Automatically snapshot acceleration from AccFilt2Lp (1Hz bandwidth filter)
    AccFilt2Lp
}

/// Number of samples to observe to determine baseline acceleration
pub enum ActChgObsPeriod {
    Samples32,
    Samples64,
    Samples128,
    Samples256,
    Samples512,
}

/// Tap Sensitivity
/// 
/// 0 = Highest, 7 = Lowest
pub enum TapSensitivity {
    SENS0,
    SENS1,
    SENS2,
    SENS3,
    SENS4,
    SENS5,
    SENS6,
    SENS7,
}

/// The minimum number of samples that must elapse between detected peaks for it to be considered part of a separate tap
pub enum MinTapDuration {
    Samples4,
    Samples8,
    Samples12,
    Samples16,
}

/// The maximum number of samples that can elapse between two detected peaks for it to be considered a double tap
pub enum DoubleTapDuration {
    Samples60,
    Samples80,
    Samples100,
    Samples120,
}

/// The maxiumum number of samples that can elapse between high and low peak of a tap for it to be considered a tap
pub enum MaxTapDuration {
    Samples6,
    Samples9,
    Samples12,
    Samples18,
}