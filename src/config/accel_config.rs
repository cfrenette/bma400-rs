use crate::{
    BMA400, ConfigError, DataSource, Filter1Bandwidth, OutputDataRate, OversampleRate, PowerMode,
    Scale,
    registers::{AccConfig0, AccConfig1, AccConfig2},
};

#[derive(Clone, Default)]
pub struct AccConfig {
    acc_config0: AccConfig0,
    acc_config1: AccConfig1,
    acc_config2: AccConfig2,
}

impl AccConfig {
    pub fn odr(&self) -> OutputDataRate {
        self.acc_config1.odr()
    }
    pub fn scale(&self) -> Scale {
        self.acc_config1.scale()
    }
    pub fn get_config0(&self) -> AccConfig0 {
        self.acc_config0
    }
    pub fn get_config1(&self) -> AccConfig1 {
        self.acc_config1
    }
}

/// Configure how the accelerometer samples, filters and ouputs data
///
/// - [PowerMode] using [`with_power_mode()`](AccConfigBuilder::with_power_mode)
/// - [DataSource] for [`get_data()`](BMA400::get_data) and [`get_unscaled_data()`](BMA400::get_unscaled_data) using [`with_reg_dta_src()`](AccConfigBuilder::with_reg_dta_src)
/// - [OversampleRate] for low power and normal modes using [`with_osr_lp()`](AccConfigBuilder::with_osr_lp) and [`with_osr()`](AccConfigBuilder::with_osr) respectively
/// - [Filter1Bandwidth] using [`with_filt1_bw()`](AccConfigBuilder::with_filt1_bw)
/// - [OutputDataRate] using [`with_odr()`](AccConfigBuilder::with_odr)
/// - [Scale] using [`with_scale()`](AccConfigBuilder::with_scale)
pub struct AccConfigBuilder<'a, Interface> {
    config: AccConfig,
    device: &'a mut BMA400<Interface>,
}

#[cfg(not(feature = "embedded-hal-async"))]
impl<'a, Interface, E> AccConfigBuilder<'a, Interface>
where
    Interface: crate::blocking::WriteToRegister<Error = E>,
    E: From<ConfigError>,
{
    /// Write this configuration to device registers
    pub fn write(self) -> Result<(), E> {
        let int_config0 = self.device.config.int_config.get_config0();
        let int_config1 = self.device.config.int_config.get_config1();

        // If Gen Int 1 / 2 or Activity Change use AccFilt1 and are enabled, ODR must be 100Hz
        let mut filt1_used_for_ints = false;
        if int_config1.actch_int()
            && matches!(self.device.config.actchg_config.src(), DataSource::AccFilt1)
        {
            filt1_used_for_ints = true;
        }
        if int_config0.gen1_int()
            && matches!(
                self.device.config.gen1int_config.src(),
                DataSource::AccFilt1
            )
        {
            filt1_used_for_ints = true;
        }
        if int_config0.gen2_int()
            && matches!(
                self.device.config.gen2int_config.src(),
                DataSource::AccFilt1
            )
        {
            filt1_used_for_ints = true;
        }
        if filt1_used_for_ints && !matches!(self.config.odr(), OutputDataRate::Hz100) {
            return Err(ConfigError::Filt1InterruptInvalidODR.into());
        }
        // If either Tap Interrupt is enabled, filt1 ODR must be set to 200Hz
        if (int_config1.d_tap_int() || int_config1.s_tap_int())
            && !matches!(self.config.odr(), OutputDataRate::Hz200)
        {
            return Err(ConfigError::TapIntEnabledInvalidODR.into());
        }
        if self.device.config.acc_config.acc_config0.bits() != self.config.acc_config0.bits() {
            self.device
                .interface
                .write_register(self.config.acc_config0)?;
            self.device.config.acc_config.acc_config0 = self.config.acc_config0;
        }
        if self.device.config.acc_config.acc_config1.bits() != self.config.acc_config1.bits() {
            self.device
                .interface
                .write_register(self.config.acc_config1)?;
            self.device.config.acc_config.acc_config1 = self.config.acc_config1;
        }
        if self.device.config.acc_config.acc_config2.bits() != self.config.acc_config2.bits() {
            self.device
                .interface
                .write_register(self.config.acc_config2)?;
            self.device.config.acc_config.acc_config2 = self.config.acc_config2;
        }
        Ok(())
    }
}

#[cfg(feature = "embedded-hal-async")]
impl<'a, Interface, E> AccConfigBuilder<'a, Interface>
where
    Interface: crate::asynch::WriteToRegister<Error = E>,
    E: From<ConfigError>,
{
    /// Write this configuration to device registers
    pub async fn write(self) -> Result<(), E> {
        let int_config0 = self.device.config.int_config.get_config0();
        let int_config1 = self.device.config.int_config.get_config1();

        // If Gen Int 1 / 2 or Activity Change use AccFilt1 and are enabled, ODR must be 100Hz
        let mut filt1_used_for_ints = false;
        if int_config1.actch_int()
            && matches!(self.device.config.actchg_config.src(), DataSource::AccFilt1)
        {
            filt1_used_for_ints = true;
        }
        if int_config0.gen1_int()
            && matches!(
                self.device.config.gen1int_config.src(),
                DataSource::AccFilt1
            )
        {
            filt1_used_for_ints = true;
        }
        if int_config0.gen2_int()
            && matches!(
                self.device.config.gen2int_config.src(),
                DataSource::AccFilt1
            )
        {
            filt1_used_for_ints = true;
        }
        if filt1_used_for_ints && !matches!(self.config.odr(), OutputDataRate::Hz100) {
            return Err(ConfigError::Filt1InterruptInvalidODR.into());
        }
        // If either Tap Interrupt is enabled, filt1 ODR must be set to 200Hz
        if (int_config1.d_tap_int() || int_config1.s_tap_int())
            && !matches!(self.config.odr(), OutputDataRate::Hz200)
        {
            return Err(ConfigError::TapIntEnabledInvalidODR.into());
        }
        if self.device.config.acc_config.acc_config0.bits() != self.config.acc_config0.bits() {
            self.device
                .interface
                .write_register(self.config.acc_config0)
                .await?;
            self.device.config.acc_config.acc_config0 = self.config.acc_config0;
        }
        if self.device.config.acc_config.acc_config1.bits() != self.config.acc_config1.bits() {
            self.device
                .interface
                .write_register(self.config.acc_config1)
                .await?;
            self.device.config.acc_config.acc_config1 = self.config.acc_config1;
        }
        if self.device.config.acc_config.acc_config2.bits() != self.config.acc_config2.bits() {
            self.device
                .interface
                .write_register(self.config.acc_config2)
                .await?;
            self.device.config.acc_config.acc_config2 = self.config.acc_config2;
        }
        Ok(())
    }
}

impl<'a, Interface> AccConfigBuilder<'a, Interface> {
    pub(crate) fn new(device: &'a mut BMA400<Interface>) -> AccConfigBuilder<'a, Interface> {
        AccConfigBuilder {
            config: device.config.acc_config.clone(),
            device,
        }
    }
    // AccConfig0
    /// Set [PowerMode]
    ///
    /// Other settings can result in the power changing automatically,
    /// for example auto wakeup and auto low-power mode.
    /// To read the current power mode from the sensor use [`get_status()`](BMA400::get_status)
    pub fn with_power_mode(mut self, power_mode: PowerMode) -> Self {
        self.config.acc_config0 = self.config.acc_config0.with_power_mode(power_mode);
        self
    }
    /// Set the [OversampleRate] used in [`PowerMode::LowPower`] mode
    pub fn with_osr_lp(mut self, osr: OversampleRate) -> Self {
        self.config.acc_config0 = self.config.acc_config0.with_osr_lp(osr);
        self
    }
    /// Set the [Filter1Bandwidth] for [`DataSource::AccFilt1`]
    pub fn with_filt1_bw(mut self, bandwidth: Filter1Bandwidth) -> Self {
        self.config.acc_config0 = self.config.acc_config0.with_filt1_bw(bandwidth);
        self
    }
    // AccConfig1
    /// Output Data Rate for [`DataSource::AccFilt1`]
    pub fn with_odr(mut self, odr: OutputDataRate) -> Self {
        self.config.acc_config1 = self.config.acc_config1.with_odr(odr);
        self
    }
    /// Set the [OversampleRate] used in [PowerMode::Normal] power mode
    pub fn with_osr(mut self, osr: OversampleRate) -> Self {
        self.config.acc_config1 = self.config.acc_config1.with_osr(osr);
        self
    }
    /// Set the [Scale] (resolution) of the data being output
    pub fn with_scale(mut self, scale: Scale) -> Self {
        self.config.acc_config1 = self.config.acc_config1.with_scale(scale);
        self
    }
    // AccConfig2
    /// Set the [DataSource] feeding the single read registers
    pub fn with_reg_dta_src(mut self, src: DataSource) -> Self {
        self.config.acc_config2 = self.config.acc_config2.with_dta_reg_src(src);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BMA400Error, tests::get_test_device};
    #[test]
    fn test_power_mode() {
        let mut device = get_test_device();
        let builder = device.config_accel();
        let builder = builder.with_power_mode(PowerMode::Sleep);
        assert_eq!(builder.config.acc_config0.bits(), 0x00);
        let builder = builder.with_power_mode(PowerMode::LowPower);
        assert_eq!(builder.config.acc_config0.bits(), 0x01);
        let builder = builder.with_power_mode(PowerMode::Normal);
        assert_eq!(builder.config.acc_config0.bits(), 0x02);
    }
    #[test]
    fn test_lp_osr() {
        let mut device = get_test_device();
        let builder = device.config_accel();
        let builder = builder.with_osr_lp(OversampleRate::OSR0);
        assert_eq!(builder.config.acc_config0.bits(), 0x00);
        let builder = builder.with_osr_lp(OversampleRate::OSR1);
        assert_eq!(builder.config.acc_config0.bits(), 0x20);
        let builder = builder.with_osr_lp(OversampleRate::OSR2);
        assert_eq!(builder.config.acc_config0.bits(), 0x40);
        let builder = builder.with_osr_lp(OversampleRate::OSR3);
        assert_eq!(builder.config.acc_config0.bits(), 0x60);
    }
    #[test]
    fn test_filt1_bw() {
        let mut device = get_test_device();
        let builder = device.config_accel();
        let builder = builder.with_filt1_bw(Filter1Bandwidth::Low);
        assert_eq!(builder.config.acc_config0.bits(), 0x80);
        let builder = builder.with_filt1_bw(Filter1Bandwidth::High);
        assert_eq!(builder.config.acc_config0.bits(), 0x00);
    }
    #[test]
    fn test_odr() {
        let mut device = get_test_device();
        let builder = device.config_accel();
        let builder = builder.with_odr(OutputDataRate::Hz12_5);
        assert_eq!(builder.config.acc_config1.bits(), 0x45);
        let builder = builder.with_odr(OutputDataRate::Hz25);
        assert_eq!(builder.config.acc_config1.bits(), 0x46);
        let builder = builder.with_odr(OutputDataRate::Hz50);
        assert_eq!(builder.config.acc_config1.bits(), 0x47);
        let builder = builder.with_odr(OutputDataRate::Hz100);
        assert_eq!(builder.config.acc_config1.bits(), 0x48);
        let builder = builder.with_odr(OutputDataRate::Hz200);
        assert_eq!(builder.config.acc_config1.bits(), 0x49);
        let builder = builder.with_odr(OutputDataRate::Hz400);
        assert_eq!(builder.config.acc_config1.bits(), 0x4A);
        let builder = builder.with_odr(OutputDataRate::Hz800);
        assert_eq!(builder.config.acc_config1.bits(), 0x4B);
    }
    #[test]
    fn test_osr() {
        let mut device = get_test_device();
        let builder = device.config_accel();
        let builder = builder.with_osr(OversampleRate::OSR0);
        assert_eq!(builder.config.acc_config1.bits(), 0x49);
        let builder = builder.with_osr(OversampleRate::OSR1);
        assert_eq!(builder.config.acc_config1.bits(), 0x59);
        let builder = builder.with_osr(OversampleRate::OSR2);
        assert_eq!(builder.config.acc_config1.bits(), 0x69);
        let builder = builder.with_osr(OversampleRate::OSR3);
        assert_eq!(builder.config.acc_config1.bits(), 0x79);
    }
    #[test]
    fn test_scale() {
        let mut device = get_test_device();
        let builder = device.config_accel();
        let builder = builder.with_scale(Scale::Range2G);
        assert_eq!(builder.config.acc_config1.bits(), 0x09);
        let builder = builder.with_scale(Scale::Range4G);
        assert_eq!(builder.config.acc_config1.bits(), 0x49);
        let builder = builder.with_scale(Scale::Range8G);
        assert_eq!(builder.config.acc_config1.bits(), 0x89);
        let builder = builder.with_scale(Scale::Range16G);
        assert_eq!(builder.config.acc_config1.bits(), 0xC9);
    }
    #[test]
    fn test_dta_src() {
        let mut device = get_test_device();
        let builder = device.config_accel();
        let builder = builder.with_reg_dta_src(DataSource::AccFilt1);
        assert_eq!(builder.config.acc_config2.bits(), 0x00);
        let builder = builder.with_reg_dta_src(DataSource::AccFilt2);
        assert_eq!(builder.config.acc_config2.bits(), 0x04);
        let builder = builder.with_reg_dta_src(DataSource::AccFilt2Lp);
        assert_eq!(builder.config.acc_config2.bits(), 0x08);
    }
    #[test]
    fn test_actch_config_err() {
        let mut device = get_test_device();
        // Set the OutputDataRate to 100Hz
        assert!(matches!(
            device
                .config_accel()
                .with_odr(OutputDataRate::Hz100)
                .write(),
            Ok(())
        ));
        // Enable the Activity Change Interrupt
        assert!(matches!(
            device.config_interrupts().with_actch_int(true).write(),
            Ok(())
        ));
        // Try to change the OutputDataRate back to 200Hz
        let result = device
            .config_accel()
            .with_odr(OutputDataRate::Hz200)
            .write();
        assert!(matches!(
            result,
            Err(BMA400Error::ConfigBuildError(
                ConfigError::Filt1InterruptInvalidODR
            ))
        ));
    }
    #[test]
    fn test_gen1_int_config_err() {
        let mut device = get_test_device();
        // Set the OutputDataRate to 100Hz
        assert!(matches!(
            device
                .config_accel()
                .with_odr(OutputDataRate::Hz100)
                .write(),
            Ok(())
        ));
        // Enable Generic Interrupt 1
        assert!(matches!(
            device.config_interrupts().with_gen1_int(true).write(),
            Ok(())
        ));
        // Try to change the OutputDataRate back to 200Hz
        let result = device
            .config_accel()
            .with_odr(OutputDataRate::Hz200)
            .write();
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
        // Set the OutputDataRate to 100Hz
        assert!(matches!(
            device
                .config_accel()
                .with_odr(OutputDataRate::Hz100)
                .write(),
            Ok(())
        ));
        // Enable Generic Interrupt 1
        assert!(matches!(
            device.config_interrupts().with_gen2_int(true).write(),
            Ok(())
        ));
        // Try to change the OutputDataRate back to 200Hz
        let result = device
            .config_accel()
            .with_odr(OutputDataRate::Hz200)
            .write();
        assert!(matches!(
            result,
            Err(BMA400Error::ConfigBuildError(
                ConfigError::Filt1InterruptInvalidODR
            ))
        ));
    }
    #[test]
    fn test_tap_int_config_err() {
        let mut device = get_test_device();
        // Set the OutputDataRate to 200Hz (no write performed since default is 200Hz)
        assert!(matches!(
            device
                .config_accel()
                .with_odr(OutputDataRate::Hz200)
                .write(),
            Ok(())
        ));
        // Enable the Single Tap Interrupt
        assert!(matches!(
            device.config_interrupts().with_s_tap_int(true).write(),
            Ok(())
        ));
        // Try to change the OutputDataRate to 100Hz
        let result = device
            .config_accel()
            .with_odr(OutputDataRate::Hz100)
            .write();
        assert!(matches!(
            result,
            Err(BMA400Error::ConfigBuildError(
                ConfigError::TapIntEnabledInvalidODR
            ))
        ));
        // Disable the Single Tap Interrupt
        assert!(matches!(
            device.config_interrupts().with_s_tap_int(false).write(),
            Ok(())
        ));
        // Enable the Double Tap Interrupt
        assert!(matches!(
            device.config_interrupts().with_d_tap_int(true).write(),
            Ok(())
        ));
        // Try to change the OutputDataRate to 100Hz
        let result = device
            .config_accel()
            .with_odr(OutputDataRate::Hz100)
            .write();
        assert!(matches!(
            result,
            Err(BMA400Error::ConfigBuildError(
                ConfigError::TapIntEnabledInvalidODR
            ))
        ));
    }
}
