use crate::{
    Debug,
    registers::{Int1Map, Int2Map, Int12Map, Int12IOCtrl},
    interface::WriteToRegister,
    BMA400,
    ConfigError, 
    PinOutputConfig, 
    InterruptPins, 
};

#[derive(Clone, Default)]
pub struct IntPinConfig {
    int1_map: Int1Map,
    int2_map: Int2Map,
    int12_map: Int12Map,
    int12_io_ctrl: Int12IOCtrl,
}

impl IntPinConfig {
    fn mapped_pins(int1: bool, int2: bool) -> InterruptPins {
        match (int1, int2) {
            (false, false) => InterruptPins::None,
            (true, false) => InterruptPins::Int1,
            (false, true) => InterruptPins::Int2,
            (true, true) => InterruptPins::Both,
        }
    }
    pub fn drdy_map(&self) -> InterruptPins {
        Self::mapped_pins(self.int1_map.drdy_int(), self.int2_map.drdy_int())
    }
    pub fn fwm_map(&self) -> InterruptPins {
        Self::mapped_pins(self.int1_map.fwm_int(), self.int2_map.fwm_int())
    }
    pub fn ffull_map(&self) -> InterruptPins {
        Self::mapped_pins(self.int1_map.ffull_int(), self.int2_map.ffull_int())
    }
    pub fn gen1_map(&self) -> InterruptPins {
        Self::mapped_pins(self.int1_map.gen1_int(), self.int2_map.gen1_int())
    }
    pub fn gen2_map(&self) -> InterruptPins {
        Self::mapped_pins(self.int1_map.gen2_int(), self.int2_map.gen2_int())
    }
    pub fn wkup_map(&self) -> InterruptPins {
        Self::mapped_pins(self.int1_map.wkup_int(), self.int2_map.wkup_int())
    }
    pub fn orientch_map(&self) -> InterruptPins {
        Self::mapped_pins(self.int1_map.orientch_int(), self.int2_map.orientch_int())
    }
    pub fn ovrrn_map(&self) -> InterruptPins {
        Self::mapped_pins(self.int1_map.ovrrn_int(), self.int2_map.ovrrn_int())
    }
    pub fn actch_map(&self) -> InterruptPins {
        Self::mapped_pins(self.int12_map.actch_int1(), self.int12_map.actch_int2())
    }
    pub fn tap_map(&self) -> InterruptPins {
        Self::mapped_pins(self.int12_map.tap_int1(), self.int12_map.tap_int2())
    }
    pub fn step_map(&self) -> InterruptPins {
        Self::mapped_pins(self.int12_map.step_int1(), self.int12_map.step_int2())
    }
}

pub struct IntPinConfigBuilder<'a, Interface> {
    config: IntPinConfig,
    device: &'a mut BMA400<Interface>,
}

impl<'a, Interface, E> IntPinConfigBuilder<'a, Interface> 
where
    Interface: WriteToRegister<Error = E>,
    E: From<ConfigError> + Debug,
{
    pub fn new(config: IntPinConfig, device: &'a mut BMA400<Interface>) -> IntPinConfigBuilder<'a, Interface> {
        IntPinConfigBuilder { config, device }
    }
    // Int1Map / Int2Map
    /// Map Data Ready Interrupt to [InterruptPins]
    pub fn with_drdy(mut self, mapped_to: InterruptPins) -> Self {
        let (int1, int2) = match mapped_to {
            InterruptPins::None => (false, false),
            InterruptPins::Int1 => (true, false),
            InterruptPins::Int2 => (false, true),
            InterruptPins::Both => (true, true),
        };
        self.config.int1_map = self.config.int1_map.with_drdy(int1);
        self.config.int2_map = self.config.int2_map.with_drdy(int2);
        self
    }
    /// Map Fifo Watermark Interrupt to [InterruptPins]
    pub fn with_fifo_wm(mut self, mapped_to: InterruptPins) -> Self {
        let (int1, int2) = match mapped_to {
            InterruptPins::None => (false, false),
            InterruptPins::Int1 => (true, false),
            InterruptPins::Int2 => (false, true),
            InterruptPins::Both => (true, true),
        };
        self.config.int1_map = self.config.int1_map.with_fwm(int1);
        self.config.int2_map = self.config.int2_map.with_fwm(int2);
        self
    }
    /// Map Fifo Full Interrupt to [InterruptPins]
    pub fn with_ffull(mut self, mapped_to: InterruptPins) -> Self {
        let (int1, int2) = match mapped_to {
            InterruptPins::None => (false, false),
            InterruptPins::Int1 => (true, false),
            InterruptPins::Int2 => (false, true),
            InterruptPins::Both => (true, true),
        };
        self.config.int1_map = self.config.int1_map.with_ffull(int1);
        self.config.int2_map = self.config.int2_map.with_ffull(int2);
        self
    }
    /// Map Interrupt Engine Overrun Interrupt to [InterruptPins]
    pub fn with_ieng_ovrrn(mut self, mapped_to: InterruptPins) -> Self {
        let (int1, int2) = match mapped_to {
            InterruptPins::None => (false, false),
            InterruptPins::Int1 => (true, false),
            InterruptPins::Int2 => (false, true),
            InterruptPins::Both => (true, true),
        };
        self.config.int1_map = self.config.int1_map.with_ovrrn(int1);
        self.config.int2_map = self.config.int2_map.with_ovrrn(int2);
        self
    }
    /// Map Generic Interrupt 2 to [InterruptPins]
    pub fn with_gen2(mut self, mapped_to: InterruptPins) -> Self {
        let (int1, int2) = match mapped_to {
            InterruptPins::None => (false, false),
            InterruptPins::Int1 => (true, false),
            InterruptPins::Int2 => (false, true),
            InterruptPins::Both => (true, true),
        };
        self.config.int1_map = self.config.int1_map.with_gen2(int1);
        self.config.int2_map = self.config.int2_map.with_gen2(int2);
        self
    }
    /// Map Generic Interrupt 1 to [InterruptPins]
    pub fn with_gen1(mut self, mapped_to: InterruptPins) -> Self {
        let (int1, int2) = match mapped_to {
            InterruptPins::None => (false, false),
            InterruptPins::Int1 => (true, false),
            InterruptPins::Int2 => (false, true),
            InterruptPins::Both => (true, true),
        };
        self.config.int1_map = self.config.int1_map.with_gen1(int1);
        self.config.int2_map = self.config.int2_map.with_gen1(int2);
        self
    }
    /// Map Orientation Change Interrupt to [InterruptPins]
    pub fn with_orientch(mut self, mapped_to: InterruptPins) -> Self {
        let (int1, int2) = match mapped_to {
            InterruptPins::None => (false, false),
            InterruptPins::Int1 => (true, false),
            InterruptPins::Int2 => (false, true),
            InterruptPins::Both => (true, true),
        };
        self.config.int1_map = self.config.int1_map.with_orientch(int1);
        self.config.int2_map = self.config.int2_map.with_orientch(int2);
        self
    }
    /// Map Wakeup Interrupt to [InterruptPins]
    pub fn with_wkup(mut self, mapped_to: InterruptPins) -> Self {
        let (int1, int2) = match mapped_to {
            InterruptPins::None => (false, false),
            InterruptPins::Int1 => (true, false),
            InterruptPins::Int2 => (false, true),
            InterruptPins::Both => (true, true),
        };
        self.config.int1_map = self.config.int1_map.with_wkup(int1);
        self.config.int2_map = self.config.int2_map.with_wkup(int2);
        self
    }

    // Int12Map

    /// Map Activity Changed Interrupt to [InterruptPins]
    pub fn with_actch(mut self, mapped_to: InterruptPins) -> Self {
        let (int1, int2) = match mapped_to {
            InterruptPins::None => (false, false),
            InterruptPins::Int1 => (true, false),
            InterruptPins::Int2 => (false, true),
            InterruptPins::Both => (true, true),
        };
        self.config.int12_map = self.config.int12_map.with_actch1(int1).with_actch2(int2);
        self
    }
    /// Map Tap Interrupt to [InterruptPins]
    pub fn with_tap(mut self, mapped_to: InterruptPins) -> Self {
    let (int1, int2) = match mapped_to {
        InterruptPins::None => (false, false),
        InterruptPins::Int1 => (true, false),
        InterruptPins::Int2 => (false, true),
        InterruptPins::Both => (true, true),
    };
    self.config.int12_map = self.config.int12_map.with_tap1(int1).with_tap2(int2);
    self
    }
    /// Map Step Interrupt to [InterruptPins]
    pub fn with_step(mut self, mapped_to: InterruptPins) -> Self {
        let (int1, int2) = match mapped_to {
            InterruptPins::None => (false, false),
            InterruptPins::Int1 => (true, false),
            InterruptPins::Int2 => (false, true),
            InterruptPins::Both => (true, true),
        };
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
    pub fn write(mut self) -> Result<(), E> {
        // Any change of an interrupt configuration must be executed when the corresponding interrupt is
        // disabled. (Datasheet p. 40)
        
        // Collect IntConfig0 interrupts with changes
        let int_config0 = self.device.config.int_config.get_config0();
        let mut tmp_int_config0 = int_config0.clone();
        // Collect IntConfig1 interrupts with changes
        let int_config1 = self.device.config.int_config.get_config1();
        let mut tmp_int_config1 = int_config1.clone();
        // Wakeup Interrupt
        let wkup_int_config0 = self.device.config.wkup_int_config.get_config0();
        let mut tmp_wkup_int_config0 = wkup_int_config0.clone();
        // If there are electrical configuration changes
        if self.device.config.int_pin_config.int12_io_ctrl.bits() != self.config.int12_io_ctrl.bits() {
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
            if int_config0.orientch_int() && !matches!(self.config.orientch_map(), InterruptPins::None) {
                tmp_int_config0 = tmp_int_config0.with_orientch_int(false);
            }
            // Wakeup
            if self.device.config.wkup_int_config.is_int_en() && !matches!(self.config.wkup_map(), InterruptPins::None) {
                tmp_wkup_int_config0 = tmp_wkup_int_config0.with_x_axis(false).with_y_axis(false).with_z_axis(false);
            }
            // Activity Change
            if int_config1.actch_int() && !matches!(self.config.actch_map(), InterruptPins::None) {
                tmp_int_config1 = tmp_int_config1.with_actch_int(false);
            }
            // Tap 
            if (int_config1.s_tap_int() || int_config1.d_tap_int()) && !matches!(self.config.tap_map(), InterruptPins::None) {
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
            self.device.interface.write_register(self.config.int12_map)?;
            self.device.config.int_pin_config.int12_map = self.config.int12_map;
        }
        if self.device.config.int_pin_config.int12_io_ctrl.bits() != self.config.int12_io_ctrl.bits() {
            self.device.interface.write_register(self.config.int12_io_ctrl)?;
            self.device.config.int_pin_config.int12_io_ctrl = self.config.int12_io_ctrl;
        }
        // Restore the disabled interrupts
        if int_config0.bits() != tmp_int_config0.bits() {
            self.device.interface.write_register(int_config0)?;
        }
        if int_config0.bits() != tmp_int_config0.bits() {
            self.device.interface.write_register(int_config1)?;
        }
        if wkup_int_config0.bits() != tmp_wkup_int_config0.bits() {
            self.device.interface.write_register(wkup_int_config0)?;
        }
        Ok(())
    }
}