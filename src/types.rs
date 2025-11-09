//! The structs and enums making up the driver API

use bitflags::bitflags;
/// Error types
#[derive(Debug)]
pub enum BMA400Error<InterfaceError> {
    /// I²C / SPI Error
    IOError(InterfaceError),
    /// Incorrect configuration
    ConfigBuildError(ConfigError),
    /// Invalid Chip ID read at initialization
    ChipIdReadFailed,
    /// Self-Test Failure
    SelfTestFailedError,
}

impl<InterfaceError> From<ConfigError> for BMA400Error<InterfaceError> {
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
    pub(crate) fn new(status_byte: u8) -> Self {
        Status { bits: status_byte }
    }
    /// Returns `true` if there is new data ready to be read
    pub fn drdy_stat(&self) -> bool {
        (self.bits & 0b1000_0000) != 0
    }
    /// Returns true if the command register is ready to receive a command
    pub fn cmd_rdy(&self) -> bool {
        (self.bits & 0b0001_0000) != 0
    }
    /// Returns the device's current [PowerMode]
    pub fn power_mode(&self) -> PowerMode {
        match (self.bits & 0b0000_0110) >> 1 {
            0 => PowerMode::Sleep,
            1 => PowerMode::LowPower,
            _ => PowerMode::Normal,
        }
    }
    /// Returns `true` if at least one of the interrupts is triggered
    pub fn int_active(&self) -> bool {
        (self.bits & 0b0000_0001) != 0
    }
}

/// The Step Interrupt Status
pub enum StepIntStatus {
    /// No Step Detected
    None,
    /// One step detected
    OneStepDetect,
    /// More than one step detected
    ManyStepDetect,
}

/// Interrupt statuses from the INT_STAT0 register:
///
/// - Data Ready Interrupt - [`drdy_stat()`](IntStatus0::drdy_stat)
/// - FIFO Watermark Interrupt (FIFO watermark surpassed) - [`fwm_stat()`](IntStatus0::fwm_stat)
/// - FIFO Buffer Full - [`ffull_stat()`](IntStatus0::ffull_stat)
/// - Interrupt Engine Overrun - [`ieng_overrun_stat()`](IntStatus0::ieng_overrun_stat)
/// - Generic Interrupt 2 - [`gen2_stat()`](IntStatus0::gen2_stat)
/// - Generic Interrupt 1 - [`gen1_stat()`](IntStatus0::gen1_stat)
/// - Orientation Changed - [`orientch_stat()`](IntStatus0::orientch_stat)
/// - Wakeup Activity Interrupt - [`wkup_stat()`](IntStatus0::wkup_stat)
pub struct IntStatus0 {
    bits: u8,
}

impl IntStatus0 {
    pub(crate) fn new(status_byte: u8) -> Self {
        IntStatus0 { bits: status_byte }
    }
    /// Returns `true` if the Data Ready Interrupt is triggered
    pub fn drdy_stat(&self) -> bool {
        (self.bits & 0b1000_0000) != 0
    }
    /// Returns `true` if the FIFO Watermark Interrupt is triggered
    pub fn fwm_stat(&self) -> bool {
        (self.bits & 0b0100_0000) != 0
    }
    /// Returns `true` if the FIFO Full Interrupt is triggered
    pub fn ffull_stat(&self) -> bool {
        (self.bits & 0b0010_0000) != 0
    }
    /// Returns `true` if the Interrupt Engine could not complete
    /// calculation of all enabled interrupts in time
    pub fn ieng_overrun_stat(&self) -> bool {
        (self.bits & 0b0001_0000) != 0
    }
    /// Returns `true` if Generic Interrupt 2 is triggered
    pub fn gen2_stat(&self) -> bool {
        (self.bits & 0b0000_1000) != 0
    }
    /// Returns `true` if Generic Interrupt 1 is triggered
    pub fn gen1_stat(&self) -> bool {
        (self.bits & 0b0000_0100) != 0
    }
    /// Returns `true` if the Orientation Change Interrupt is triggered
    pub fn orientch_stat(&self) -> bool {
        (self.bits & 0b0000_0010) != 0
    }
    /// Returns `true` if the Wake-Up Interrupt is triggered
    pub fn wkup_stat(&self) -> bool {
        (self.bits & 0b0000_0001) != 0
    }
}

/// Interrupt statuses from the INT_STAT1 register
///
/// - Interrupt Engine Overrun - [`ieng_overrun_stat()`](IntStatus0::ieng_overrun_stat)
/// - Double Tap Interrupt - [`d_tap_stat()`](IntStatus1::d_tap_stat)
/// - Single Tap Interrupt - [`s_tap_stat()`](IntStatus1::s_tap_stat)
/// - Step Interrupt - [`step_int_stat()`](IntStatus1::step_int_stat)
pub struct IntStatus1 {
    bits: u8,
}

impl IntStatus1 {
    pub(crate) fn new(status_byte: u8) -> Self {
        IntStatus1 { bits: status_byte }
    }
    /// Returns `true` if the Interrupt Engine could not complete
    /// calculation of all enabled interrupts in time
    pub fn ieng_overrun_stat(&self) -> bool {
        (self.bits & 0b0001_0000) != 0
    }
    /// Returns `true` if the Double Tap Interrupt is triggered
    pub fn d_tap_stat(&self) -> bool {
        (self.bits & 0b0000_1000) != 0
    }
    /// Returns `true` if the Single Tap Interrupt is triggered
    pub fn s_tap_stat(&self) -> bool {
        (self.bits & 0b0000_0100) != 0
    }
    /// Returns `true` if the Step Interrupt is triggered
    pub fn step_int_stat(&self) -> StepIntStatus {
        match self.bits & 0b0000_0011 {
            0x00 => StepIntStatus::None,
            0x01 => StepIntStatus::OneStepDetect,
            _ => StepIntStatus::ManyStepDetect,
        }
    }
}

/// Interrupt statuses from the INT_STAT2 register
///
/// - Interrupt Engine Overrun - [`ieng_overrun_stat()`](IntStatus0::ieng_overrun_stat)
/// - Activity Change Z - [`actch_z_stat()`](IntStatus2::actch_z_stat)
/// - Activity Change Y - [`actch_y_stat()`](IntStatus2::actch_y_stat)
/// - Activity Change X - [`actch_x_stat()`](IntStatus2::actch_x_stat)
pub struct IntStatus2 {
    bits: u8,
}

impl IntStatus2 {
    pub(crate) fn new(status_byte: u8) -> Self {
        IntStatus2 { bits: status_byte }
    }
    /// Returns `true` if the Interrupt Engine could not complete
    /// calculation of all enabled interrupts in time
    pub fn ieng_overrun_stat(&self) -> bool {
        (self.bits & 0b0001_0000) != 0
    }
    /// Returns `true` if the Activity Change Interrupt was triggered along the z-axis
    pub fn actch_z_stat(&self) -> bool {
        (self.bits & 0b0000_0100) != 0
    }
    /// Returns `true` if the Activity Change Interrupt was triggered along the y-axis
    pub fn actch_y_stat(&self) -> bool {
        (self.bits & 0b0000_0010) != 0
    }
    /// Returns `true` if the Activity Change Interrupt was triggered along the x-axis
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
    pub(crate) fn from_bytes_unscaled(bytes: &[u8]) -> Self {
        Self::new(
            Self::to_i16(bytes[0], bytes[1]),
            Self::to_i16(bytes[2], bytes[3]),
            Self::to_i16(bytes[4], bytes[5]),
        )
    }
    pub(crate) fn from_bytes_scaled(scale: Scale, bytes: &[u8]) -> Self {
        let shift = match scale {
            Scale::Range2G => 0,
            Scale::Range4G => 1,
            Scale::Range8G => 2,
            Scale::Range16G => 3,
        };
        Self::new(
            Self::to_i16(bytes[0], bytes[1]) << shift,
            Self::to_i16(bytes[2], bytes[3]) << shift,
            Self::to_i16(bytes[4], bytes[5]) << shift,
        )
    }
    fn to_i16(lsb: u8, msb: u8) -> i16 {
        let clear_rsvd_bits = msb & 0x0F;
        i16::from_le_bytes([
            lsb,
            if (clear_rsvd_bits >> 3) == 0u8 {
                clear_rsvd_bits
            } else {
                clear_rsvd_bits | 0xF0
            },
        ])
    }
}

/// The BMA400's Hardware Interrupt Pins, Int1 and Int2
pub enum InterruptPins {
    /// The interrupt is mapped to neither pin
    None,
    /// Map the interrupt to only the Int1 pin
    Int1,
    /// Map the interrupt to only the Int2 pin
    Int2,
    /// Map the interrupt to both Int1 and Int2 pins
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
/// See [p. 21 of the datasheet](https://www.bosch-sensortec.com/media/boschsensortec/downloads/datasheets/bst-bma400-ds000.pdf#page=21)
pub enum OversampleRate {
    /// Lowest Precision / Power Draw
    ///
    /// [`PowerMode::LowPower`] 0.85μA
    ///
    /// [`PowerMode::Normal`]  3.5μA
    OSR0,
    /// [`PowerMode::LowPower`] 0.93μA
    ///
    /// [`PowerMode::Normal`] 5.8μA
    OSR1,
    /// [`PowerMode::LowPower`] 1.1μA
    ///
    /// [`PowerMode::Normal`] 9.5μA
    OSR2,
    /// Highest Precision / Power Draw
    ///
    /// [`PowerMode::LowPower`] 1.35μA
    ///
    /// [`PowerMode::Normal`] 14.5μA
    OSR3,
}

/// The Power Mode of the device
///
/// [`PowerMode::Sleep`] lowest power - no data output, no FIFO Read or Write
///
/// [`PowerMode::LowPower`] Data is output at 25Hz to be used with Activity Detection and Auto Wakeup, no FIFO Write
///
/// [`PowerMode::Normal`] highest power - All functionality available
///
/// See [p.19 of the datasheet](https://www.bosch-sensortec.com/media/boschsensortec/downloads/datasheets/bst-bma400-ds000.pdf#page=19)
pub enum PowerMode {
    /// Sleep Mode: lowest power - no data output, no FIFO Read or Write
    Sleep,
    /// Low Power Mode: Data is output at 25Hz to be used with Activity Detection and Auto Wakeup, no FIFO Write
    LowPower,
    /// Normal Mode: highest power - All functionality available
    Normal,
}

/// Measurement Axis relative to the orientation of the sensor
///
/// See [p. 115 of the datasheet](https://www.bosch-sensortec.com/media/boschsensortec/downloads/datasheets/bst-bma400-ds000.pdf#page=115)
pub enum Axis {
    /// x axis
    X,
    /// y-axis
    Y,
    /// z-axis
    Z,
}

#[derive(Debug)]
/// Type of Activity Detected
///
/// - [`Activity::Still`]
/// - [`Activity::Walk`]
/// - [`Activity::Run`]
pub enum Activity {
    /// No Activity
    Still,
    /// Walking Detected
    Walk,
    /// Running Detected
    Run,
}

/// An individual frame read from the FIFO buffer.
///
/// The frame can be one of three [FrameType]s:
///
/// [`FrameType::Data`] - Contains an accelerometer reading for the axes enabled
/// at the time of measurement
///
/// [`FrameType::Control`] - This frame type is sent when there are changes to
/// either:
/// - the [DataSource] configured for the FIFO
/// - [Filter1Bandwidth] or
/// - [OutputDataRate], [OversampleRate] and/or [Scale]
///
/// [`FrameType::Time`] - Only sent if FIFO is configured with send_time_on_empty
/// enabled. This is the sensor clock reading as of reading past the last byte of the FIFO
#[derive(Debug, PartialEq)]
pub struct Frame<'a> {
    slice: &'a [u8],
}

impl<'a> Frame<'a> {
    /// Returns the [FrameType] of the [Frame]
    pub fn frame_type(&self) -> FrameType {
        Header::from_bits_truncate(self.slice[0]).frame_type()
    }
    /// If the [Frame] has x-axis data returns a result containing the x-axis measurement, None otherwise
    pub fn x(&self) -> Option<i16> {
        let header = Header::from_bits_truncate(self.slice[0]);
        if !matches!(header.frame_type(), FrameType::Data) || !header.has_x_data() {
            return None;
        }
        Some(self.data_at_offset(0, header.resolution_is_12bit()))
    }
    /// If the [Frame] has y-axis data returns a result containing the y-axis measurement, None otherwise
    pub fn y(&self) -> Option<i16> {
        let header = Header::from_bits_truncate(self.slice[0]);
        if !matches!(header.frame_type(), FrameType::Data) || !header.has_y_data() {
            return None;
        }
        let offset = if header.has_x_data() { 1 } else { 0 };
        Some(self.data_at_offset(offset, header.resolution_is_12bit()))
    }
    /// If the [Frame] has z-axis data returns a result containing the z-axis measurement, None otherwise
    pub fn z(&self) -> Option<i16> {
        let header = Header::from_bits_truncate(self.slice[0]);
        if !matches!(header.frame_type(), FrameType::Data) || !header.has_z_data() {
            return None;
        }
        let offset =
            if header.has_x_data() { 1 } else { 0 } + if header.has_y_data() { 1 } else { 0 };
        Some(self.data_at_offset(offset, header.resolution_is_12bit()))
    }
    /// If the [FrameType] is [`FrameType::Time`], returns a result containing the sensor time, None otherwise
    pub fn time(&self) -> Option<u32> {
        if !matches!(self.frame_type(), FrameType::Time) {
            return None;
        }
        Some(u32::from_le_bytes([
            self.slice[1],
            self.slice[2],
            self.slice[3],
            0,
        ]))
    }
    /// If the [FrameType] is [`FrameType::Control`], returns a result containing whether a FIFO data source change was indicated
    pub fn fifo_src_chg(&self) -> Option<bool> {
        if let FrameType::Control = self.frame_type() {
            Some(self.slice[1] & 0b0010 != 0)
        } else {
            None
        }
    }
    /// If the [FrameType] is [`FrameType::Control`], returns a result containing whether a [Filter1Bandwidth] change was indicated
    pub fn filt1_bw_chg(&self) -> Option<bool> {
        if let FrameType::Control = self.frame_type() {
            Some(self.slice[1] & 0b0100 != 0)
        } else {
            None
        }
    }
    /// If the [FrameType] is [`FrameType::Control`], returns a result containing whether a [OutputDataRate],
    /// [OversampleRate] and/or [Scale] change was indicated
    pub fn acc1_chg(&self) -> Option<bool> {
        if let FrameType::Control = self.frame_type() {
            Some(self.slice[1] & 0b1000 != 0)
        } else {
            None
        }
    }
    fn data_at_offset(&self, offset: usize, resolution_is_12bit: bool) -> i16 {
        let (lsb, msb);
        if resolution_is_12bit {
            lsb = (self.slice[offset * 2 + 1] & 0xF) | (self.slice[offset * 2 + 2] << 4);
            msb = self.slice[offset * 2 + 2] >> 4;
        } else {
            lsb = self.slice[offset + 1] << 4;
            msb = self.slice[offset + 1] >> 4;
        }
        i16::from_le_bytes([lsb, if (msb >> 3) == 0u8 { msb } else { msb | 0xF0 }])
    }
}

/// The type of the FIFO Frame
pub enum FrameType {
    /// Acceleration Data
    Data,
    /// Sensor Time
    Time,
    /// Configuration Change
    Control,
}

/// An iterator over the buffer provided to [`read_fifo_frames()`](crate::BMA400::read_fifo_frames)
#[derive(Debug)]
pub struct FifoFrames<'a> {
    index: usize,
    bytes: &'a [u8],
}

impl<'a> FifoFrames<'a> {
    pub(crate) fn new(bytes: &'_ [u8]) -> FifoFrames<'_> {
        FifoFrames { index: 0, bytes }
    }
}

impl<'a> Iterator for FifoFrames<'a> {
    type Item = Frame<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.bytes.len() {
            return None;
        }
        let header_idx = self.index;
        let header = Header::from_bits_truncate(self.bytes[header_idx]);
        if matches!(header.frame_type(), FrameType::Data) && !header.has_data() {
            self.index += 2;
            return None;
        }
        self.index += header.num_payload_bytes() + 1;
        // Incomplete read
        if self.index > self.bytes.len() {
            return None;
        }
        Some(Frame {
            slice: &self.bytes[header_idx..self.index],
        })
    }
}

bitflags! {
    struct Header: u8 {
        const FH_MODE1  = 0b1000_0000;
        const FH_MODE0  = 0b0100_0000;
        const FH_PARAM4 = 0b0010_0000;
        const FH_PARAM3 = 0b0001_0000;
        const FH_PARAM2 = 0b0000_1000;
        const FH_PARAM1 = 0b0000_0100;
        const FH_PARAM0 = 0b0000_0010;

        const TIME = Self::FH_MODE1.bits | Self::FH_PARAM4.bits;
        const RESOLUTION = Self::FH_PARAM3.bits;
        const AXES = Self::FH_PARAM2.bits | Self::FH_PARAM1.bits | Self::FH_PARAM0.bits;
    }
}

impl Header {
    pub const fn frame_type(&self) -> FrameType {
        if self.contains(Self::TIME) {
            FrameType::Time
        } else if self.intersects(Self::FH_MODE0) {
            FrameType::Control
        } else {
            FrameType::Data
        }
    }
    pub const fn resolution_is_12bit(&self) -> bool {
        match self.frame_type() {
            FrameType::Data => self.intersects(Self::RESOLUTION),
            _ => false,
        }
    }
    pub const fn has_data(&self) -> bool {
        match self.frame_type() {
            FrameType::Data => self.intersects(Self::AXES),
            _ => false,
        }
    }
    pub const fn num_payload_bytes(&self) -> usize {
        match self.frame_type() {
            FrameType::Time => 3,
            FrameType::Data => {
                if !self.has_data() {
                    return 1;
                }
                let mut n = self.intersection(Self::AXES).bits();
                let mut num_axes = 0;
                while n != 0 {
                    n &= n - 1;
                    num_axes += 1;
                }
                if self.resolution_is_12bit() {
                    num_axes * 2
                } else {
                    num_axes
                }
            }
            FrameType::Control => 1,
        }
    }
    pub const fn has_x_data(&self) -> bool {
        match self.frame_type() {
            FrameType::Data => self.intersects(Self::FH_PARAM0),
            _ => false,
        }
    }
    pub const fn has_y_data(&self) -> bool {
        match self.frame_type() {
            FrameType::Data => self.intersects(Self::FH_PARAM1),
            _ => false,
        }
    }
    pub const fn has_z_data(&self) -> bool {
        match self.frame_type() {
            FrameType::Data => self.intersects(Self::FH_PARAM2),
            _ => false,
        }
    }
}

/// Automatically enter low power mode after a defined timeout
///
/// Non-timed triggers are still supported if timeout is disabled
///
/// See [p.25 of the datasheet](https://www.bosch-sensortec.com/media/boschsensortec/downloads/datasheets/bst-bma400-ds000.pdf#page=25)
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
/// [WakeupIntRefMode::Manual] - The reference acceleration must be set manually
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
    EveryTime,
}

/// Orientation Changed reference update mode
///
/// [OrientIntRefMode::Manual] - The reference acceleration must be set manually
///
/// [OrientIntRefMode::AccFilt2] - A snapshot of the acceleration from AccFilt2
/// is written when stable orientation is detected
///
/// [OrientIntRefMode::AccFilt2Lp] - A snapshot of the acceleration from AccFilt2Lp
///  (1Hz bandwidth filter) is written when stable orientation is detected
pub enum OrientIntRefMode {
    /// Manually set reference acceleration
    Manual,
    /// Automatically snapshot acceleration from AccFilt2
    AccFilt2,
    /// Automatically snapshot acceleration from AccFilt2Lp (1Hz bandwidth filter)
    AccFilt2Lp,
}

/// Number of samples to observe to determine baseline acceleration
pub enum ActChgObsPeriod {
    /// 32 Samples
    Samples32,
    /// 64 Samples
    Samples64,
    /// 128 Samples
    Samples128,
    /// 256 Samples
    Samples256,
    /// 512 Samples
    Samples512,
}

/// Tap Sensitivity
///
/// 0 = Highest, 7 = Lowest
///
/// See [p. 45 of the datasheet](https://www.bosch-sensortec.com/media/boschsensortec/downloads/datasheets/bst-bma400-ds000.pdf#page=45)
pub enum TapSensitivity {
    /// Setting 0 - Highest
    SENS0,
    /// Setting 1
    SENS1,
    /// Setting 2
    SENS2,
    /// Setting 3
    SENS3,
    /// Setting 4
    SENS4,
    /// Setting 5
    SENS5,
    /// Setting 6
    SENS6,
    /// Setting 7 - Lowest
    SENS7,
}

/// The minimum number of samples that must elapse between detected peaks for it to be considered
/// part of a separate tap
pub enum MinTapDuration {
    /// 4 Samples
    Samples4,
    /// 8 Samples
    Samples8,
    /// 12 Samples
    Samples12,
    /// 16 Samples
    Samples16,
}

/// The maximum number of samples that can elapse between two detected peaks for it to be considered
/// a double tap
pub enum DoubleTapDuration {
    /// 60 Samples
    Samples60,
    /// 80 Samples
    Samples80,
    /// 100 Samples
    Samples100,
    /// 120 Samples
    Samples120,
}

/// The maxiumum number of samples that can elapse between high and low peak of a tap for it to be
/// considered a tap
pub enum MaxTapDuration {
    /// 6 Samples
    Samples6,
    /// 9 Samples
    Samples9,
    /// 12 Samples
    Samples12,
    /// 18 Samples
    Samples18,
}

/// Generic interrupt activity detection reference acceleration update mode
pub enum GenIntRefMode {
    /// Reference is not updated automatically and must be set by using `with_ref_accel()`
    Manual,
    /// A reference acceleration snapshot is taken from the selected data source once upon
    /// triggering the interrupt or entering normal mode
    OneTime,
    /// A reference acceleration snapshot is taken from the selected data source each time the
    /// interrupt condition is evaluated
    EveryTimeFromSrc,
    /// A reference acceleration snapshot is taken from AccFilt2Lp (1Hz) each time the interrupt
    /// condition is evaluated
    EveryTimeFromLp,
}

/// Hysteresis configuration options for the Generic interrupt activity comparision
pub enum Hysteresis {
    /// No hysteresis
    None,
    /// 24 milli-g hysteresis
    Hyst24mg,
    /// 48 milli-g hysteresis
    Hyst48mg,
    /// 96 milli-g hysteresis
    Hyst96mg,
}

/// Select whether the interrupt triggers on detecting acceleration
/// either outside or inside the \[`ref_accel`-`threshold`,`ref_accel`+`threshold`\] window
pub enum GenIntCriterionMode {
    /// Interrupt triggers on acceleration inside reference +/- threshold (Inactivity Detection)
    Inactivity,
    /// Interrupt triggers on acceleration outside reference +/- threshold (Activity Detection)
    Activity,
}

/// Select whether the interrupt triggers on any single access satisfying its criterion
/// or all enabled axes must satisfy their criteria
pub enum GenIntLogicMode {
    /// Interrupt triggers if the acceleration for _any_ axis satisfies its criterion
    Or,
    /// Interrupt triggers only if the acceleration for _all_ axes satisfies their criteria
    And,
}
