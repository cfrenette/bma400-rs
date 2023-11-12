use crate::{
    interface::WriteToRegister,
    registers::{IntConfig0, IntConfig1},
    ConfigError, DataSource, OutputDataRate, BMA400,
};

#[cfg(feature = "async")]
use crate::{interface::AsyncWriteToRegister, AsyncBMA400};

#[derive(Clone, Default)]
pub struct IntConfig {
    int_config0: IntConfig0,
    int_config1: IntConfig1,
}

impl IntConfig {
    // API to quickly enable / disable interrupts for config changes
    pub fn get_config0(&self) -> IntConfig0 {
        self.int_config0
    }
    pub fn get_config1(&self) -> IntConfig1 {
        self.int_config1
    }
}

/// Enable or disable interrupts[^except] and set interrupt latch mode
///
/// [^except]: To enable the Auto-Wakeup Interrupt see [`config_autowkup()`](BMA400::config_autowkup)
pub struct IntConfigBuilder<Device> {
    config: IntConfig,
    device: Device,
}

impl<Device> IntConfigBuilder<Device> {
    // IntConfig0
    /// Enable/Disable the Data Ready Interrupt
    pub fn with_dta_rdy_int(mut self, enabled: bool) -> Self {
        self.config.int_config0 = self.config.int_config0.with_dta_rdy_int(enabled);
        self
    }
    /// Enable/Disable the Fifo Watermark Interrupt
    pub fn with_fwm_int(mut self, enabled: bool) -> Self {
        self.config.int_config0 = self.config.int_config0.with_fwm_int(enabled);
        self
    }
    /// Enable/Disable the Fifo Full Interrupt
    pub fn with_ffull_int(mut self, enabled: bool) -> Self {
        self.config.int_config0 = self.config.int_config0.with_ffull_int(enabled);
        self
    }
    /// Enable/Disable Generic Interrupt 2
    pub fn with_gen2_int(mut self, enabled: bool) -> Self {
        self.config.int_config0 = self.config.int_config0.with_gen2_int(enabled);
        self
    }
    /// Enable/Disable Generic Interrupt 1
    pub fn with_gen1_int(mut self, enabled: bool) -> Self {
        self.config.int_config0 = self.config.int_config0.with_gen1_int(enabled);
        self
    }
    /// Enable/Disable the Orientation Change Interrupt
    pub fn with_orientch_int(mut self, enabled: bool) -> Self {
        self.config.int_config0 = self.config.int_config0.with_orientch_int(enabled);
        self
    }

    // IntConfig1

    /// Enable/Disable latched interrupt mode
    ///
    /// When enabled, all interrupts persist until the corresponding IntStatus is read
    pub fn with_latch_int(mut self, enabled: bool) -> Self {
        self.config.int_config1 = self.config.int_config1.with_latch_int(enabled);
        self
    }
    /// Enable/Disable Activity Changed Interrupt
    pub fn with_actch_int(mut self, enabled: bool) -> Self {
        self.config.int_config1 = self.config.int_config1.with_actch_int(enabled);
        self
    }
    /// Enable/Disable Double Tap Interrupt
    pub fn with_d_tap_int(mut self, enabled: bool) -> Self {
        self.config.int_config1 = self.config.int_config1.with_d_tap_int(enabled);
        self
    }
    /// Enable/Disable Single Tap Interrupt
    pub fn with_s_tap_int(mut self, enabled: bool) -> Self {
        self.config.int_config1 = self.config.int_config1.with_s_tap_int(enabled);
        self
    }
    /// Enable/Disable Step Interrupt
    pub fn with_step_int(mut self, enabled: bool) -> Self {
        self.config.int_config1 = self.config.int_config1.with_step_int(enabled);
        self
    }
}

impl<'a, Interface, E> IntConfigBuilder<&'a mut BMA400<Interface>>
where
    Interface: WriteToRegister<Error = E>,
    E: From<ConfigError>,
{
    pub(crate) fn new(device: &'a mut BMA400<Interface>) -> Self {
        IntConfigBuilder {
            config: device.config.int_config.clone(),
            device,
        }
    }
    /// Write this configuration to device registers
    pub fn write(self) -> Result<(), E> {
        if (self.config.int_config1.d_tap_int() || self.config.int_config1.s_tap_int())
            && !matches!(self.device.config.acc_config.odr(), OutputDataRate::Hz200)
        {
            return Err(ConfigError::TapIntEnabledInvalidODR.into());
        }

        // Check DataSource for each enabled interrupt that can use Filt1 and validate

        // Gen 1
        if self.config.int_config0.gen1_int()
            && !matches!(self.device.config.acc_config.odr(), OutputDataRate::Hz100)
            && matches!(
                self.device.config.gen1int_config.src(),
                DataSource::AccFilt1
            )
        {
            return Err(ConfigError::Filt1InterruptInvalidODR.into());
        }
        // Gen 2
        if self.config.int_config0.gen2_int()
            && !matches!(self.device.config.acc_config.odr(), OutputDataRate::Hz100)
            && matches!(
                self.device.config.gen2int_config.src(),
                DataSource::AccFilt1
            )
        {
            return Err(ConfigError::Filt1InterruptInvalidODR.into());
        }
        // Activity Change
        if self.config.int_config1.actch_int()
            && !matches!(self.device.config.acc_config.odr(), OutputDataRate::Hz100)
            && matches!(self.device.config.actchg_config.src(), DataSource::AccFilt1)
        {
            return Err(ConfigError::Filt1InterruptInvalidODR.into());
        }

        if self.device.config.int_config.int_config0.bits() != self.config.int_config0.bits() {
            self.device
                .interface
                .write_register(self.config.int_config0)?;
            self.device.config.int_config.int_config0 = self.config.int_config0;
        }
        if self.device.config.int_config.int_config1.bits() != self.config.int_config1.bits() {
            self.device
                .interface
                .write_register(self.config.int_config1)?;
            self.device.config.int_config.int_config1 = self.config.int_config1;
        }
        Ok(())
    }
}

#[cfg(feature = "async")]
#[cfg_attr(docsrs, doc(cfg(feature = "async")))]
impl<'a, Interface, E> IntConfigBuilder<&'a mut AsyncBMA400<Interface>>
where
    Interface: AsyncWriteToRegister<Error = E>,
    E: From<ConfigError>,
{
    pub(crate) fn new_async(device: &'a mut AsyncBMA400<Interface>) -> Self {
        IntConfigBuilder {
            config: device.config.int_config.clone(),
            device,
        }
    }
    /// Write this configuration to device registers
    pub async fn write(self) -> Result<(), E> {
        if (self.config.int_config1.d_tap_int() || self.config.int_config1.s_tap_int())
            && !matches!(self.device.config.acc_config.odr(), OutputDataRate::Hz200)
        {
            return Err(ConfigError::TapIntEnabledInvalidODR.into());
        }

        // Check DataSource for each enabled interrupt that can use Filt1 and validate

        // Gen 1
        if self.config.int_config0.gen1_int()
            && !matches!(self.device.config.acc_config.odr(), OutputDataRate::Hz100)
            && matches!(
                self.device.config.gen1int_config.src(),
                DataSource::AccFilt1
            )
        {
            return Err(ConfigError::Filt1InterruptInvalidODR.into());
        }
        // Gen 2
        if self.config.int_config0.gen2_int()
            && !matches!(self.device.config.acc_config.odr(), OutputDataRate::Hz100)
            && matches!(
                self.device.config.gen2int_config.src(),
                DataSource::AccFilt1
            )
        {
            return Err(ConfigError::Filt1InterruptInvalidODR.into());
        }
        // Activity Change
        if self.config.int_config1.actch_int()
            && !matches!(self.device.config.acc_config.odr(), OutputDataRate::Hz100)
            && matches!(self.device.config.actchg_config.src(), DataSource::AccFilt1)
        {
            return Err(ConfigError::Filt1InterruptInvalidODR.into());
        }

        if self.device.config.int_config.int_config0.bits() != self.config.int_config0.bits() {
            self.device
                .interface
                .write_register(self.config.int_config0)
                .await?;
            self.device.config.int_config.int_config0 = self.config.int_config0;
        }
        if self.device.config.int_config.int_config1.bits() != self.config.int_config1.bits() {
            self.device
                .interface
                .write_register(self.config.int_config1)
                .await?;
            self.device.config.int_config.int_config1 = self.config.int_config1;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{tests::get_test_device, BMA400Error};
    #[test]
    fn test_dta_rdy() {
        let mut device = get_test_device();
        let builder = device.config_interrupts();
        let builder = builder.with_dta_rdy_int(true);
        assert_eq!(builder.config.int_config0.bits(), 0x80);
    }
    #[test]
    fn test_fwm() {
        let mut device = get_test_device();
        let builder = device.config_interrupts();
        let builder = builder.with_fwm_int(true);
        assert_eq!(builder.config.int_config0.bits(), 0x40);
    }
    #[test]
    fn test_ffull() {
        let mut device = get_test_device();
        let builder = device.config_interrupts();
        let builder = builder.with_ffull_int(true);
        assert_eq!(builder.config.int_config0.bits(), 0x20);
    }
    #[test]
    fn test_gen2() {
        let mut device = get_test_device();
        let builder = device.config_interrupts();
        let builder = builder.with_gen2_int(true);
        assert_eq!(builder.config.int_config0.bits(), 0x08);
    }
    #[test]
    fn test_gen1() {
        let mut device = get_test_device();
        let builder = device.config_interrupts();
        let builder = builder.with_gen1_int(true);
        assert_eq!(builder.config.int_config0.bits(), 0x04);
    }
    #[test]
    fn test_orientch() {
        let mut device = get_test_device();
        let builder = device.config_interrupts();
        let builder = builder.with_orientch_int(true);
        assert_eq!(builder.config.int_config0.bits(), 0x02);
    }
    #[test]
    fn test_latch() {
        let mut device = get_test_device();
        let builder = device.config_interrupts();
        let builder = builder.with_latch_int(true);
        assert_eq!(builder.config.int_config1.bits(), 0x80);
    }
    #[test]
    fn test_actch() {
        let mut device = get_test_device();
        let builder = device.config_interrupts();
        let builder = builder.with_actch_int(true);
        assert_eq!(builder.config.int_config1.bits(), 0x10);
    }
    #[test]
    fn test_dtap() {
        let mut device = get_test_device();
        let builder = device.config_interrupts();
        let builder = builder.with_d_tap_int(true);
        assert_eq!(builder.config.int_config1.bits(), 0x08);
    }
    #[test]
    fn test_stap() {
        let mut device = get_test_device();
        let builder = device.config_interrupts();
        let builder = builder.with_s_tap_int(true);
        assert_eq!(builder.config.int_config1.bits(), 0x04);
    }
    #[test]
    fn test_step() {
        let mut device = get_test_device();
        let builder = device.config_interrupts();
        let builder = builder.with_step_int(true);
        assert_eq!(builder.config.int_config1.bits(), 0x01);
    }
    #[test]
    fn test_tap_int_config_err() {
        let mut device = get_test_device();
        // Set the output data rate to 100Hz
        assert!(matches!(
            device
                .config_accel()
                .with_odr(OutputDataRate::Hz100)
                .write(),
            Ok(())
        ));
        // Try to enable the single tap interrupt
        let result = device.config_interrupts().with_s_tap_int(true).write();
        assert!(matches!(
            result,
            Err(BMA400Error::ConfigBuildError(
                ConfigError::TapIntEnabledInvalidODR
            ))
        ));
        // Try to enable the double tap interrupt
        let result = device.config_interrupts().with_d_tap_int(true).write();
        assert!(matches!(
            result,
            Err(BMA400Error::ConfigBuildError(
                ConfigError::TapIntEnabledInvalidODR
            ))
        ));
    }
    #[test]
    fn test_gen1_int_config_err() {
        let mut device = get_test_device();
        // By default Generic Interrupts are set to use Filt1 and default ODR is 200Hz, so no need
        // to set manually Try to enable the gen1 interrupt
        let result = device.config_interrupts().with_gen1_int(true).write();
        assert!(matches!(
            result,
            Err(BMA400Error::ConfigBuildError(
                ConfigError::Filt1InterruptInvalidODR
            ))
        ));
    }
    #[test]
    fn test_gen2_int_config_err() {
        let mut device = get_test_device();
        // By default Generic Interrupts are set to use Filt1 and default ODR is 200Hz, so no need
        // to set manually Try to enable the gen1 interrupt
        let result = device.config_interrupts().with_gen2_int(true).write();
        assert!(matches!(
            result,
            Err(BMA400Error::ConfigBuildError(
                ConfigError::Filt1InterruptInvalidODR
            ))
        ));
    }
    #[test]
    fn test_actch_config_err() {
        let mut device = get_test_device();
        // By default Activity Change is set to use Filt1 and default ODR is 200Hz, so no need to
        // set manually Try to enable the activity change interrupt
        let result = device.config_interrupts().with_actch_int(true).write();
        assert!(matches!(
            result,
            Err(BMA400Error::ConfigBuildError(
                ConfigError::Filt1InterruptInvalidODR
            ))
        ));
    }
}
