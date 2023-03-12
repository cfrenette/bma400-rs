use crate::{
    types::{
        ActChgObsPeriod,
        AutoLPTimeoutTrigger,
        Axis,
        DataSource,
        DoubleTapDuration,
        Filter1Bandwidth,
        MaxTapDuration,
        MinTapDuration,
        OrientIntRefMode,
        OutputDataRate,
        OversampleRate,
        PinOutputConfig,
        PinOutputLevel,
        PowerMode,
        Scale,
        TapSensitivity,
        WakeupIntRefMode,
    },
    GenIntCriterionMode,
    GenIntLogicMode,
    GenIntRefMode,
    Hysteresis,
};

pub trait ReadReg {
    const ADDR: u8;
    fn addr(&self) -> u8 {
        Self::ADDR
    }
}

pub trait ConfigReg: ReadReg
where
    Self: Sized,
{
    fn to_byte(&self) -> u8;
}

/// RegisterName: Address
macro_rules! r_register {
    ($name:ident: $address:literal) => {
        pub struct $name;
        impl ReadReg for $name {
            const ADDR: u8 = $address;
        }
    };
}

macro_rules! cfg_register {
    ($name:ident: $address:literal = $default:literal {
        $(const $field_name:ident = $bitmask:expr;)+
    }) => {
        bitflags::bitflags! {
            pub struct $name: u8 {
                $(const $field_name = $bitmask;)+
            }
        }
        impl ReadReg for $name {
            const ADDR: u8 = $address;
        }
        impl ConfigReg for $name {
            fn to_byte(&self) -> u8 {
                self.bits
            }
        }
        impl Default for $name {
            fn default() -> Self {
                Self::from_bits_truncate($default)
            }
        }
    }
}

r_register!(ChipId: 0x00);
r_register!(ErrReg: 0x02);
r_register!(StatusReg: 0x03);
r_register!(AccXLSB: 0x04);
r_register!(AccXMSB: 0x05);
r_register!(AccYLSB: 0x06);
r_register!(AccYMSB: 0x07);
r_register!(AccZLSB: 0x08);
r_register!(AccZMSB: 0x09);
r_register!(SensorTime0: 0x0A);
r_register!(SensorTime1: 0x0B);
r_register!(SensorTime2: 0x0C);
r_register!(Event: 0x0D);
r_register!(InterruptStatus0: 0x0E);
r_register!(InterruptStatus1: 0x0F);
r_register!(InterruptStatus2: 0x10);
r_register!(TempData: 0x11);
r_register!(FifoLength0: 0x12);
r_register!(FifoLength1: 0x13);
r_register!(FifoData: 0x14);
r_register!(StepCount0: 0x15);
r_register!(StepCount1: 0x16);
r_register!(StepCount2: 0x17);
r_register!(StepStatus: 0x18);

cfg_register! {
    AccConfig0: 0x19 = 0x00 {
        const FILT_BW = 0b1000_0000;
        const OSR_LP1 = 0b0100_0000;
        const OSR_LP0 = 0b0010_0000;
        const PW_CFG1 = 0b0000_0010;
        const PW_CFG0 = 0b0000_0001;

        const OSR_LP = Self::OSR_LP1.bits | Self::OSR_LP0.bits;
        const PWR_MODE = Self::PW_CFG1.bits | Self::PW_CFG0.bits;
    }
}

impl AccConfig0 {
    pub const fn with_filt1_bw(self, bandwidth: Filter1Bandwidth) -> Self {
        match bandwidth {
            Filter1Bandwidth::High => self.difference(Self::FILT_BW),
            Filter1Bandwidth::Low => self.union(Self::FILT_BW),
        }
    }
    pub const fn with_osr_lp(self, osr: OversampleRate) -> Self {
        match osr {
            OversampleRate::OSR0 => self.difference(Self::OSR_LP),
            OversampleRate::OSR1 => self.difference(Self::OSR_LP).union(Self::OSR_LP0),
            OversampleRate::OSR2 => self.difference(Self::OSR_LP).union(Self::OSR_LP1),
            OversampleRate::OSR3 => self.union(Self::OSR_LP),
        }
    }
    pub const fn with_power_mode(self, power_mode: PowerMode) -> Self {
        match power_mode {
            PowerMode::Sleep => self.difference(Self::PWR_MODE),
            PowerMode::LowPower => self.difference(Self::PWR_MODE).union(Self::PW_CFG0),
            PowerMode::Normal => self.difference(Self::PWR_MODE).union(Self::PW_CFG1),
        }
    }
}

cfg_register! {
    AccConfig1: 0x1A = 0x49 {
        const ACC_RANGE1 = 0b1000_0000;
        const ACC_RANGE0 = 0b0100_0000;
        const OSR1       = 0b0010_0000;
        const OSR0       = 0b0001_0000;
        const ACC_ODR3   = 0b0000_1000;
        const ACC_ODR2   = 0b0000_0100;
        const ACC_ODR1   = 0b0000_0010;
        const ACC_ODR0   = 0b0000_0001;

        const ACC_RANGE = Self::ACC_RANGE1.bits | Self::ACC_RANGE0.bits;
        const OSR = Self::OSR1.bits | Self::OSR0.bits;
        const ACC_ODR = Self::ACC_ODR3.bits | Self::ACC_ODR2.bits | Self::ACC_ODR1.bits | Self::ACC_ODR0.bits;
    }
}

impl AccConfig1 {
    pub const fn scale(&self) -> Scale {
        match (self.intersection(Self::ACC_RANGE)).bits() >> 6 {
            0x00 => Scale::Range2G,
            0x01 => Scale::Range4G,
            0x02 => Scale::Range8G,
            _ => Scale::Range16G,
        }
    }
    pub const fn with_scale(self, scale: Scale) -> Self {
        match scale {
            Scale::Range2G => self.difference(Self::ACC_RANGE),
            Scale::Range4G => self.difference(Self::ACC_RANGE).union(Self::ACC_RANGE0),
            Scale::Range8G => self.difference(Self::ACC_RANGE).union(Self::ACC_RANGE1),
            Scale::Range16G => self.union(Self::ACC_RANGE),
        }
    }
    pub const fn with_osr(self, osr: OversampleRate) -> Self {
        match osr {
            OversampleRate::OSR0 => self.difference(Self::OSR),
            OversampleRate::OSR1 => self.difference(Self::OSR).union(Self::OSR0),
            OversampleRate::OSR2 => self.difference(Self::OSR).union(Self::OSR1),
            OversampleRate::OSR3 => self.union(Self::OSR),
        }
    }
    pub const fn odr(&self) -> OutputDataRate {
        match self.intersection(Self::ACC_ODR).bits() {
            0x05 => OutputDataRate::Hz12_5,
            0x06 => OutputDataRate::Hz25,
            0x07 => OutputDataRate::Hz50,
            0x08 => OutputDataRate::Hz100,
            0x09 => OutputDataRate::Hz200,
            0x0A => OutputDataRate::Hz400,
            _ => OutputDataRate::Hz800,
        }
    }
    pub const fn with_odr(self, odr: OutputDataRate) -> Self {
        self.difference(Self::ACC_ODR).union(match odr {
            OutputDataRate::Hz12_5 => Self::ACC_ODR2.union(Self::ACC_ODR0),
            OutputDataRate::Hz25 => Self::ACC_ODR2.union(Self::ACC_ODR1),
            OutputDataRate::Hz50 => Self::ACC_ODR2.union(Self::ACC_ODR1).union(Self::ACC_ODR0),
            OutputDataRate::Hz100 => Self::ACC_ODR3,
            OutputDataRate::Hz200 => Self::ACC_ODR3.union(Self::ACC_ODR0),
            OutputDataRate::Hz400 => Self::ACC_ODR3.union(Self::ACC_ODR1),
            OutputDataRate::Hz800 => Self::ACC_ODR3.union(Self::ACC_ODR1).union(Self::ACC_ODR0),
        })
    }
}

cfg_register! {
    AccConfig2: 0x1B = 0x00 {
        const DTA_SRC1 = 0b0000_1000;
        const DTA_SRC0 = 0b0000_0100;

        const DTA_SRC = Self::DTA_SRC1.bits | Self::DTA_SRC0.bits;
    }
}

impl AccConfig2 {
    pub const fn with_dta_reg_src(self, data_src: DataSource) -> Self {
        match data_src {
            DataSource::AccFilt1 => self.difference(Self::DTA_SRC),
            DataSource::AccFilt2 => self.difference(Self::DTA_SRC).union(Self::DTA_SRC0),
            DataSource::AccFilt2Lp => self.difference(Self::DTA_SRC).union(Self::DTA_SRC1),
        }
    }
}

cfg_register! {
    IntConfig0: 0x1F = 0x00 {
        const DRDY_INT_EN     = 0b1000_0000;
        const FWM_INT_EN      = 0b0100_0000;
        const FFULL_INT_EN    = 0b0010_0000;
        const GEN2_INT_EN     = 0b0000_1000;
        const GEN1_INT_EN     = 0b0000_0100;
        const ORIENTCH_INT_EN = 0b0000_0010;
    }
}

impl IntConfig0 {
    pub const fn dta_rdy_int(&self) -> bool {
        self.intersects(Self::DRDY_INT_EN)
    }
    pub const fn with_dta_rdy_int(self, enabled: bool) -> Self {
        if enabled {
            self.union(Self::DRDY_INT_EN)
        } else {
            self.difference(Self::DRDY_INT_EN)
        }
    }
    pub const fn fwm_int(&self) -> bool {
        self.intersects(Self::FWM_INT_EN)
    }
    pub const fn with_fwm_int(self, enabled: bool) -> Self {
        if enabled {
            self.union(Self::FWM_INT_EN)
        } else {
            self.difference(Self::FWM_INT_EN)
        }
    }
    pub const fn ffull_int(&self) -> bool {
        self.intersects(Self::FFULL_INT_EN)
    }
    pub const fn with_ffull_int(self, enabled: bool) -> Self {
        if enabled {
            self.union(Self::FFULL_INT_EN)
        } else {
            self.difference(Self::FFULL_INT_EN)
        }
    }
    pub const fn gen2_int(&self) -> bool {
        self.intersects(Self::GEN2_INT_EN)
    }
    pub const fn with_gen2_int(self, enabled: bool) -> Self {
        if enabled {
            self.union(Self::GEN2_INT_EN)
        } else {
            self.difference(Self::GEN2_INT_EN)
        }
    }
    pub const fn gen1_int(&self) -> bool {
        self.intersects(Self::GEN1_INT_EN)
    }
    pub const fn with_gen1_int(self, enabled: bool) -> Self {
        if enabled {
            self.union(Self::GEN1_INT_EN)
        } else {
            self.difference(Self::GEN1_INT_EN)
        }
    }
    pub const fn orientch_int(&self) -> bool {
        self.intersects(Self::ORIENTCH_INT_EN)
    }
    pub const fn with_orientch_int(self, enabled: bool) -> Self {
        if enabled {
            self.union(Self::ORIENTCH_INT_EN)
        } else {
            self.difference(Self::ORIENTCH_INT_EN)
        }
    }
}

cfg_register! {
    IntConfig1: 0x20 = 0x00 {
        const LATCH_INT = 0b1000_0000;
        const ACTCH_INT = 0b0001_0000;
        const D_TAP_INT = 0b0000_1000;
        const S_TAP_INT = 0b0000_0100;
        const STEP_INT  = 0b0000_0001;
    }
}

impl IntConfig1 {
    pub const fn with_latch_int(self, enabled: bool) -> Self {
        if enabled {
            self.union(Self::LATCH_INT)
        } else {
            self.difference(Self::LATCH_INT)
        }
    }
    pub const fn actch_int(&self) -> bool {
        self.intersects(Self::ACTCH_INT)
    }
    pub const fn with_actch_int(self, enabled: bool) -> Self {
        if enabled {
            self.union(Self::ACTCH_INT)
        } else {
            self.difference(Self::ACTCH_INT)
        }
    }
    pub const fn s_tap_int(&self) -> bool {
        self.intersects(Self::S_TAP_INT)
    }
    pub const fn with_s_tap_int(self, enabled: bool) -> Self {
        if enabled {
            self.union(Self::S_TAP_INT)
        } else {
            self.difference(Self::S_TAP_INT)
        }
    }
    pub const fn d_tap_int(&self) -> bool {
        self.intersects(Self::D_TAP_INT)
    }
    pub const fn with_d_tap_int(self, enabled: bool) -> Self {
        if enabled {
            self.union(Self::D_TAP_INT)
        } else {
            self.difference(Self::D_TAP_INT)
        }
    }
    pub const fn step_int(&self) -> bool {
        self.intersects(Self::STEP_INT)
    }
    pub const fn with_step_int(self, enabled: bool) -> Self {
        if enabled {
            self.union(Self::STEP_INT)
        } else {
            self.difference(Self::STEP_INT)
        }
    }
}

cfg_register! {
    Int1Map: 0x21 = 0x00 {
        const DRDY      = 0b1000_0000;
        const FWM       = 0b0100_0000;
        const FFULL     = 0b0010_0000;
        const OVRRN     = 0b0001_0000;
        const GEN2      = 0b0000_1000;
        const GEN1      = 0b0000_0100;
        const ORIENTCH  = 0b0000_0010;
        const WKUP      = 0b0000_0001;
    }
}

impl Int1Map {
    pub const fn drdy_int(&self) -> bool {
        self.intersects(Self::DRDY)
    }
    pub const fn with_drdy(self, mapped: bool) -> Self {
        if mapped {
            self.union(Self::DRDY)
        } else {
            self.difference(Self::DRDY)
        }
    }
    pub const fn fwm_int(&self) -> bool {
        self.intersects(Self::FWM)
    }
    pub const fn with_fwm(self, mapped: bool) -> Self {
        if mapped {
            self.union(Self::FWM)
        } else {
            self.difference(Self::FWM)
        }
    }
    pub const fn ffull_int(&self) -> bool {
        self.intersects(Self::FFULL)
    }
    pub const fn with_ffull(self, mapped: bool) -> Self {
        if mapped {
            self.union(Self::FFULL)
        } else {
            self.difference(Self::FFULL)
        }
    }
    pub const fn with_ovrrn(self, mapped: bool) -> Self {
        if mapped {
            self.union(Self::OVRRN)
        } else {
            self.difference(Self::OVRRN)
        }
    }
    pub const fn gen2_int(&self) -> bool {
        self.intersects(Self::GEN2)
    }
    pub const fn with_gen2(self, mapped: bool) -> Self {
        if mapped {
            self.union(Self::GEN2)
        } else {
            self.difference(Self::GEN2)
        }
    }
    pub const fn gen1_int(&self) -> bool {
        self.intersects(Self::GEN1)
    }
    pub const fn with_gen1(self, mapped: bool) -> Self {
        if mapped {
            self.union(Self::GEN1)
        } else {
            self.difference(Self::GEN1)
        }
    }
    pub const fn orientch_int(&self) -> bool {
        self.intersects(Self::ORIENTCH)
    }
    pub const fn with_orientch(self, mapped: bool) -> Self {
        if mapped {
            self.union(Self::ORIENTCH)
        } else {
            self.difference(Self::ORIENTCH)
        }
    }
    pub const fn wkup_int(&self) -> bool {
        self.intersects(Self::WKUP)
    }
    pub const fn with_wkup(self, mapped: bool) -> Self {
        if mapped {
            self.union(Self::WKUP)
        } else {
            self.difference(Self::WKUP)
        }
    }
}

cfg_register! {
    Int2Map: 0x22 = 0x00 {
        const DRDY      = 0b1000_0000;
        const FWM       = 0b0100_0000;
        const FFULL     = 0b0010_0000;
        const OVRRN     = 0b0001_0000;
        const GEN2      = 0b0000_1000;
        const GEN1      = 0b0000_0100;
        const ORIENTCH  = 0b0000_0010;
        const WKUP      = 0b0000_0001;
    }
}

impl Int2Map {
    pub const fn drdy_int(&self) -> bool {
        self.intersects(Self::DRDY)
    }
    pub const fn with_drdy(self, mapped: bool) -> Self {
        if mapped {
            self.union(Self::DRDY)
        } else {
            self.difference(Self::DRDY)
        }
    }
    pub const fn fwm_int(&self) -> bool {
        self.intersects(Self::FWM)
    }
    pub const fn with_fwm(self, mapped: bool) -> Self {
        if mapped {
            self.union(Self::FWM)
        } else {
            self.difference(Self::FWM)
        }
    }
    pub const fn ffull_int(&self) -> bool {
        self.intersects(Self::FFULL)
    }
    pub const fn with_ffull(self, mapped: bool) -> Self {
        if mapped {
            self.union(Self::FFULL)
        } else {
            self.difference(Self::FFULL)
        }
    }
    pub const fn with_ovrrn(self, mapped: bool) -> Self {
        if mapped {
            self.union(Self::OVRRN)
        } else {
            self.difference(Self::OVRRN)
        }
    }
    pub const fn gen2_int(&self) -> bool {
        self.intersects(Self::GEN2)
    }
    pub const fn with_gen2(self, mapped: bool) -> Self {
        if mapped {
            self.union(Self::GEN2)
        } else {
            self.difference(Self::GEN2)
        }
    }
    pub const fn gen1_int(&self) -> bool {
        self.intersects(Self::GEN1)
    }
    pub const fn with_gen1(self, mapped: bool) -> Self {
        if mapped {
            self.union(Self::GEN1)
        } else {
            self.difference(Self::GEN1)
        }
    }
    pub const fn orientch_int(&self) -> bool {
        self.intersects(Self::ORIENTCH)
    }
    pub const fn with_orientch(self, mapped: bool) -> Self {
        if mapped {
            self.union(Self::ORIENTCH)
        } else {
            self.difference(Self::ORIENTCH)
        }
    }
    pub const fn wkup_int(&self) -> bool {
        self.intersects(Self::WKUP)
    }
    pub const fn with_wkup(self, mapped: bool) -> Self {
        if mapped {
            self.union(Self::WKUP)
        } else {
            self.difference(Self::WKUP)
        }
    }
}

cfg_register! {
    Int12Map: 0x23 = 0x00 {
        const ACTCH2 = 0b1000_0000;
        const TAP2   = 0b0100_0000;
        const STEP2  = 0b0001_0000;
        const ACTCH1 = 0b0000_1000;
        const TAP1   = 0b0000_0100;
        const STEP1  = 0b0000_0001;
    }
}

impl Int12Map {
    pub const fn actch_int2(&self) -> bool {
        self.intersects(Self::ACTCH2)
    }
    pub const fn with_actch2(self, mapped: bool) -> Self {
        if mapped {
            self.union(Self::ACTCH2)
        } else {
            self.difference(Self::ACTCH2)
        }
    }
    pub const fn actch_int1(&self) -> bool {
        self.intersects(Self::ACTCH1)
    }
    pub const fn with_actch1(self, mapped: bool) -> Self {
        if mapped {
            self.union(Self::ACTCH1)
        } else {
            self.difference(Self::ACTCH1)
        }
    }
    pub const fn tap_int2(&self) -> bool {
        self.intersects(Self::TAP2)
    }
    pub const fn with_tap2(self, mapped: bool) -> Self {
        if mapped {
            self.union(Self::TAP2)
        } else {
            self.difference(Self::TAP2)
        }
    }
    pub const fn tap_int1(&self) -> bool {
        self.intersects(Self::TAP1)
    }
    pub const fn with_tap1(self, mapped: bool) -> Self {
        if mapped {
            self.union(Self::TAP1)
        } else {
            self.difference(Self::TAP1)
        }
    }
    pub const fn step_int2(&self) -> bool {
        self.intersects(Self::STEP2)
    }
    pub const fn with_step2(self, mapped: bool) -> Self {
        if mapped {
            self.union(Self::STEP2)
        } else {
            self.difference(Self::STEP2)
        }
    }
    pub const fn step_int1(&self) -> bool {
        self.intersects(Self::STEP1)
    }
    pub const fn with_step1(self, mapped: bool) -> Self {
        if mapped {
            self.union(Self::STEP1)
        } else {
            self.difference(Self::STEP1)
        }
    }
}

cfg_register! {
    Int12IOCtrl: 0x24 = 0x22 {
        const INT2_OD = 0b0100_0000;
        const INT2_LV = 0b0010_0000;
        const INT1_OD = 0b0000_0100;
        const INT1_LV = 0b0000_0010;
    }
}

impl Int12IOCtrl {
    pub const fn with_int1_cfg(self, config: PinOutputConfig) -> Self {
        match config {
            PinOutputConfig::PushPull(level) => match level {
                PinOutputLevel::ActiveLow => self.difference(Self::INT1_LV.union(Self::INT1_OD)),
                PinOutputLevel::ActiveHigh => self.difference(Self::INT1_OD).union(Self::INT1_LV),
            },
            PinOutputConfig::OpenDrain(level) => match level {
                PinOutputLevel::ActiveLow => self.difference(Self::INT1_LV).union(Self::INT1_OD),
                PinOutputLevel::ActiveHigh => self.union(Self::INT1_LV.union(Self::INT1_OD)),
            },
        }
    }
    pub const fn with_int2_cfg(self, config: PinOutputConfig) -> Self {
        match config {
            PinOutputConfig::PushPull(level) => match level {
                PinOutputLevel::ActiveLow => self.difference(Self::INT2_LV.union(Self::INT2_OD)),
                PinOutputLevel::ActiveHigh => self.difference(Self::INT2_OD).union(Self::INT2_LV),
            },
            PinOutputConfig::OpenDrain(level) => match level {
                PinOutputLevel::ActiveLow => self.difference(Self::INT2_LV).union(Self::INT2_OD),
                PinOutputLevel::ActiveHigh => self.union(Self::INT2_LV.union(Self::INT2_OD)),
            },
        }
    }
}

cfg_register! {
    FifoConfig0: 0x26 = 0x00 {
        const FIFO_Z    = 0b1000_0000;
        const FIFO_Y    = 0b0100_0000;
        const FIFO_X    = 0b0010_0000;
        const FIFO_8    = 0b0001_0000;
        const FIFO_SRC  = 0b0000_1000;
        const FIFO_TIME = 0b0000_0100;
        const FIFO_STOP = 0b0000_0010;
        const PWR_FLUSH = 0b0000_0001;
    }
}

impl FifoConfig0 {
    pub const fn with_fifo_z(self, enabled: bool) -> Self {
        if enabled {
            self.union(Self::FIFO_Z)
        } else {
            self.difference(Self::FIFO_Z)
        }
    }
    pub const fn with_fifo_y(self, enabled: bool) -> Self {
        if enabled {
            self.union(Self::FIFO_Y)
        } else {
            self.difference(Self::FIFO_Y)
        }
    }
    pub const fn with_fifo_x(self, enabled: bool) -> Self {
        if enabled {
            self.union(Self::FIFO_X)
        } else {
            self.difference(Self::FIFO_X)
        }
    }
    pub const fn with_fifo_8bit(self, enabled: bool) -> Self {
        if enabled {
            self.union(Self::FIFO_8)
        } else {
            self.difference(Self::FIFO_8)
        }
    }
    pub const fn with_fifo_src(self, data_source: DataSource) -> Self {
        match data_source {
            DataSource::AccFilt1 => self.difference(Self::FIFO_SRC),
            DataSource::AccFilt2 => self.union(Self::FIFO_SRC),
            _ => unreachable!(), // Handled in the public API
        }
    }
    pub const fn with_send_time_on_empty(self, enabled: bool) -> Self {
        if enabled {
            self.union(Self::FIFO_TIME)
        } else {
            self.difference(Self::FIFO_TIME)
        }
    }
    pub const fn with_stop_on_full(self, enabled: bool) -> Self {
        if enabled {
            self.union(Self::FIFO_STOP)
        } else {
            self.difference(Self::FIFO_STOP)
        }
    }
    pub const fn with_flush_on_pwr_mode_change(self, enabled: bool) -> Self {
        if enabled {
            self.union(Self::PWR_FLUSH)
        } else {
            self.difference(Self::PWR_FLUSH)
        }
    }
}

cfg_register! {
    FifoConfig1: 0x27 = 0x00 {
        const FIFO_THRESH_LSB7 = 0b1000_0000;
        const FIFO_THRESH_LSB6 = 0b0100_0000;
        const FIFO_THRESH_LSB5 = 0b0010_0000;
        const FIFO_THRESH_LSB4 = 0b0001_0000;
        const FIFO_THRESH_LSB3 = 0b0000_1000;
        const FIFO_THRESH_LSB2 = 0b0000_0100;
        const FIFO_THRESH_LSB1 = 0b0000_0010;
        const FIFO_THRESH_LSB0 = 0b0000_0001;
    }
}

impl FifoConfig1 {
    pub const fn with_fifo_wtrmk_threshold(self, threshold: u8) -> Self {
        Self::from_bits_truncate(threshold)
    }
}

cfg_register! {
    FifoConfig2: 0x28 = 0x00 {
        const FIFO_THRESH_MSB2 = 0b0000_0100;
        const FIFO_THRESH_MSB1 = 0b0000_0010;
        const FIFO_THRESH_MSB0 = 0b0000_0001;

        const FIFO_THRESH_MSB = Self::FIFO_THRESH_MSB2.bits | Self::FIFO_THRESH_MSB1.bits | Self::FIFO_THRESH_MSB0.bits;
    }
}

impl FifoConfig2 {
    pub const fn with_fifo_wtrmk_threshold(self, threshold: u8) -> Self {
        Self::from_bits_truncate(threshold)
    }
}

cfg_register! {
    FifoPwrConfig: 0x29 = 0x00 {
        const READ_DISABLE = 0b0000_0001;
    }
}

impl FifoPwrConfig {
    pub const fn fifo_pwr_disable(&self) -> bool {
        self.intersects(Self::READ_DISABLE)
    }
    pub const fn with_fifo_pwr_disable(self, enabled: bool) -> Self {
        if enabled {
            self.union(Self::READ_DISABLE)
        } else {
            self.difference(Self::READ_DISABLE)
        }
    }
}

cfg_register! {
    AutoLowPow0: 0x2A = 0x00 {
        const LP_TIMEOUT_MSB7 = 0b1000_0000;
        const LP_TIMEOUT_MSB6 = 0b0100_0000;
        const LP_TIMEOUT_MSB5 = 0b0010_0000;
        const LP_TIMEOUT_MSB4 = 0b0001_0000;
        const LP_TIMEOUT_MSB3 = 0b0000_1000;
        const LP_TIMEOUT_MSB2 = 0b0000_0100;
        const LP_TIMEOUT_MSB1 = 0b0000_0010;
        const LP_TIMEOUT_MSB0 = 0b0000_0001;
    }
}

impl AutoLowPow0 {
    pub const fn with_auto_lp_timeout_msb(self, timeout: u16) -> Self {
        Self::from_bits_truncate((timeout >> 4).to_le_bytes()[0])
    }
}

cfg_register! {
    AutoLowPow1: 0x2B = 0x00 {
        const LP_TIMEOUT_LSB3 = 0b1000_0000;
        const LP_TIMEOUT_LSB2 = 0b0100_0000;
        const LP_TIMEOUT_LSB1 = 0b0010_0000;
        const LP_TIMEOUT_LSB0 = 0b0001_0000;
        const AUT_LP_TIMEOUT1 = 0b0000_1000;
        const AUT_LP_TIMEOUT0 = 0b0000_0100;
        const GEN1_INT_TRIG   = 0b0000_0010;
        const DRDY_TRIG       = 0b0000_0001;

        const LP_TIMEOUT_LSB = Self::LP_TIMEOUT_LSB3.bits | Self::LP_TIMEOUT_LSB2.bits | Self::LP_TIMEOUT_LSB1.bits | Self::LP_TIMEOUT_LSB0.bits;
        const AUTO_LP_TIMEOUT = Self::AUT_LP_TIMEOUT1.bits | Self::AUT_LP_TIMEOUT0.bits;
    }
}

impl AutoLowPow1 {
    pub const fn with_auto_lp_timeout_lsb(self, timeout: u16) -> Self {
        self.difference(Self::LP_TIMEOUT_LSB)
            .union(Self::from_bits_truncate((timeout << 4).to_le_bytes()[0]))
    }
    pub const fn with_auto_lp_timeout_mode(self, mode: AutoLPTimeoutTrigger) -> Self {
        match mode {
            AutoLPTimeoutTrigger::TimeoutDisabled => self.difference(Self::AUTO_LP_TIMEOUT),
            AutoLPTimeoutTrigger::TimeoutEnabledNoReset => {
                self.difference(Self::AUTO_LP_TIMEOUT).union(Self::AUT_LP_TIMEOUT0)
            }
            AutoLPTimeoutTrigger::TimeoutEnabledGen2IntReset => {
                self.difference(Self::AUTO_LP_TIMEOUT).union(Self::AUT_LP_TIMEOUT1)
            }
        }
    }
    pub const fn with_gen1_int_trigger(self, enabled: bool) -> Self {
        if enabled {
            self.union(Self::GEN1_INT_TRIG)
        } else {
            self.difference(Self::GEN1_INT_TRIG)
        }
    }
    pub const fn with_drdy_trigger(self, enabled: bool) -> Self {
        if enabled {
            self.union(Self::DRDY_TRIG)
        } else {
            self.difference(Self::DRDY_TRIG)
        }
    }
}

cfg_register! {
    AutoWakeup0: 0x2C = 0x00 {
        const WKUP_TIMEOUT_MSB7 = 0b1000_0000;
        const WKUP_TIMEOUT_MSB6 = 0b0100_0000;
        const WKUP_TIMEOUT_MSB5 = 0b0010_0000;
        const WKUP_TIMEOUT_MSB4 = 0b0001_0000;
        const WKUP_TIMEOUT_MSB3 = 0b0000_1000;
        const WKUP_TIMEOUT_MSB2 = 0b0000_0100;
        const WKUP_TIMEOUT_MSB1 = 0b0000_0010;
        const WKUP_TIMEOUT_MSB0 = 0b0000_0001;
    }
}

impl AutoWakeup0 {
    pub const fn with_wakeup_timeout_msb(self, timeout: u16) -> Self {
        Self::from_bits_truncate((timeout >> 4).to_le_bytes()[0])
    }
}

cfg_register! {
    AutoWakeup1: 0x2D = 0x00 {
        const WKUP_TIMEOUT_LSB3 = 0b1000_0000;
        const WKUP_TIMEOUT_LSB2 = 0b0100_0000;
        const WKUP_TIMEOUT_LSB1 = 0b0010_0000;
        const WKUP_TIMEOUT_LSB0 = 0b0001_0000;
        const WKUP_TIMEOUT      = 0b0000_0100;
        const WKUP_INT          = 0b0000_0010;

        const WKUP_TIMEOUT_LSB = Self::WKUP_TIMEOUT_LSB3.bits | Self::WKUP_TIMEOUT_LSB2.bits | Self::WKUP_TIMEOUT_LSB1.bits | Self::WKUP_TIMEOUT_LSB0.bits;
    }
}

impl AutoWakeup1 {
    pub const fn with_wakeup_timeout_lsb(self, timeout: u16) -> Self {
        self.difference(Self::WKUP_TIMEOUT_LSB)
            .union(Self::from_bits_truncate((timeout << 4).to_le_bytes()[0]))
    }
    pub const fn with_wakeup_timeout(self, enabled: bool) -> Self {
        if enabled {
            self.union(Self::WKUP_TIMEOUT)
        } else {
            self.difference(Self::WKUP_TIMEOUT)
        }
    }
    pub const fn with_wakeup_int(self, enabled: bool) -> Self {
        if enabled {
            self.union(Self::WKUP_INT)
        } else {
            self.difference(Self::WKUP_INT)
        }
    }
}

cfg_register! {
    WakeupIntConfig0: 0x2F = 0x00 {
        const WKUP_Z_EN = 0b1000_0000;
        const WKUP_Y_EN = 0b0100_0000;
        const WKUP_X_EN = 0b0010_0000;
        const NUM_SMPL2 = 0b0001_0000;
        const NUM_SMPL1 = 0b0000_1000;
        const NUM_SMPL0 = 0b0000_0100;
        const WKUP_REF1 = 0b0000_0010;
        const WKUP_REF0 = 0b0000_0001;

        const NUM_SAMPLES = Self::NUM_SMPL2.bits | Self::NUM_SMPL1.bits | Self::NUM_SMPL0.bits;
        const WKUP_REFU = Self::WKUP_REF1.bits | Self::WKUP_REF0.bits;
    }
}

impl WakeupIntConfig0 {
    pub const fn wkup_int_en(&self) -> bool {
        self.intersects(Self::WKUP_X_EN.union(Self::WKUP_Y_EN).union(Self::WKUP_Z_EN))
    }
    pub const fn with_z_axis(self, enabled: bool) -> Self {
        if enabled {
            self.union(Self::WKUP_Z_EN)
        } else {
            self.difference(Self::WKUP_Z_EN)
        }
    }
    pub const fn with_y_axis(self, enabled: bool) -> Self {
        if enabled {
            self.union(Self::WKUP_Y_EN)
        } else {
            self.difference(Self::WKUP_Y_EN)
        }
    }
    pub const fn with_x_axis(self, enabled: bool) -> Self {
        if enabled {
            self.union(Self::WKUP_X_EN)
        } else {
            self.difference(Self::WKUP_X_EN)
        }
    }
    pub const fn with_num_samples(self, num_samples: u8) -> Self {
        self.difference(Self::NUM_SAMPLES).union(Self::from_bits_truncate(num_samples << 2))
    }
    pub const fn with_reference_mode(self, ref_mode: WakeupIntRefMode) -> Self {
        match ref_mode {
            WakeupIntRefMode::Manual => self.difference(Self::WKUP_REFU),
            WakeupIntRefMode::OneTime => self.difference(Self::WKUP_REFU).union(Self::WKUP_REF0),
            WakeupIntRefMode::EveryTime => self.difference(Self::WKUP_REFU).union(Self::WKUP_REF1),
        }
    }
}

cfg_register! {
    WakeupIntConfig1: 0x30 = 0x00 {
        const WKUP_INT_THRESH7 = 0b1000_0000;
        const WKUP_INT_THRESH6 = 0b0100_0000;
        const WKUP_INT_THRESH5 = 0b0010_0000;
        const WKUP_INT_THRESH4 = 0b0001_0000;
        const WKUP_INT_THRESH3 = 0b0000_1000;
        const WKUP_INT_THRESH2 = 0b0000_0100;
        const WKUP_INT_THRESH1 = 0b0000_0010;
        const WKUP_INT_THRESH0 = 0b0000_0001;
    }
}

impl WakeupIntConfig1 {
    pub const fn with_threshold(self, threshold: u8) -> Self {
        Self::from_bits_truncate(threshold)
    }
}

cfg_register! {
    WakeupIntConfig2: 0x31 = 0x00 {
        const WKUP_REF_X7 = 0b1000_0000;
        const WKUP_REF_X6 = 0b0100_0000;
        const WKUP_REF_X5 = 0b0010_0000;
        const WKUP_REF_X4 = 0b0001_0000;
        const WKUP_REF_X3 = 0b0000_1000;
        const WKUP_REF_X2 = 0b0000_0100;
        const WKUP_REF_X1 = 0b0000_0010;
        const WKUP_REF_X0 = 0b0000_0001;
    }
}

impl WakeupIntConfig2 {
    pub const fn with_x_ref(self, x_ref: u8) -> Self {
        Self::from_bits_truncate(x_ref)
    }
}
cfg_register! {
    WakeupIntConfig3: 0x32 = 0x00 {
        const WKUP_REF_Y7 = 0b1000_0000;
        const WKUP_REF_Y6 = 0b0100_0000;
        const WKUP_REF_Y5 = 0b0010_0000;
        const WKUP_REF_Y4 = 0b0001_0000;
        const WKUP_REF_Y3 = 0b0000_1000;
        const WKUP_REF_Y2 = 0b0000_0100;
        const WKUP_REF_Y1 = 0b0000_0010;
        const WKUP_REF_Y0 = 0b0000_0001;
    }
}

impl WakeupIntConfig3 {
    pub const fn with_y_ref(self, y_ref: u8) -> Self {
        Self::from_bits_truncate(y_ref)
    }
}

cfg_register! {
    WakeupIntConfig4: 0x33 = 0x00 {
        const WKUP_REF_Z7 = 0b1000_0000;
        const WKUP_REF_Z6 = 0b0100_0000;
        const WKUP_REF_Z5 = 0b0010_0000;
        const WKUP_REF_Z4 = 0b0001_0000;
        const WKUP_REF_Z3 = 0b0000_1000;
        const WKUP_REF_Z2 = 0b0000_0100;
        const WKUP_REF_Z1 = 0b0000_0010;
        const WKUP_REF_Z0 = 0b0000_0001;
    }
}

impl WakeupIntConfig4 {
    pub const fn with_z_ref(self, z_ref: u8) -> Self {
        Self::from_bits_truncate(z_ref)
    }
}

cfg_register! {
    OrientChgConfig0: 0x35 = 0x00 {
        const ORIENT_Z_EN   = 0b1000_0000;
        const ORIENT_Y_EN   = 0b0100_0000;
        const ORIENT_X_EN   = 0b0010_0000;
        const ORIENT_SRC    = 0b0001_0000;
        const ORIENT_REFU1  = 0b0000_1000;
        const ORIENT_REFU0  = 0b0000_0100;

        const ORIENT_REFU = Self::ORIENT_REFU1.bits | Self::ORIENT_REFU0.bits;
    }
}

impl OrientChgConfig0 {
    pub const fn with_z_axis(self, enabled: bool) -> Self {
        if enabled {
            self.union(Self::ORIENT_Z_EN)
        } else {
            self.difference(Self::ORIENT_Z_EN)
        }
    }
    pub const fn with_y_axis(self, enabled: bool) -> Self {
        if enabled {
            self.union(Self::ORIENT_Y_EN)
        } else {
            self.difference(Self::ORIENT_Y_EN)
        }
    }
    pub const fn with_x_axis(self, enabled: bool) -> Self {
        if enabled {
            self.union(Self::ORIENT_X_EN)
        } else {
            self.difference(Self::ORIENT_X_EN)
        }
    }
    pub const fn with_data_src(self, src: DataSource) -> Self {
        match src {
            DataSource::AccFilt1 => unreachable!(), // Handled in the public API
            DataSource::AccFilt2 => self.difference(Self::ORIENT_SRC),
            DataSource::AccFilt2Lp => self.union(Self::ORIENT_SRC),
        }
    }
    pub const fn with_update_mode(self, update_mode: OrientIntRefMode) -> Self {
        match update_mode {
            OrientIntRefMode::Manual => self.difference(Self::ORIENT_REFU),
            OrientIntRefMode::AccFilt2 => {
                self.difference(Self::ORIENT_REFU).union(Self::ORIENT_REFU0)
            }
            OrientIntRefMode::AccFilt2Lp => {
                self.difference(Self::ORIENT_REFU).union(Self::ORIENT_REFU1)
            }
        }
    }
}

cfg_register! {
    OrientChgConfig1: 0x36 = 0x00 {
        const ORIENT_THRESH7 = 0b1000_0000;
        const ORIENT_THRESH6 = 0b0100_0000;
        const ORIENT_THRESH5 = 0b0010_0000;
        const ORIENT_THRESH4 = 0b0001_0000;
        const ORIENT_THRESH3 = 0b0000_1000;
        const ORIENT_THRESH2 = 0b0000_0100;
        const ORIENT_THRESH1 = 0b0000_0010;
        const ORIENT_THRESH0 = 0b0000_0001;
    }
}

impl OrientChgConfig1 {
    pub const fn with_orient_thresh(self, threshold: u8) -> Self {
        Self::from_bits_truncate(threshold)
    }
}

cfg_register! {
    OrientChgConfig3: 0x38 = 0x00 {
        const ORIENT_DUR7 = 0b1000_0000;
        const ORIENT_DUR6 = 0b0100_0000;
        const ORIENT_DUR5 = 0b0010_0000;
        const ORIENT_DUR4 = 0b0001_0000;
        const ORIENT_DUR3 = 0b0000_1000;
        const ORIENT_DUR2 = 0b0000_0100;
        const ORIENT_DUR1 = 0b0000_0010;
        const ORIENT_DUR0 = 0b0000_0001;
    }
}

impl OrientChgConfig3 {
    pub const fn with_orient_dur(self, duration: u8) -> Self {
        Self::from_bits_truncate(duration)
    }
}

cfg_register! {
    OrientChgConfig4: 0x39 = 0x00 {
        const REFX_LSB7 = 0b1000_0000;
        const REFX_LSB6 = 0b0100_0000;
        const REFX_LSB5 = 0b0010_0000;
        const REFX_LSB4 = 0b0001_0000;
        const REFX_LSB3 = 0b0000_1000;
        const REFX_LSB2 = 0b0000_0100;
        const REFX_LSB1 = 0b0000_0010;
        const REFX_LSB0 = 0b0000_0001;
    }
}

impl OrientChgConfig4 {
    pub const fn with_refx_lsb(self, ref_x: i16) -> Self {
        Self::from_bits_truncate(ref_x.to_le_bytes()[0])
    }
}

cfg_register! {
    OrientChgConfig5: 0x3A = 0x00 {
        const REFX_MSB3 = 0b0000_1000;
        const REFX_MSB2 = 0b0000_0100;
        const REFX_MSB1 = 0b0000_0010;
        const REFX_MSB0 = 0b0000_0001;
    }
}

impl OrientChgConfig5 {
    pub const fn with_refx_msb(self, ref_x: i16) -> Self {
        Self::from_bits_truncate(ref_x.to_le_bytes()[1])
    }
}

cfg_register! {
    OrientChgConfig6: 0x3B = 0x00 {
        const REFY_LSB7 = 0b1000_0000;
        const REFY_LSB6 = 0b0100_0000;
        const REFY_LSB5 = 0b0010_0000;
        const REFY_LSB4 = 0b0001_0000;
        const REFY_LSB3 = 0b0000_1000;
        const REFY_LSB2 = 0b0000_0100;
        const REFY_LSB1 = 0b0000_0010;
        const REFY_LSB0 = 0b0000_0001;
    }
}

impl OrientChgConfig6 {
    pub const fn with_refy_lsb(self, ref_y: i16) -> Self {
        Self::from_bits_truncate(ref_y.to_le_bytes()[0])
    }
}

cfg_register! {
    OrientChgConfig7: 0x3C = 0x00 {
        const REFY_MSB3 = 0b0000_1000;
        const REFY_MSB2 = 0b0000_0100;
        const REFY_MSB1 = 0b0000_0010;
        const REFY_MSB0 = 0b0000_0001;
    }
}

impl OrientChgConfig7 {
    pub const fn with_refy_msb(self, ref_y: i16) -> Self {
        Self::from_bits_truncate(ref_y.to_le_bytes()[1])
    }
}

cfg_register! {
    OrientChgConfig8: 0x3D = 0x00 {
        const REFZ_LSB7 = 0b1000_0000;
        const REFZ_LSB6 = 0b0100_0000;
        const REFZ_LSB5 = 0b0010_0000;
        const REFZ_LSB4 = 0b0001_0000;
        const REFZ_LSB3 = 0b0000_1000;
        const REFZ_LSB2 = 0b0000_0100;
        const REFZ_LSB1 = 0b0000_0010;
        const REFZ_LSB0 = 0b0000_0001;
    }
}

impl OrientChgConfig8 {
    pub const fn with_refz_lsb(self, ref_z: i16) -> Self {
        Self::from_bits_truncate(ref_z.to_le_bytes()[0])
    }
}

cfg_register! {
    OrientChgConfig9: 0x3E = 0x00 {
        const REFZ_MSB3 = 0b0000_1000;
        const REFZ_MSB2 = 0b0000_0100;
        const REFZ_MSB1 = 0b0000_0010;
        const REFZ_MSB0 = 0b0000_0001;
    }
}

impl OrientChgConfig9 {
    pub const fn with_refz_msb(self, ref_z: i16) -> Self {
        Self::from_bits_truncate(ref_z.to_le_bytes()[1])
    }
}

cfg_register! {
    Gen1IntConfig0: 0x3F = 0x00 {
        const ACT_Z_EN  = 0b1000_0000;
        const ACT_Y_EN  = 0b0100_0000;
        const ACT_X_EN  = 0b0010_0000;
        const SRC       = 0b0001_0000;
        const ACT_REFU1 = 0b0000_1000;
        const ACT_REFU0 = 0b0000_0100;
        const ACT_HYST1 = 0b0000_0010;
        const ACT_HYST0 = 0b0000_0001;

        const ACT_REFU_MODE = Self::ACT_REFU1.bits | Self::ACT_REFU0.bits;
        const ACT_HYST = Self::ACT_HYST1.bits | Self::ACT_HYST0.bits;
    }
}

impl Gen1IntConfig0 {
    pub const fn with_z_axis(&self, enabled: bool) -> Self {
        if enabled {
            self.union(Self::ACT_Z_EN)
        } else {
            self.difference(Self::ACT_Z_EN)
        }
    }
    pub const fn with_y_axis(&self, enabled: bool) -> Self {
        if enabled {
            self.union(Self::ACT_Y_EN)
        } else {
            self.difference(Self::ACT_Y_EN)
        }
    }
    pub const fn with_x_axis(&self, enabled: bool) -> Self {
        if enabled {
            self.union(Self::ACT_X_EN)
        } else {
            self.difference(Self::ACT_X_EN)
        }
    }
    pub const fn with_src(&self, src: DataSource) -> Self {
        match src {
            DataSource::AccFilt1 => self.difference(Self::SRC),
            DataSource::AccFilt2 => self.union(Self::SRC),
            DataSource::AccFilt2Lp => unreachable!(), // Handled in the public API
        }
    }
    pub const fn src(&self) -> DataSource {
        if self.intersects(Self::SRC) {
            DataSource::AccFilt2
        } else {
            DataSource::AccFilt1
        }
    }
    pub const fn with_refu_mode(&self, mode: GenIntRefMode) -> Self {
        match mode {
            GenIntRefMode::Manual => self.difference(Self::ACT_REFU_MODE),
            GenIntRefMode::OneTime => self.difference(Self::ACT_REFU_MODE).union(Self::ACT_REFU0),
            GenIntRefMode::EveryTimeFromSrc => {
                self.difference(Self::ACT_REFU_MODE).union(Self::ACT_REFU1)
            }
            GenIntRefMode::EveryTimeFromLp => self.union(Self::ACT_REFU_MODE),
        }
    }
    pub const fn with_act_hysteresis(&self, hysteresis: Hysteresis) -> Self {
        match hysteresis {
            Hysteresis::None => self.difference(Self::ACT_HYST),
            Hysteresis::Hyst24mg => self.difference(Self::ACT_HYST).union(Self::ACT_HYST0),
            Hysteresis::Hyst48mg => self.difference(Self::ACT_HYST).union(Self::ACT_HYST1),
            Hysteresis::Hyst96mg => self.union(Self::ACT_HYST),
        }
    }
}

cfg_register! {
    Gen1IntConfig1: 0x40 = 0x00 {
        const CRITERION_SEL = 0b0000_0010;
        const COMB_SEL      = 0b0000_0001;
    }
}

impl Gen1IntConfig1 {
    pub const fn with_criterion_sel(&self, mode: GenIntCriterionMode) -> Self {
        match mode {
            GenIntCriterionMode::Inactivity => self.difference(Self::CRITERION_SEL),
            GenIntCriterionMode::Activity => self.union(Self::CRITERION_SEL),
        }
    }
    pub const fn with_comb_sel(&self, mode: GenIntLogicMode) -> Self {
        match mode {
            GenIntLogicMode::Or => self.difference(Self::COMB_SEL),
            GenIntLogicMode::And => self.union(Self::COMB_SEL),
        }
    }
}

cfg_register! {
    Gen1IntConfig2: 0x41 = 0x00 {
        const THRESHOLD7 = 0b1000_0000;
        const THRESHOLD6 = 0b0100_0000;
        const THRESHOLD5 = 0b0010_0000;
        const THRESHOLD4 = 0b0001_0000;
        const THRESHOLD3 = 0b0000_1000;
        const THRESHOLD2 = 0b0000_0100;
        const THRESHOLD1 = 0b0000_0010;
        const THRESHOLD0 = 0b0000_0001;
    }
}

impl Gen1IntConfig2 {
    pub const fn with_threshold(&self, threshold: u8) -> Self {
        Self::from_bits_truncate(threshold)
    }
}

cfg_register! {
    Gen1IntConfig3: 0x42 = 0x00 {
        const DURATION_MSB7 = 0b1000_0000;
        const DURATION_MSB6 = 0b0100_0000;
        const DURATION_MSB5 = 0b0010_0000;
        const DURATION_MSB4 = 0b0001_0000;
        const DURATION_MSB3 = 0b0000_1000;
        const DURATION_MSB2 = 0b0000_0100;
        const DURATION_MSB1 = 0b0000_0010;
        const DURATION_MSB0 = 0b0000_0001;
    }
}

impl Gen1IntConfig3 {
    pub const fn with_duration_msb(&self, duration: u8) -> Self {
        Self::from_bits_truncate(duration)
    }
}

cfg_register! {
    Gen1IntConfig31: 0x43 = 0x00 {
        const DURATION_LSB7 = 0b1000_0000;
        const DURATION_LSB6 = 0b0100_0000;
        const DURATION_LSB5 = 0b0010_0000;
        const DURATION_LSB4 = 0b0001_0000;
        const DURATION_LSB3 = 0b0000_1000;
        const DURATION_LSB2 = 0b0000_0100;
        const DURATION_LSB1 = 0b0000_0010;
        const DURATION_LSB0 = 0b0000_0001;
    }
}

impl Gen1IntConfig31 {
    pub const fn with_duration_lsb(&self, duration: u8) -> Self {
        Self::from_bits_truncate(duration)
    }
}

cfg_register! {
    Gen1IntConfig4: 0x44 = 0x00 {
        const REFX_LSB7 = 0b1000_0000;
        const REFX_LSB6 = 0b0100_0000;
        const REFX_LSB5 = 0b0010_0000;
        const REFX_LSB4 = 0b0001_0000;
        const REFX_LSB3 = 0b0000_1000;
        const REFX_LSB2 = 0b0000_0100;
        const REFX_LSB1 = 0b0000_0010;
        const REFX_LSB0 = 0b0000_0001;
    }
}

impl Gen1IntConfig4 {
    pub const fn with_ref_x_lsb(&self, ref_x: u8) -> Self {
        Self::from_bits_truncate(ref_x)
    }
}

cfg_register! {
    Gen1IntConfig5: 0x45 = 0x00 {
        const REFX_MSB3 = 0b0000_1000;
        const REFX_MSB2 = 0b0000_0100;
        const REFX_MSB1 = 0b0000_0010;
        const REFX_MSB0 = 0b0000_0001;
    }
}

impl Gen1IntConfig5 {
    pub const fn with_ref_x_msb(&self, ref_x: u8) -> Self {
        Self::from_bits_truncate(ref_x)
    }
}

cfg_register! {
    Gen1IntConfig6: 0x46 = 0x00 {
        const REFY_LSB7 = 0b1000_0000;
        const REFY_LSB6 = 0b0100_0000;
        const REFY_LSB5 = 0b0010_0000;
        const REFY_LSB4 = 0b0001_0000;
        const REFY_LSB3 = 0b0000_1000;
        const REFY_LSB2 = 0b0000_0100;
        const REFY_LSB1 = 0b0000_0010;
        const REFY_LSB0 = 0b0000_0001;
    }
}

impl Gen1IntConfig6 {
    pub const fn with_ref_y_lsb(&self, ref_y: u8) -> Self {
        Self::from_bits_truncate(ref_y)
    }
}

cfg_register! {
    Gen1IntConfig7: 0x47 = 0x00 {
        const REFY_MSB3 = 0b0000_1000;
        const REFY_MSB2 = 0b0000_0100;
        const REFY_MSB1 = 0b0000_0010;
        const REFY_MSB0 = 0b0000_0001;
    }
}

impl Gen1IntConfig7 {
    pub const fn with_ref_y_msb(&self, ref_y: u8) -> Self {
        Self::from_bits_truncate(ref_y)
    }
}

cfg_register! {
    Gen1IntConfig8: 0x48 = 0x00 {
        const REFZ_LSB7 = 0b1000_0000;
        const REFZ_LSB6 = 0b0100_0000;
        const REFZ_LSB5 = 0b0010_0000;
        const REFZ_LSB4 = 0b0001_0000;
        const REFZ_LSB3 = 0b0000_1000;
        const REFZ_LSB2 = 0b0000_0100;
        const REFZ_LSB1 = 0b0000_0010;
        const REFZ_LSB0 = 0b0000_0001;
    }
}

impl Gen1IntConfig8 {
    pub const fn with_ref_z_lsb(&self, ref_z: u8) -> Self {
        Self::from_bits_truncate(ref_z)
    }
}

cfg_register! {
    Gen1IntConfig9: 0x49 = 0x00 {
        const REFZ_MSB3 = 0b0000_1000;
        const REFZ_MSB2 = 0b0000_0100;
        const REFZ_MSB1 = 0b0000_0010;
        const REFZ_MSB0 = 0b0000_0001;
    }
}

impl Gen1IntConfig9 {
    pub const fn with_ref_z_msb(&self, ref_z: u8) -> Self {
        Self::from_bits_truncate(ref_z)
    }
}

cfg_register! {
    Gen2IntConfig0: 0x4A = 0x00 {
        const ACT_Z_EN  = 0b1000_0000;
        const ACT_Y_EN  = 0b0100_0000;
        const ACT_X_EN  = 0b0010_0000;
        const SRC       = 0b0001_0000;
        const ACT_REFU1 = 0b0000_1000;
        const ACT_REFU0 = 0b0000_0100;
        const ACT_HYST1 = 0b0000_0010;
        const ACT_HYST0 = 0b0000_0001;

        const ACT_REFU_MODE = Self::ACT_REFU1.bits | Self::ACT_REFU0.bits;
        const ACT_HYST = Self::ACT_HYST1.bits | Self::ACT_HYST0.bits;
    }
}

impl Gen2IntConfig0 {
    pub const fn with_z_axis(&self, enabled: bool) -> Self {
        if enabled {
            self.union(Self::ACT_Z_EN)
        } else {
            self.difference(Self::ACT_Z_EN)
        }
    }
    pub const fn with_y_axis(&self, enabled: bool) -> Self {
        if enabled {
            self.union(Self::ACT_Y_EN)
        } else {
            self.difference(Self::ACT_Y_EN)
        }
    }
    pub const fn with_x_axis(&self, enabled: bool) -> Self {
        if enabled {
            self.union(Self::ACT_X_EN)
        } else {
            self.difference(Self::ACT_X_EN)
        }
    }
    pub const fn with_src(&self, src: DataSource) -> Self {
        match src {
            DataSource::AccFilt1 => self.difference(Self::SRC),
            DataSource::AccFilt2 => self.union(Self::SRC),
            DataSource::AccFilt2Lp => unreachable!(), // Handled in the public API
        }
    }
    pub const fn src(&self) -> DataSource {
        if self.intersects(Self::SRC) {
            DataSource::AccFilt2
        } else {
            DataSource::AccFilt1
        }
    }
    pub const fn with_refu_mode(&self, mode: GenIntRefMode) -> Self {
        match mode {
            GenIntRefMode::Manual => self.difference(Self::ACT_REFU_MODE),
            GenIntRefMode::OneTime => self.difference(Self::ACT_REFU_MODE).union(Self::ACT_REFU0),
            GenIntRefMode::EveryTimeFromSrc => {
                self.difference(Self::ACT_REFU_MODE).union(Self::ACT_REFU1)
            }
            GenIntRefMode::EveryTimeFromLp => self.union(Self::ACT_REFU_MODE),
        }
    }
    pub const fn with_act_hysteresis(&self, hysteresis: Hysteresis) -> Self {
        match hysteresis {
            Hysteresis::None => self.difference(Self::ACT_HYST),
            Hysteresis::Hyst24mg => self.difference(Self::ACT_HYST).union(Self::ACT_HYST0),
            Hysteresis::Hyst48mg => self.difference(Self::ACT_HYST).union(Self::ACT_HYST1),
            Hysteresis::Hyst96mg => self.union(Self::ACT_HYST),
        }
    }
}

cfg_register! {
    Gen2IntConfig1: 0x4B = 0x00 {
        const CRITERION_SEL = 0b0000_0010;
        const COMB_SEL      = 0b0000_0001;
    }
}

impl Gen2IntConfig1 {
    pub const fn with_criterion_sel(&self, mode: GenIntCriterionMode) -> Self {
        match mode {
            GenIntCriterionMode::Inactivity => self.difference(Self::CRITERION_SEL),
            GenIntCriterionMode::Activity => self.union(Self::CRITERION_SEL),
        }
    }
    pub const fn with_comb_sel(&self, mode: GenIntLogicMode) -> Self {
        match mode {
            GenIntLogicMode::Or => self.difference(Self::COMB_SEL),
            GenIntLogicMode::And => self.union(Self::COMB_SEL),
        }
    }
}

cfg_register! {
    Gen2IntConfig2: 0x4C = 0x00 {
        const THRESHOLD7 = 0b1000_0000;
        const THRESHOLD6 = 0b0100_0000;
        const THRESHOLD5 = 0b0010_0000;
        const THRESHOLD4 = 0b0001_0000;
        const THRESHOLD3 = 0b0000_1000;
        const THRESHOLD2 = 0b0000_0100;
        const THRESHOLD1 = 0b0000_0010;
        const THRESHOLD0 = 0b0000_0001;
    }
}

impl Gen2IntConfig2 {
    pub const fn with_threshold(&self, threshold: u8) -> Self {
        Self::from_bits_truncate(threshold)
    }
}

cfg_register! {
    Gen2IntConfig3: 0x4D = 0x00 {
        const DURATION_MSB7 = 0b1000_0000;
        const DURATION_MSB6 = 0b0100_0000;
        const DURATION_MSB5 = 0b0010_0000;
        const DURATION_MSB4 = 0b0001_0000;
        const DURATION_MSB3 = 0b0000_1000;
        const DURATION_MSB2 = 0b0000_0100;
        const DURATION_MSB1 = 0b0000_0010;
        const DURATION_MSB0 = 0b0000_0001;
    }
}

impl Gen2IntConfig3 {
    pub const fn with_duration_msb(&self, duration: u8) -> Self {
        Self::from_bits_truncate(duration)
    }
}

cfg_register! {
    Gen2IntConfig31: 0x4E = 0x00 {
        const DURATION_LSB7 = 0b1000_0000;
        const DURATION_LSB6 = 0b0100_0000;
        const DURATION_LSB5 = 0b0010_0000;
        const DURATION_LSB4 = 0b0001_0000;
        const DURATION_LSB3 = 0b0000_1000;
        const DURATION_LSB2 = 0b0000_0100;
        const DURATION_LSB1 = 0b0000_0010;
        const DURATION_LSB0 = 0b0000_0001;
    }
}

impl Gen2IntConfig31 {
    pub const fn with_duration_lsb(&self, duration: u8) -> Self {
        Self::from_bits_truncate(duration)
    }
}

cfg_register! {
    Gen2IntConfig4: 0x4F = 0x00 {
        const REFX_LSB7 = 0b1000_0000;
        const REFX_LSB6 = 0b0100_0000;
        const REFX_LSB5 = 0b0010_0000;
        const REFX_LSB4 = 0b0001_0000;
        const REFX_LSB3 = 0b0000_1000;
        const REFX_LSB2 = 0b0000_0100;
        const REFX_LSB1 = 0b0000_0010;
        const REFX_LSB0 = 0b0000_0001;
    }
}

impl Gen2IntConfig4 {
    pub const fn with_ref_x_lsb(&self, ref_x: u8) -> Self {
        Self::from_bits_truncate(ref_x)
    }
}

cfg_register! {
    Gen2IntConfig5: 0x50 = 0x00 {
        const REFX_MSB3 = 0b0000_1000;
        const REFX_MSB2 = 0b0000_0100;
        const REFX_MSB1 = 0b0000_0010;
        const REFX_MSB0 = 0b0000_0001;
    }
}

impl Gen2IntConfig5 {
    pub const fn with_ref_x_msb(&self, ref_x: u8) -> Self {
        Self::from_bits_truncate(ref_x)
    }
}

cfg_register! {
    Gen2IntConfig6: 0x51 = 0x00 {
        const REFY_LSB7 = 0b1000_0000;
        const REFY_LSB6 = 0b0100_0000;
        const REFY_LSB5 = 0b0010_0000;
        const REFY_LSB4 = 0b0001_0000;
        const REFY_LSB3 = 0b0000_1000;
        const REFY_LSB2 = 0b0000_0100;
        const REFY_LSB1 = 0b0000_0010;
        const REFY_LSB0 = 0b0000_0001;
    }
}

impl Gen2IntConfig6 {
    pub const fn with_ref_y_lsb(&self, ref_y: u8) -> Self {
        Self::from_bits_truncate(ref_y)
    }
}

cfg_register! {
    Gen2IntConfig7: 0x52 = 0x00 {
        const REFY_MSB3 = 0b0000_1000;
        const REFY_MSB2 = 0b0000_0100;
        const REFY_MSB1 = 0b0000_0010;
        const REFY_MSB0 = 0b0000_0001;
    }
}

impl Gen2IntConfig7 {
    pub const fn with_ref_y_msb(&self, ref_y: u8) -> Self {
        Self::from_bits_truncate(ref_y)
    }
}

cfg_register! {
    Gen2IntConfig8: 0x53 = 0x00 {
        const REFZ_LSB7 = 0b1000_0000;
        const REFZ_LSB6 = 0b0100_0000;
        const REFZ_LSB5 = 0b0010_0000;
        const REFZ_LSB4 = 0b0001_0000;
        const REFZ_LSB3 = 0b0000_1000;
        const REFZ_LSB2 = 0b0000_0100;
        const REFZ_LSB1 = 0b0000_0010;
        const REFZ_LSB0 = 0b0000_0001;
    }
}

impl Gen2IntConfig8 {
    pub const fn with_ref_z_lsb(&self, ref_z: u8) -> Self {
        Self::from_bits_truncate(ref_z)
    }
}

cfg_register! {
    Gen2IntConfig9: 0x54 = 0x00 {
        const REFZ_MSB3 = 0b0000_1000;
        const REFZ_MSB2 = 0b0000_0100;
        const REFZ_MSB1 = 0b0000_0010;
        const REFZ_MSB0 = 0b0000_0001;
    }
}

impl Gen2IntConfig9 {
    pub const fn with_ref_z_msb(&self, ref_z: u8) -> Self {
        Self::from_bits_truncate(ref_z)
    }
}

cfg_register! {
    ActChgConfig0: 0x55 = 0x00 {
        const ACTCH_THRES7 = 0b1000_0000;
        const ACTCH_THRES6 = 0b0100_0000;
        const ACTCH_THRES5 = 0b0010_0000;
        const ACTCH_THRES4 = 0b0001_0000;
        const ACTCH_THRES3 = 0b0000_1000;
        const ACTCH_THRES2 = 0b0000_0100;
        const ACTCH_THRES1 = 0b0000_0010;
        const ACTCH_THRES0 = 0b0000_0001;
    }
}

impl ActChgConfig0 {
    pub const fn with_actch_thres(self, threshold: u8) -> Self {
        Self::from_bits_truncate(threshold)
    }
}

cfg_register! {
    ActChgConfig1: 0x56 = 0x00 {
        const ACTCH_Z_EN = 0b1000_0000;
        const ACTCH_Y_EN = 0b0100_0000;
        const ACTCH_X_EN = 0b0010_0000;
        const ACTCH_SRC  = 0b0001_0000;
        const NUM_SMPLS3 = 0b0000_1000;
        const NUM_SMPLS2 = 0b0000_0100;
        const NUM_SMPLS1 = 0b0000_0010;
        const NUM_SMPLS0 = 0b0000_0001;

        const NUM_SAMPLES = Self::NUM_SMPLS3.bits | Self::NUM_SMPLS2.bits | Self::NUM_SMPLS1.bits | Self::NUM_SMPLS0.bits;
    }
}

impl ActChgConfig1 {
    pub const fn with_z_axis(self, enabled: bool) -> Self {
        if enabled {
            self.union(Self::ACTCH_Z_EN)
        } else {
            self.difference(Self::ACTCH_Z_EN)
        }
    }
    pub const fn with_y_axis(self, enabled: bool) -> Self {
        if enabled {
            self.union(Self::ACTCH_Y_EN)
        } else {
            self.difference(Self::ACTCH_Y_EN)
        }
    }
    pub const fn with_x_axis(self, enabled: bool) -> Self {
        if enabled {
            self.union(Self::ACTCH_X_EN)
        } else {
            self.difference(Self::ACTCH_X_EN)
        }
    }
    pub const fn src(&self) -> DataSource {
        if self.intersects(Self::ACTCH_SRC) {
            DataSource::AccFilt2
        } else {
            DataSource::AccFilt1
        }
    }
    pub const fn with_dta_src(self, src: DataSource) -> Self {
        match src {
            DataSource::AccFilt1 => self.difference(Self::ACTCH_SRC),
            DataSource::AccFilt2 => self.union(Self::ACTCH_SRC),
            DataSource::AccFilt2Lp => unreachable!(),
        }
    }
    pub const fn with_observation_period(self, period: ActChgObsPeriod) -> Self {
        match period {
            ActChgObsPeriod::Samples32 => self.difference(Self::NUM_SAMPLES),
            ActChgObsPeriod::Samples64 => {
                self.difference(Self::NUM_SAMPLES).union(Self::NUM_SMPLS0)
            }
            ActChgObsPeriod::Samples128 => {
                self.difference(Self::NUM_SAMPLES).union(Self::NUM_SMPLS1)
            }
            ActChgObsPeriod::Samples256 => {
                self.difference(Self::NUM_SAMPLES).union(Self::NUM_SMPLS1).union(Self::NUM_SMPLS0)
            }
            ActChgObsPeriod::Samples512 => {
                self.difference(Self::NUM_SAMPLES).union(Self::NUM_SMPLS2)
            }
        }
    }
}

cfg_register! {
    TapConfig0: 0x57 = 0x00 {
        const SEL_AXIS1 = 0b0001_0000;
        const SEL_AXIS0 = 0b0000_1000;
        const TAP_SENS2 = 0b0000_0100;
        const TAP_SENS1 = 0b0000_0010;
        const TAP_SENS0 = 0b0000_0001;

        const SEL_AXIS = Self::SEL_AXIS1.bits | Self::SEL_AXIS0.bits;
        const TAP_SENS = Self::TAP_SENS2.bits | Self::TAP_SENS1.bits | Self::TAP_SENS0.bits;
    }
}

impl TapConfig0 {
    pub const fn with_axis(self, axis: Axis) -> Self {
        match axis {
            Axis::Z => self.difference(Self::SEL_AXIS),
            Axis::Y => self.difference(Self::SEL_AXIS).union(Self::SEL_AXIS0),
            Axis::X => self.difference(Self::SEL_AXIS).union(Self::SEL_AXIS1),
        }
    }
    pub const fn with_sensitivity(self, sens: TapSensitivity) -> Self {
        match sens {
            TapSensitivity::SENS0 => self.difference(Self::TAP_SENS),
            TapSensitivity::SENS1 => self.difference(Self::TAP_SENS).union(Self::TAP_SENS0),
            TapSensitivity::SENS2 => self.difference(Self::TAP_SENS).union(Self::TAP_SENS1),
            TapSensitivity::SENS3 => {
                self.difference(Self::TAP_SENS).union(Self::TAP_SENS1).union(Self::TAP_SENS0)
            }
            TapSensitivity::SENS4 => self.difference(Self::TAP_SENS).union(Self::TAP_SENS2),
            TapSensitivity::SENS5 => {
                self.difference(Self::TAP_SENS).union(Self::TAP_SENS2).union(Self::TAP_SENS0)
            }
            TapSensitivity::SENS6 => {
                self.difference(Self::TAP_SENS).union(Self::TAP_SENS2).union(Self::TAP_SENS1)
            }
            TapSensitivity::SENS7 => self.union(Self::TAP_SENS),
        }
    }
}

cfg_register! {
    TapConfig1: 0x58 = 0x06 {
        const QUIET_DT1 = 0b0010_0000;
        const QUIET_DT0 = 0b0001_0000;
        const QUIET1    = 0b0000_1000;
        const QUIET0    = 0b0000_0100;
        const TICS_TH1  = 0b0000_0010;
        const TICS_TH0  = 0b0000_0001;

        const QUIET_DT = Self::QUIET_DT1.bits | Self::QUIET_DT0.bits;
        const QUIET = Self::QUIET1.bits | Self::QUIET0.bits;
        const TICS_TH = Self::TICS_TH1.bits | Self::TICS_TH0.bits;
    }
}

impl TapConfig1 {
    pub const fn with_min_tap_duration(self, min_duration: MinTapDuration) -> Self {
        match min_duration {
            MinTapDuration::Samples4 => self.difference(Self::QUIET_DT),
            MinTapDuration::Samples8 => self.difference(Self::QUIET_DT).union(Self::QUIET_DT0),
            MinTapDuration::Samples12 => self.difference(Self::QUIET_DT).union(Self::QUIET_DT1),
            MinTapDuration::Samples16 => self.union(Self::QUIET_DT),
        }
    }
    pub const fn with_double_tap_duration(self, max_duration: DoubleTapDuration) -> Self {
        match max_duration {
            DoubleTapDuration::Samples60 => self.difference(Self::QUIET),
            DoubleTapDuration::Samples80 => self.difference(Self::QUIET).union(Self::QUIET0),
            DoubleTapDuration::Samples100 => self.difference(Self::QUIET).union(Self::QUIET1),
            DoubleTapDuration::Samples120 => self.union(Self::QUIET),
        }
    }
    pub const fn with_max_tap_duration(self, max_duration: MaxTapDuration) -> Self {
        match max_duration {
            MaxTapDuration::Samples6 => self.difference(Self::TICS_TH),
            MaxTapDuration::Samples9 => self.difference(Self::TICS_TH).union(Self::TICS_TH0),
            MaxTapDuration::Samples12 => self.difference(Self::TICS_TH).union(Self::TICS_TH1),
            MaxTapDuration::Samples18 => self.union(Self::TICS_TH),
        }
    }
}

#[cfg(any(feature = "spi", test))]
cfg_register! {
    InterfaceConfig: 0x7C = 0x00 {
        const SPI3 = 0b0000_0001;
    }
}

#[cfg(any(feature = "spi", test))]
impl InterfaceConfig {
    pub const fn with_spi_3wire_mode(self, enabled: bool) -> Self {
        if enabled {
            self.union(Self::SPI3)
        } else {
            self.difference(Self::SPI3)
        }
    }
}

cfg_register! {
    SelfTest: 0x7D = 0x00 {
        const TEST_SIGN = 0b0000_1000;
        const TEST_Z_EN = 0b0000_0100;
        const TEST_Y_EN = 0b0000_0010;
        const TEST_X_EN = 0b0000_0001;
    }
}

pub enum Command {
    FlushFifo,
    ClearStepCount,
    SoftReset,
}

impl ReadReg for Command {
    const ADDR: u8 = 0x7E;
}

impl ConfigReg for Command {
    fn to_byte(&self) -> u8 {
        match self {
            Command::FlushFifo => 0xB0,
            Command::ClearStepCount => 0xB1,
            Command::SoftReset => 0xB6,
        }
    }
}
