use crate::{
    interface::WriteToRegister,
    registers::{Int12IOCtrl, Int12Map, Int1Map, Int2Map},
    ConfigError, InterruptPins, PinOutputConfig, BMA400,
};

#[cfg(feature = "async")]
use crate::{interface::AsyncWriteToRegister, AsyncBMA400};

#[derive(Clone, Default)]
pub struct IntPinConfig {
    int1_map: Int1Map,
    int2_map: Int2Map,
    int12_map: Int12Map,
    int12_io_ctrl: Int12IOCtrl,
}

impl IntPinConfig {
    pub fn drdy_map(&self) -> InterruptPins {
        mapped_pins(self.int1_map.drdy_int(), self.int2_map.drdy_int())
    }
    pub fn fwm_map(&self) -> InterruptPins {
        mapped_pins(self.int1_map.fwm_int(), self.int2_map.fwm_int())
    }
    pub fn ffull_map(&self) -> InterruptPins {
        mapped_pins(self.int1_map.ffull_int(), self.int2_map.ffull_int())
    }
    pub fn gen1_map(&self) -> InterruptPins {
        mapped_pins(self.int1_map.gen1_int(), self.int2_map.gen1_int())
    }
    pub fn gen2_map(&self) -> InterruptPins {
        mapped_pins(self.int1_map.gen2_int(), self.int2_map.gen2_int())
    }
    pub fn wkup_map(&self) -> InterruptPins {
        mapped_pins(self.int1_map.wkup_int(), self.int2_map.wkup_int())
    }
    pub fn orientch_map(&self) -> InterruptPins {
        mapped_pins(self.int1_map.orientch_int(), self.int2_map.orientch_int())
    }
    pub fn actch_map(&self) -> InterruptPins {
        mapped_pins(self.int12_map.actch_int1(), self.int12_map.actch_int2())
    }
    pub fn tap_map(&self) -> InterruptPins {
        mapped_pins(self.int12_map.tap_int1(), self.int12_map.tap_int2())
    }
    pub fn step_map(&self) -> InterruptPins {
        mapped_pins(self.int12_map.step_int1(), self.int12_map.step_int2())
    }
}

/// Map interrupts to the [InterruptPins::Int1] / [InterruptPins::Int2] hardware interrupt pins
///  
/// - Control the pin electrical behavior using [`with_int1_cfg()`](IntPinConfigBuilder::with_int1_cfg) / [`with_int2_cfg()`](IntPinConfigBuilder::with_int2_cfg)
///    - [`PinOutputConfig::PushPull`] High = VDDIO, Low = GND
///    - [`PinOutputConfig::OpenDrain`] High = VDDIO, Low = High Impedance
pub struct IntPinConfigBuilder<Device> {
    config: IntPinConfig,
    device: Device,
}

fn mapped_pins(int1: bool, int2: bool) -> InterruptPins {
    match (int1, int2) {
        (false, false) => InterruptPins::None,
        (true, false) => InterruptPins::Int1,
        (false, true) => InterruptPins::Int2,
        (true, true) => InterruptPins::Both,
    }
}

fn match_mapped(mapped_to: InterruptPins) -> (bool, bool) {
    match mapped_to {
        InterruptPins::None => (false, false),
        InterruptPins::Int1 => (true, false),
        InterruptPins::Int2 => (false, true),
        InterruptPins::Both => (true, true),
    }
}

impl<Device> IntPinConfigBuilder<Device> {
    // Int1Map / Int2Map
    /// Map Data Ready Interrupt to [InterruptPins]
    pub fn with_drdy(mut self, mapped_to: InterruptPins) -> Self {
        let (int1, int2) = match_mapped(mapped_to);
        self.config.int1_map = self.config.int1_map.with_drdy(int1);
        self.config.int2_map = self.config.int2_map.with_drdy(int2);
        self
    }
    /// Map Fifo Watermark Interrupt to [InterruptPins]
    pub fn with_fifo_wm(mut self, mapped_to: InterruptPins) -> Self {
        let (int1, int2) = match_mapped(mapped_to);
        self.config.int1_map = self.config.int1_map.with_fwm(int1);
        self.config.int2_map = self.config.int2_map.with_fwm(int2);
        self
    }
    /// Map Fifo Full Interrupt to [InterruptPins]
    pub fn with_ffull(mut self, mapped_to: InterruptPins) -> Self {
        let (int1, int2) = match_mapped(mapped_to);
        self.config.int1_map = self.config.int1_map.with_ffull(int1);
        self.config.int2_map = self.config.int2_map.with_ffull(int2);
        self
    }
    /// Map Interrupt Engine Overrun Interrupt to [InterruptPins]
    pub fn with_ieng_ovrrn(mut self, mapped_to: InterruptPins) -> Self {
        let (int1, int2) = match_mapped(mapped_to);
        self.config.int1_map = self.config.int1_map.with_ovrrn(int1);
        self.config.int2_map = self.config.int2_map.with_ovrrn(int2);
        self
    }
    /// Map Generic Interrupt 2 to [InterruptPins]
    pub fn with_gen2(mut self, mapped_to: InterruptPins) -> Self {
        let (int1, int2) = match_mapped(mapped_to);
        self.config.int1_map = self.config.int1_map.with_gen2(int1);
        self.config.int2_map = self.config.int2_map.with_gen2(int2);
        self
    }
    /// Map Generic Interrupt 1 to [InterruptPins]
    pub fn with_gen1(mut self, mapped_to: InterruptPins) -> Self {
        let (int1, int2) = match_mapped(mapped_to);
        self.config.int1_map = self.config.int1_map.with_gen1(int1);
        self.config.int2_map = self.config.int2_map.with_gen1(int2);
        self
    }
    /// Map Orientation Change Interrupt to [InterruptPins]
    pub fn with_orientch(mut self, mapped_to: InterruptPins) -> Self {
        let (int1, int2) = match_mapped(mapped_to);
        self.config.int1_map = self.config.int1_map.with_orientch(int1);
        self.config.int2_map = self.config.int2_map.with_orientch(int2);
        self
    }
    /// Map Wakeup Interrupt to [InterruptPins]
    pub fn with_wkup(mut self, mapped_to: InterruptPins) -> Self {
        let (int1, int2) = match_mapped(mapped_to);
        self.config.int1_map = self.config.int1_map.with_wkup(int1);
        self.config.int2_map = self.config.int2_map.with_wkup(int2);
        self
    }

    // Int12Map
    /// Map Activity Changed Interrupt to [InterruptPins]
    pub fn with_actch(mut self, mapped_to: InterruptPins) -> Self {
        let (int1, int2) = match_mapped(mapped_to);
        self.config.int12_map = self.config.int12_map.with_actch1(int1).with_actch2(int2);
        self
    }
    /// Map Tap Interrupt to [InterruptPins]
    pub fn with_tap(mut self, mapped_to: InterruptPins) -> Self {
        let (int1, int2) = match_mapped(mapped_to);
        self.config.int12_map = self.config.int12_map.with_tap1(int1).with_tap2(int2);
        self
    }
    /// Map Step Interrupt to [InterruptPins]
    pub fn with_step(mut self, mapped_to: InterruptPins) -> Self {
        let (int1, int2) = match_mapped(mapped_to);
        self.config.int12_map = self.config.int12_map.with_step1(int1).with_step2(int2);
        self
    }

    //Int12IOCtrl
    /// Int1 Pin Output Mode
    ///
    /// See Datasheet p.39
    pub fn with_int1_cfg(mut self, config: PinOutputConfig) -> Self {
        self.config.int12_io_ctrl = self.config.int12_io_ctrl.with_int1_cfg(config);
        self
    }
    /// Int2 Pin Output Mode
    ///
    /// See Datasheet p.39
    pub fn with_int2_cfg(mut self, config: PinOutputConfig) -> Self {
        self.config.int12_io_ctrl = self.config.int12_io_ctrl.with_int2_cfg(config);
        self
    }
}

impl<'a, Interface, E> IntPinConfigBuilder<&'a mut BMA400<Interface>>
where
    Interface: WriteToRegister<Error = E>,
    E: From<ConfigError>,
{
    pub(crate) fn new(device: &'a mut BMA400<Interface>) -> Self {
        IntPinConfigBuilder {
            config: device.config.int_pin_config.clone(),
            device,
        }
    }

    /// Write this configuration to device registers
    // Clippy: ignore lint for intentional XOR with self, avoiding an awkward import / function call
    #[allow(clippy::eq_op)]
    pub fn write(self) -> Result<(), E> {
        // Any change of an interrupt configuration must be executed when the corresponding
        // interrupt is disabled. (Datasheet p. 40)

        // Collect IntConfig0 interrupts with changes
        let int_config0 = self.device.config.int_config.get_config0();
        let mut tmp_int_config0 = int_config0;
        // Collect IntConfig1 interrupts with changes
        let int_config1 = self.device.config.int_config.get_config1();
        let mut tmp_int_config1 = int_config1;
        // Wakeup Interrupt
        let wkup_int_config0 = self.device.config.wkup_int_config.get_config0();
        let mut tmp_wkup_int_config0 = wkup_int_config0;
        // If there are electrical configuration changes
        if self.device.config.int_pin_config.int12_io_ctrl.bits()
            != self.config.int12_io_ctrl.bits()
        {
            // Disable Everything
            tmp_int_config0 = tmp_int_config0 ^ tmp_int_config0;
            tmp_int_config1 = tmp_int_config1 ^ tmp_int_config1;
        } else {
            // Data Ready
            if int_config0.dta_rdy_int() && !matches!(self.config.drdy_map(), InterruptPins::None) {
                tmp_int_config0 = tmp_int_config0.with_dta_rdy_int(false);
            }
            // Fifo Watermark
            if int_config0.fwm_int() && !matches!(self.config.fwm_map(), InterruptPins::None) {
                tmp_int_config0 = tmp_int_config0.with_fwm_int(false);
            }
            // Fifo Full
            if int_config0.ffull_int() && !matches!(self.config.ffull_map(), InterruptPins::None) {
                tmp_int_config0 = tmp_int_config0.with_ffull_int(false);
            }
            // Gen Int 1
            if int_config0.gen1_int() && !matches!(self.config.gen1_map(), InterruptPins::None) {
                tmp_int_config0 = tmp_int_config0.with_gen1_int(false);
            }
            // Gen Int 2
            if int_config0.gen2_int() && !matches!(self.config.gen2_map(), InterruptPins::None) {
                tmp_int_config0 = tmp_int_config0.with_gen2_int(false);
            }
            // Orientation Change
            if int_config0.orientch_int()
                && !matches!(self.config.orientch_map(), InterruptPins::None)
            {
                tmp_int_config0 = tmp_int_config0.with_orientch_int(false);
            }
            // Wakeup
            if self.device.config.wkup_int_config.is_int_en()
                && !matches!(self.config.wkup_map(), InterruptPins::None)
            {
                tmp_wkup_int_config0 = tmp_wkup_int_config0
                    .with_x_axis(false)
                    .with_y_axis(false)
                    .with_z_axis(false);
            }
            // Activity Change
            if int_config1.actch_int() && !matches!(self.config.actch_map(), InterruptPins::None) {
                tmp_int_config1 = tmp_int_config1.with_actch_int(false);
            }
            // Tap
            if (int_config1.s_tap_int() || int_config1.d_tap_int())
                && !matches!(self.config.tap_map(), InterruptPins::None)
            {
                tmp_int_config1 = tmp_int_config1.with_d_tap_int(false).with_s_tap_int(false);
            }
            // Step
            if int_config1.step_int() && !matches!(self.config.step_map(), InterruptPins::None) {
                tmp_int_config1 = tmp_int_config1.with_step_int(false);
            }
        }
        // Write the temporary changes
        if int_config0.bits() != tmp_int_config0.bits() {
            self.device.interface.write_register(tmp_int_config0)?;
        }
        if int_config1.bits() != tmp_int_config1.bits() {
            self.device.interface.write_register(tmp_int_config1)?;
        }
        if wkup_int_config0.bits() != tmp_wkup_int_config0.bits() {
            self.device.interface.write_register(wkup_int_config0)?;
        }
        // Write the config changes
        if self.device.config.int_pin_config.int1_map.bits() != self.config.int1_map.bits() {
            self.device.interface.write_register(self.config.int1_map)?;
            self.device.config.int_pin_config.int1_map = self.config.int1_map;
        }
        if self.device.config.int_pin_config.int2_map.bits() != self.config.int2_map.bits() {
            self.device.interface.write_register(self.config.int2_map)?;
            self.device.config.int_pin_config.int2_map = self.config.int2_map;
        }
        if self.device.config.int_pin_config.int12_map.bits() != self.config.int12_map.bits() {
            self.device
                .interface
                .write_register(self.config.int12_map)?;
            self.device.config.int_pin_config.int12_map = self.config.int12_map;
        }
        if self.device.config.int_pin_config.int12_io_ctrl.bits()
            != self.config.int12_io_ctrl.bits()
        {
            self.device
                .interface
                .write_register(self.config.int12_io_ctrl)?;
            self.device.config.int_pin_config.int12_io_ctrl = self.config.int12_io_ctrl;
        }
        // Restore the disabled interrupts
        if self.device.config.int_config.get_config0().bits() != tmp_int_config0.bits() {
            self.device.interface.write_register(int_config0)?;
        }
        if self.device.config.int_config.get_config1().bits() != tmp_int_config0.bits() {
            self.device.interface.write_register(int_config1)?;
        }
        if wkup_int_config0.bits() != tmp_wkup_int_config0.bits() {
            self.device.interface.write_register(wkup_int_config0)?;
        }
        Ok(())
    }
}

#[cfg(feature = "async")]
#[cfg_attr(docsrs, doc(cfg(feature = "async")))]
impl<'a, Interface, E> IntPinConfigBuilder<&'a mut AsyncBMA400<Interface>>
where
    Interface: AsyncWriteToRegister<Error = E>,
    E: From<ConfigError>,
{
    pub(crate) fn new_async(device: &'a mut AsyncBMA400<Interface>) -> Self {
        IntPinConfigBuilder {
            config: device.config.int_pin_config.clone(),
            device,
        }
    }

    /// Write this configuration to device registers
    // Clippy: ignore lint for intentional XOR with self, avoiding an awkward import / function call
    #[allow(clippy::eq_op)]
    pub async fn write(self) -> Result<(), E> {
        // Any change of an interrupt configuration must be executed when the corresponding
        // interrupt is disabled. (Datasheet p. 40)

        // Collect IntConfig0 interrupts with changes
        let int_config0 = self.device.config.int_config.get_config0();
        let mut tmp_int_config0 = int_config0;
        // Collect IntConfig1 interrupts with changes
        let int_config1 = self.device.config.int_config.get_config1();
        let mut tmp_int_config1 = int_config1;
        // Wakeup Interrupt
        let wkup_int_config0 = self.device.config.wkup_int_config.get_config0();
        let mut tmp_wkup_int_config0 = wkup_int_config0;
        // If there are electrical configuration changes
        if self.device.config.int_pin_config.int12_io_ctrl.bits()
            != self.config.int12_io_ctrl.bits()
        {
            // Disable Everything
            tmp_int_config0 = tmp_int_config0 ^ tmp_int_config0;
            tmp_int_config1 = tmp_int_config1 ^ tmp_int_config1;
        } else {
            // Data Ready
            if int_config0.dta_rdy_int() && !matches!(self.config.drdy_map(), InterruptPins::None) {
                tmp_int_config0 = tmp_int_config0.with_dta_rdy_int(false);
            }
            // Fifo Watermark
            if int_config0.fwm_int() && !matches!(self.config.fwm_map(), InterruptPins::None) {
                tmp_int_config0 = tmp_int_config0.with_fwm_int(false);
            }
            // Fifo Full
            if int_config0.ffull_int() && !matches!(self.config.ffull_map(), InterruptPins::None) {
                tmp_int_config0 = tmp_int_config0.with_ffull_int(false);
            }
            // Gen Int 1
            if int_config0.gen1_int() && !matches!(self.config.gen1_map(), InterruptPins::None) {
                tmp_int_config0 = tmp_int_config0.with_gen1_int(false);
            }
            // Gen Int 2
            if int_config0.gen2_int() && !matches!(self.config.gen2_map(), InterruptPins::None) {
                tmp_int_config0 = tmp_int_config0.with_gen2_int(false);
            }
            // Orientation Change
            if int_config0.orientch_int()
                && !matches!(self.config.orientch_map(), InterruptPins::None)
            {
                tmp_int_config0 = tmp_int_config0.with_orientch_int(false);
            }
            // Wakeup
            if self.device.config.wkup_int_config.is_int_en()
                && !matches!(self.config.wkup_map(), InterruptPins::None)
            {
                tmp_wkup_int_config0 = tmp_wkup_int_config0
                    .with_x_axis(false)
                    .with_y_axis(false)
                    .with_z_axis(false);
            }
            // Activity Change
            if int_config1.actch_int() && !matches!(self.config.actch_map(), InterruptPins::None) {
                tmp_int_config1 = tmp_int_config1.with_actch_int(false);
            }
            // Tap
            if (int_config1.s_tap_int() || int_config1.d_tap_int())
                && !matches!(self.config.tap_map(), InterruptPins::None)
            {
                tmp_int_config1 = tmp_int_config1.with_d_tap_int(false).with_s_tap_int(false);
            }
            // Step
            if int_config1.step_int() && !matches!(self.config.step_map(), InterruptPins::None) {
                tmp_int_config1 = tmp_int_config1.with_step_int(false);
            }
        }
        // Write the temporary changes
        if int_config0.bits() != tmp_int_config0.bits() {
            self.device
                .interface
                .write_register(tmp_int_config0)
                .await?;
        }
        if int_config1.bits() != tmp_int_config1.bits() {
            self.device
                .interface
                .write_register(tmp_int_config1)
                .await?;
        }
        if wkup_int_config0.bits() != tmp_wkup_int_config0.bits() {
            self.device
                .interface
                .write_register(wkup_int_config0)
                .await?;
        }
        // Write the config changes
        if self.device.config.int_pin_config.int1_map.bits() != self.config.int1_map.bits() {
            self.device
                .interface
                .write_register(self.config.int1_map)
                .await?;
            self.device.config.int_pin_config.int1_map = self.config.int1_map;
        }
        if self.device.config.int_pin_config.int2_map.bits() != self.config.int2_map.bits() {
            self.device
                .interface
                .write_register(self.config.int2_map)
                .await?;
            self.device.config.int_pin_config.int2_map = self.config.int2_map;
        }
        if self.device.config.int_pin_config.int12_map.bits() != self.config.int12_map.bits() {
            self.device
                .interface
                .write_register(self.config.int12_map)
                .await?;
            self.device.config.int_pin_config.int12_map = self.config.int12_map;
        }
        if self.device.config.int_pin_config.int12_io_ctrl.bits()
            != self.config.int12_io_ctrl.bits()
        {
            self.device
                .interface
                .write_register(self.config.int12_io_ctrl)
                .await?;
            self.device.config.int_pin_config.int12_io_ctrl = self.config.int12_io_ctrl;
        }
        // Restore the disabled interrupts
        if self.device.config.int_config.get_config0().bits() != tmp_int_config0.bits() {
            self.device.interface.write_register(int_config0).await?;
        }
        if self.device.config.int_config.get_config1().bits() != tmp_int_config0.bits() {
            self.device.interface.write_register(int_config1).await?;
        }
        if wkup_int_config0.bits() != tmp_wkup_int_config0.bits() {
            self.device
                .interface
                .write_register(wkup_int_config0)
                .await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{tests::get_test_device, PinOutputLevel};
    #[test]
    fn test_mapped_pins() {
        assert!(matches!(mapped_pins(false, false), InterruptPins::None));
        assert!(matches!(mapped_pins(true, false), InterruptPins::Int1));
        assert!(matches!(mapped_pins(false, true), InterruptPins::Int2));
        assert!(matches!(mapped_pins(true, true), InterruptPins::Both));
    }
    #[test]
    fn test_match_mapped() {
        assert!(matches!(match_mapped(InterruptPins::None), (false, false)));
        assert!(matches!(match_mapped(InterruptPins::Int1), (true, false)));
        assert!(matches!(match_mapped(InterruptPins::Int2), (false, true)));
        assert!(matches!(match_mapped(InterruptPins::Both), (true, true)));
    }
    #[test]
    fn test_drdy() {
        let mut device = get_test_device();
        let builder = device.config_int_pins();
        let builder = builder.with_drdy(InterruptPins::Both);
        assert_eq!(builder.config.int1_map.bits(), 0x80);
        assert_eq!(builder.config.int2_map.bits(), 0x80);
        let builder = builder.with_drdy(InterruptPins::Int1);
        assert_eq!(builder.config.int1_map.bits(), 0x80);
        assert_eq!(builder.config.int2_map.bits(), 0x00);
        let builder = builder.with_drdy(InterruptPins::Int2);
        assert_eq!(builder.config.int1_map.bits(), 0x00);
        assert_eq!(builder.config.int2_map.bits(), 0x80);
        let builder = builder.with_drdy(InterruptPins::None);
        assert_eq!(builder.config.int1_map.bits(), 0x00);
        assert_eq!(builder.config.int2_map.bits(), 0x00);
    }
    #[test]
    fn test_fwm() {
        let mut device = get_test_device();
        let builder = device.config_int_pins();
        let builder = builder.with_fifo_wm(InterruptPins::Both);
        assert_eq!(builder.config.int1_map.bits(), 0x40);
        assert_eq!(builder.config.int2_map.bits(), 0x40);
        let builder = builder.with_fifo_wm(InterruptPins::Int1);
        assert_eq!(builder.config.int1_map.bits(), 0x40);
        assert_eq!(builder.config.int2_map.bits(), 0x00);
        let builder = builder.with_fifo_wm(InterruptPins::Int2);
        assert_eq!(builder.config.int1_map.bits(), 0x00);
        assert_eq!(builder.config.int2_map.bits(), 0x40);
        let builder = builder.with_fifo_wm(InterruptPins::None);
        assert_eq!(builder.config.int1_map.bits(), 0x00);
        assert_eq!(builder.config.int2_map.bits(), 0x00);
    }
    #[test]
    fn test_ffull() {
        let mut device = get_test_device();
        let builder = device.config_int_pins();
        let builder = builder.with_ffull(InterruptPins::Both);
        assert_eq!(builder.config.int1_map.bits(), 0x20);
        assert_eq!(builder.config.int2_map.bits(), 0x20);
        let builder = builder.with_ffull(InterruptPins::Int1);
        assert_eq!(builder.config.int1_map.bits(), 0x20);
        assert_eq!(builder.config.int2_map.bits(), 0x00);
        let builder = builder.with_ffull(InterruptPins::Int2);
        assert_eq!(builder.config.int1_map.bits(), 0x00);
        assert_eq!(builder.config.int2_map.bits(), 0x20);
        let builder = builder.with_ffull(InterruptPins::None);
        assert_eq!(builder.config.int1_map.bits(), 0x00);
        assert_eq!(builder.config.int2_map.bits(), 0x00);
    }
    #[test]
    fn test_ieng_ovrrn() {
        let mut device = get_test_device();
        let builder = device.config_int_pins();
        let builder = builder.with_ieng_ovrrn(InterruptPins::Both);
        assert_eq!(builder.config.int1_map.bits(), 0x10);
        assert_eq!(builder.config.int2_map.bits(), 0x10);
        let builder = builder.with_ieng_ovrrn(InterruptPins::Int1);
        assert_eq!(builder.config.int1_map.bits(), 0x10);
        assert_eq!(builder.config.int2_map.bits(), 0x00);
        let builder = builder.with_ieng_ovrrn(InterruptPins::Int2);
        assert_eq!(builder.config.int1_map.bits(), 0x00);
        assert_eq!(builder.config.int2_map.bits(), 0x10);
        let builder = builder.with_ieng_ovrrn(InterruptPins::None);
        assert_eq!(builder.config.int1_map.bits(), 0x00);
        assert_eq!(builder.config.int2_map.bits(), 0x00);
    }
    #[test]
    fn test_gen2() {
        let mut device = get_test_device();
        let builder = device.config_int_pins();
        let builder = builder.with_gen2(InterruptPins::Both);
        assert_eq!(builder.config.int1_map.bits(), 0x08);
        assert_eq!(builder.config.int2_map.bits(), 0x08);
        let builder = builder.with_gen2(InterruptPins::Int1);
        assert_eq!(builder.config.int1_map.bits(), 0x08);
        assert_eq!(builder.config.int2_map.bits(), 0x00);
        let builder = builder.with_gen2(InterruptPins::Int2);
        assert_eq!(builder.config.int1_map.bits(), 0x00);
        assert_eq!(builder.config.int2_map.bits(), 0x08);
        let builder = builder.with_gen2(InterruptPins::None);
        assert_eq!(builder.config.int1_map.bits(), 0x00);
        assert_eq!(builder.config.int2_map.bits(), 0x00);
    }
    #[test]
    fn test_gen1() {
        let mut device = get_test_device();
        let builder = device.config_int_pins();
        let builder = builder.with_gen1(InterruptPins::Both);
        assert_eq!(builder.config.int1_map.bits(), 0x04);
        assert_eq!(builder.config.int2_map.bits(), 0x04);
        let builder = builder.with_gen1(InterruptPins::Int1);
        assert_eq!(builder.config.int1_map.bits(), 0x04);
        assert_eq!(builder.config.int2_map.bits(), 0x00);
        let builder = builder.with_gen1(InterruptPins::Int2);
        assert_eq!(builder.config.int1_map.bits(), 0x00);
        assert_eq!(builder.config.int2_map.bits(), 0x04);
        let builder = builder.with_gen1(InterruptPins::None);
        assert_eq!(builder.config.int1_map.bits(), 0x00);
        assert_eq!(builder.config.int2_map.bits(), 0x00);
    }
    #[test]
    fn test_orientch() {
        let mut device = get_test_device();
        let builder = device.config_int_pins();
        let builder = builder.with_orientch(InterruptPins::Both);
        assert_eq!(builder.config.int1_map.bits(), 0x02);
        assert_eq!(builder.config.int2_map.bits(), 0x02);
        let builder = builder.with_orientch(InterruptPins::Int1);
        assert_eq!(builder.config.int1_map.bits(), 0x02);
        assert_eq!(builder.config.int2_map.bits(), 0x00);
        let builder = builder.with_orientch(InterruptPins::Int2);
        assert_eq!(builder.config.int1_map.bits(), 0x00);
        assert_eq!(builder.config.int2_map.bits(), 0x02);
        let builder = builder.with_orientch(InterruptPins::None);
        assert_eq!(builder.config.int1_map.bits(), 0x00);
        assert_eq!(builder.config.int2_map.bits(), 0x00);
    }
    #[test]
    fn test_wkup() {
        let mut device = get_test_device();
        let builder = device.config_int_pins();
        let builder = builder.with_wkup(InterruptPins::Both);
        assert_eq!(builder.config.int1_map.bits(), 0x01);
        assert_eq!(builder.config.int2_map.bits(), 0x01);
        let builder = builder.with_wkup(InterruptPins::Int1);
        assert_eq!(builder.config.int1_map.bits(), 0x01);
        assert_eq!(builder.config.int2_map.bits(), 0x00);
        let builder = builder.with_wkup(InterruptPins::Int2);
        assert_eq!(builder.config.int1_map.bits(), 0x00);
        assert_eq!(builder.config.int2_map.bits(), 0x01);
        let builder = builder.with_wkup(InterruptPins::None);
        assert_eq!(builder.config.int1_map.bits(), 0x00);
        assert_eq!(builder.config.int2_map.bits(), 0x00);
    }
    #[test]
    fn test_actch() {
        let mut device = get_test_device();
        let builder = device.config_int_pins();
        let builder = builder.with_actch(InterruptPins::Both);
        assert_eq!(builder.config.int12_map.bits(), 0x88);
        let builder = builder.with_actch(InterruptPins::Int1);
        assert_eq!(builder.config.int12_map.bits(), 0x08);
        let builder = builder.with_actch(InterruptPins::Int2);
        assert_eq!(builder.config.int12_map.bits(), 0x80);
        let builder = builder.with_actch(InterruptPins::None);
        assert_eq!(builder.config.int12_map.bits(), 0x00);
    }
    #[test]
    fn test_tap() {
        let mut device = get_test_device();
        let builder = device.config_int_pins();
        let builder = builder.with_tap(InterruptPins::Both);
        assert_eq!(builder.config.int12_map.bits(), 0x44);
        let builder = builder.with_tap(InterruptPins::Int1);
        assert_eq!(builder.config.int12_map.bits(), 0x04);
        let builder = builder.with_tap(InterruptPins::Int2);
        assert_eq!(builder.config.int12_map.bits(), 0x40);
        let builder = builder.with_tap(InterruptPins::None);
        assert_eq!(builder.config.int12_map.bits(), 0x00);
    }
    #[test]
    fn test_step() {
        let mut device = get_test_device();
        let builder = device.config_int_pins();
        let builder = builder.with_step(InterruptPins::Both);
        assert_eq!(builder.config.int12_map.bits(), 0x11);
        let builder = builder.with_step(InterruptPins::Int1);
        assert_eq!(builder.config.int12_map.bits(), 0x01);
        let builder = builder.with_step(InterruptPins::Int2);
        assert_eq!(builder.config.int12_map.bits(), 0x10);
        let builder = builder.with_step(InterruptPins::None);
        assert_eq!(builder.config.int12_map.bits(), 0x00);
    }
    #[test]
    fn test_int1_cfg() {
        let mut device = get_test_device();
        let builder = device.config_int_pins();
        let builder = builder.with_int1_cfg(PinOutputConfig::OpenDrain(PinOutputLevel::ActiveLow));
        assert_eq!(builder.config.int12_io_ctrl.bits(), 0x24);
        let builder = builder.with_int1_cfg(PinOutputConfig::OpenDrain(PinOutputLevel::ActiveHigh));
        assert_eq!(builder.config.int12_io_ctrl.bits(), 0x26);
        let builder = builder.with_int1_cfg(PinOutputConfig::PushPull(PinOutputLevel::ActiveLow));
        assert_eq!(builder.config.int12_io_ctrl.bits(), 0x20);
        let builder = builder.with_int1_cfg(PinOutputConfig::PushPull(PinOutputLevel::ActiveHigh));
        assert_eq!(builder.config.int12_io_ctrl.bits(), 0x22);
    }
    #[test]
    fn test_int2_cfg() {
        let mut device = get_test_device();
        let builder = device.config_int_pins();
        let builder = builder.with_int2_cfg(PinOutputConfig::OpenDrain(PinOutputLevel::ActiveLow));
        assert_eq!(builder.config.int12_io_ctrl.bits(), 0x42);
        let builder = builder.with_int2_cfg(PinOutputConfig::OpenDrain(PinOutputLevel::ActiveHigh));
        assert_eq!(builder.config.int12_io_ctrl.bits(), 0x62);
        let builder = builder.with_int2_cfg(PinOutputConfig::PushPull(PinOutputLevel::ActiveLow));
        assert_eq!(builder.config.int12_io_ctrl.bits(), 0x02);
        let builder = builder.with_int2_cfg(PinOutputConfig::PushPull(PinOutputLevel::ActiveHigh));
        assert_eq!(builder.config.int12_io_ctrl.bits(), 0x22);
    }
}
