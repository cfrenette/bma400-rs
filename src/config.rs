use core::fmt::Debug;
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

// Re-export builders
pub use accel_config::AccConfigBuilder;
pub use int_config::IntConfigBuilder;
pub use int_pin_config::IntPinConfigBuilder;
pub use fifo_config::FifoConfigBuilder;
pub use auto_lp_config::AutoLpConfigBuilder;
pub use auto_wkup_config::AutoWakeupConfigBuilder;
pub use wkup_int_config::WakeupIntConfigBuilder;

mod orientch_config;
use orientch_config::OrientChgConfig;
pub use orientch_config::OrientChgConfigBuilder;
// TODO Generic Interrupts
// mod gen_int_config;
mod actchg_config;
use actchg_config::ActChgConfig;
pub use actchg_config::ActChgConfigBuilder;
mod tap_config;
use tap_config::TapConfig;
pub use tap_config::TapConfigBuilder;

use crate::{
    Scale, 
    registers::{
        IntConfig0, IntConfig1, 
        AccConfig1,
    }, 
    BMA400Error, interface::WriteToRegister
};

#[derive(Default, Clone)]
pub(crate) struct Config {
    acc_config: AccConfig,
    int_config: IntConfig,
    int_pin_config: IntPinConfig,
    fifo_config: FifoConfig,
    auto_lp_config: AutoLpConfig,
    auto_wkup_config: AutoWakeupConfig,
    wkup_int_config: WakeupIntConfig,

    // Maybe #[cfg(feature = "adv-int-orientchg")]
    orientch_config: OrientChgConfig,
    
    // Maybe #[cfg(feature = "adv-int-generic")]
    /* TODO
    gen1int_config: GenIntConfig,
    gen2int_config: GenIntConfig,
    */

    // Maybe #[cfg(feature = "adv-int-actchg")]
    actchg_config: ActChgConfig,

    // Maybe #[cfg(feature = "adv-int-tap")]
    tap_config: TapConfig,

    /* TODO
    #[cfg(feature = "spi")]
    if_conf: InterfaceConfig,
    */
}

impl Config {
    pub fn scale(&self) -> Scale {
        self.acc_config.scale()
    }
    pub fn is_fifo_read_disabled(&self) -> bool {
        self.fifo_config.is_read_disabled()
    }
    pub fn setup_self_test<Interface: WriteToRegister<Error = BMA400Error<InterfaceError, PinError>>, InterfaceError: Debug, PinError: Debug>(&self, interface: &mut Interface) -> Result<(), BMA400Error<InterfaceError, PinError>> {
        // Disable Interrupts
        interface.write_register(IntConfig0::from_bits_truncate(0x00))?;
        interface.write_register(IntConfig1::from_bits_truncate(0x00))?;
        interface.write_register(self.auto_wkup_config.get_config1().clone().with_wakeup_int(false))?;
        // Disable FIFO
        interface.write_register(self.fifo_config.get_config0().clone().with_fifo_x(false).with_fifo_y(false).with_fifo_z(false))?;

        // Set PowerMode = Normal
        interface.write_register(self.acc_config.get_config0().clone().with_power_mode(crate::PowerMode::Normal))?;
        // Set Range = 4G, OSR = OSR3, ODR = 100Hz
        interface.write_register(AccConfig1::from_bits_truncate(0x78))?;
        Ok(())
    }
    pub fn cleanup_self_test<Interface: WriteToRegister<Error = BMA400Error<InterfaceError, PinError>>, InterfaceError: Debug, PinError: Debug>(&self, interface: &mut Interface) -> Result<(), BMA400Error<InterfaceError, PinError>> {
        // Restore AccConfig
        interface.write_register(self.acc_config.get_config0())?;
        interface.write_register(self.acc_config.get_config1())?;
        // Restore IntConfig
        interface.write_register(self.int_config.get_config0())?;
        interface.write_register(self.int_config.get_config1())?;
        interface.write_register(self.auto_wkup_config.get_config1())?;
        // Restore FifoConfig
        interface.write_register(self.fifo_config.get_config0())?;
        Ok(())
    }
}
