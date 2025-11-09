//! Accelerometer configuration options
mod accel_config;
use accel_config::AccConfig;
mod int_config;
use int_config::IntConfig;
mod int_pin_config;
use int_pin_config::IntPinConfig;
mod fifo_config;
use fifo_config::FifoConfig;
mod auto_lp_config;
use auto_lp_config::AutoLpConfig;
mod auto_wkup_config;
use auto_wkup_config::AutoWakeupConfig;
mod wkup_int_config;
use wkup_int_config::WakeupIntConfig;
mod actchg_config;
use actchg_config::ActChgConfig;
mod tap_config;
use tap_config::TapConfig;
mod orientch_config;
use orientch_config::OrientChgConfig;

// Re-export builders
pub use accel_config::AccConfigBuilder;
pub use actchg_config::ActChgConfigBuilder;
pub use auto_lp_config::AutoLpConfigBuilder;
pub use auto_wkup_config::AutoWakeupConfigBuilder;
pub use fifo_config::FifoConfigBuilder;
pub use gen_int_config::GenIntConfigBuilder;
pub use int_config::IntConfigBuilder;
pub use int_pin_config::IntPinConfigBuilder;
pub use orientch_config::OrientChgConfigBuilder;
pub use tap_config::TapConfigBuilder;
pub use wkup_int_config::WakeupIntConfigBuilder;

mod gen_int_config;
use gen_int_config::{Gen1IntConfig, Gen2IntConfig};

use crate::Scale;

#[derive(Default, Clone)]
pub(crate) struct Config {
    acc_config: AccConfig,
    int_config: IntConfig,
    int_pin_config: IntPinConfig,
    fifo_config: FifoConfig,
    auto_lp_config: AutoLpConfig,
    auto_wkup_config: AutoWakeupConfig,
    wkup_int_config: WakeupIntConfig,
    orientch_config: OrientChgConfig,
    gen1int_config: Gen1IntConfig,
    gen2int_config: Gen2IntConfig,
    actchg_config: ActChgConfig,
    tap_config: TapConfig,
}

impl Config {
    pub fn scale(&self) -> Scale {
        self.acc_config.scale()
    }
    pub fn is_fifo_read_disabled(&self) -> bool {
        self.fifo_config.is_read_disabled()
    }
    pub fn acc_config(&self) -> &AccConfig {
        &self.acc_config
    }
    pub fn auto_wkup_config(&self) -> &AutoWakeupConfig {
        &self.auto_wkup_config
    }
    pub fn fifo_config(&self) -> &FifoConfig {
        &self.fifo_config
    }
    pub fn int_config(&self) -> &IntConfig {
        &self.int_config
    }
}
