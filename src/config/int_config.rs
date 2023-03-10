use crate::{
    registers::{
        IntConfig0, IntConfig1
    }, 
    interface::WriteToRegister,
    ConfigError,
    BMA400,
    OutputDataRate,
    DataSource,
};


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

pub struct IntConfigBuilder<'a, Interface: WriteToRegister> 
{
    config: IntConfig,
    device: &'a mut BMA400<Interface>,
}

impl<'a, Interface, E> IntConfigBuilder<'a, Interface> 
where
    Interface: WriteToRegister<Error = E>,
    E: From<ConfigError>,
{
    pub(crate) fn new(device: &'a mut BMA400<Interface>) -> IntConfigBuilder<'a, Interface> {
        IntConfigBuilder { config: device.config.int_config.clone(), device }
    }
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
    pub fn write(self) -> Result<(), E> {
        if (self.config.int_config1.d_tap_int() || self.config.int_config1.s_tap_int()) && !matches!(self.device.config.acc_config.odr(), OutputDataRate::Hz200) {
            return Err(ConfigError::TapIntEnabledInvalidODR.into());
        }
        
        // Check DataSource for each enabled interrupt that can use Filt1 and validate

        // Gen 1
        if self.config.int_config0.gen1_int() && !matches!(self.device.config.acc_config.odr(), OutputDataRate::Hz100) && matches!(self.device.config.gen1int_config.src(), DataSource::AccFilt1) {
            return Err(ConfigError::Filt1InterruptInvalidODR.into());
        }
        // Gen 2
        if self.config.int_config0.gen2_int() && !matches!(self.device.config.acc_config.odr(), OutputDataRate::Hz100) && matches!(self.device.config.gen2int_config.src(), DataSource::AccFilt1) {
            return Err(ConfigError::Filt1InterruptInvalidODR.into());
        }
        // Activity Change
        if self.config.int_config1.actch_int() && !matches!(self.device.config.acc_config.odr(), OutputDataRate::Hz100) && matches!(self.device.config.actchg_config.src(), DataSource::AccFilt1) {
            return Err(ConfigError::Filt1InterruptInvalidODR.into());
        }

        if self.device.config.int_config.int_config0.bits() != self.config.int_config0.bits() {
            self.device.interface.write_register(self.config.int_config0)?;
            self.device.config.int_config.int_config0 = self.config.int_config0;
        }
        if self.device.config.int_config.int_config1.bits() != self.config.int_config1.bits() {
            self.device.interface.write_register(self.config.int_config1)?;
            self.device.config.int_config.int_config1 = self.config.int_config1;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use embedded_hal_mock::i2c::{Mock, Transaction};
    use crate::{
        BMA400Error,
        i2c::I2CInterface,
    };
    const ADDR: u8 = crate::i2c::ADDR;
    fn device_no_write() -> BMA400<I2CInterface<Mock>> {
        let expected = [
            Transaction::write_read(ADDR, [0x00].into_iter().collect(), [0x90].into_iter().collect())
        ];
        BMA400::new_i2c(Mock::new(&expected)).unwrap()
    }
    fn device_write(expected: &[Transaction]) -> BMA400<I2CInterface<Mock>> {
        BMA400::new_i2c(Mock::new(expected)).unwrap()
    }
    #[test]
    fn test_dta_rdy() {
        let mut device = device_no_write();
        let builder = device.config_interrupts();
        let builder = builder.with_dta_rdy_int(true);
        assert_eq!(builder.config.int_config0.bits(), 0x80);
    }
    #[test]
    fn test_fwm() {
        let mut device = device_no_write();
        let builder = device.config_interrupts();
        let builder = builder.with_fwm_int(true);
        assert_eq!(builder.config.int_config0.bits(), 0x40);
    }
    #[test]
    fn test_ffull() {
        let mut device = device_no_write();
        let builder = device.config_interrupts();
        let builder = builder.with_ffull_int(true);
        assert_eq!(builder.config.int_config0.bits(), 0x20);
    }
    #[test]
    fn test_gen2() {
        let mut device = device_no_write();
        let builder = device.config_interrupts();
        let builder = builder.with_gen2_int(true);
        assert_eq!(builder.config.int_config0.bits(), 0x08);
    }
    #[test]
    fn test_gen1() {
        let mut device = device_no_write();
        let builder = device.config_interrupts();
        let builder = builder.with_gen1_int(true);
        assert_eq!(builder.config.int_config0.bits(), 0x04);
    }
    #[test]
    fn test_orientch() {
        let mut device = device_no_write();
        let builder = device.config_interrupts();
        let builder = builder.with_orientch_int(true);
        assert_eq!(builder.config.int_config0.bits(), 0x02);
    }
    #[test]
    fn test_latch() {
        let mut device = device_no_write();
        let builder = device.config_interrupts();
        let builder = builder.with_latch_int(true);
        assert_eq!(builder.config.int_config1.bits(), 0x80);
    }
    #[test]
    fn test_actch() {
        let mut device = device_no_write();
        let builder = device.config_interrupts();
        let builder = builder.with_actch_int(true);
        assert_eq!(builder.config.int_config1.bits(), 0x10);
    }
    #[test]
    fn test_dtap() {
        let mut device = device_no_write();
        let builder = device.config_interrupts();
        let builder = builder.with_d_tap_int(true);
        assert_eq!(builder.config.int_config1.bits(), 0x08);
    }
    #[test]
    fn test_stap() {
        let mut device = device_no_write();
        let builder = device.config_interrupts();
        let builder = builder.with_s_tap_int(true);
        assert_eq!(builder.config.int_config1.bits(), 0x04);
    }
    #[test]
    fn test_step() {
        let mut device = device_no_write();
        let builder = device.config_interrupts();
        let builder = builder.with_step_int(true);
        assert_eq!(builder.config.int_config1.bits(), 0x01);
    }
    #[test]
    fn test_tap_int_config_err() {
        let expected = [
            Transaction::write_read(ADDR, [0x00].into_iter().collect(), [0x90].into_iter().collect()),
            // Set the Output Data Rate to 100Hz
            Transaction::write(ADDR, [0x1A, 0x48].into_iter().collect()),
        ];
        let mut device = device_write(&expected);
        // Set the output data rate to 100Hz
        assert!(matches!(device.config_accel().with_odr(OutputDataRate::Hz100).write(), Ok(())));
        // Try to enable the single tap interrupt
        let result = device.config_interrupts().with_s_tap_int(true).write();
        assert!(matches!(result, Err(BMA400Error::ConfigBuildError(ConfigError::TapIntEnabledInvalidODR))));
        // Try to enable the double tap interrupt
        let result = device.config_interrupts().with_d_tap_int(true).write();
        assert!(matches!(result, Err(BMA400Error::ConfigBuildError(ConfigError::TapIntEnabledInvalidODR))));
    }
    #[test]
    fn test_gen1_int_config_err() {
        let mut device = device_no_write();
        // By default Generic Interrupts are set to use Filt1 and default ODR is 200Hz, so no need to set manually
        // Try to enable the gen1 interrupt
        let result = device.config_interrupts().with_gen1_int(true).write();
        assert!(matches!(result, Err(BMA400Error::ConfigBuildError(ConfigError::Filt1InterruptInvalidODR))));
    }
    #[test]
    fn test_gen2_int_config_err() {
        let mut device = device_no_write();
        // By default Generic Interrupts are set to use Filt1 and default ODR is 200Hz, so no need to set manually
        // Try to enable the gen1 interrupt
        let result = device.config_interrupts().with_gen2_int(true).write();
        assert!(matches!(result, Err(BMA400Error::ConfigBuildError(ConfigError::Filt1InterruptInvalidODR))));
    }
    #[test]
    fn test_actch_config_err() {
        let mut device = device_no_write();
        // By default Activity Change is set to use Filt1 and default ODR is 200Hz, so no need to set manually
        // Try to enable the activity change interrupt
        let result = device.config_interrupts().with_actch_int(true).write();
        assert!(matches!(result, Err(BMA400Error::ConfigBuildError(ConfigError::Filt1InterruptInvalidODR))));
    }
}
