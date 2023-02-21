use crate::{
    Debug,
    registers::{FifoConfig0, FifoConfig1, FifoConfig2, FifoPwrConfig},
    interface::WriteToRegister,
    BMA400,
    ConfigError, 
    DataSource,
};

#[derive(Clone, Default)]
pub struct FifoConfig {
    fifo_config0: FifoConfig0,
    fifo_config1: FifoConfig1,
    fifo_config2: FifoConfig2,
    fifo_pwr_config: FifoPwrConfig
}

impl FifoConfig {
    pub fn is_read_disabled(&self) -> bool {
        self.fifo_pwr_config.fifo_pwr_disable()
    }
}

pub struct FifoConfigBuilder<'a, Interface: WriteToRegister> {
    config: FifoConfig,
    device: &'a mut BMA400<Interface>,
}

impl<'a, Interface, E> FifoConfigBuilder<'a, Interface>
where 
    Interface: WriteToRegister<Error = E>,
    E: From<ConfigError> + Debug,
{
    // FifoConfig0

    /// Manually Disable power to the FIFO Read circuit. This can save 100nA but you must wait 50µs
    /// after re-enabling before attempting to read
    /// 
    /// See Datasheet p.30
    pub fn with_read_disabled(mut self, disabled: bool) -> Self {
        self.config.fifo_pwr_config = self.config.fifo_pwr_config.with_fifo_pwr_disable(disabled);
        self
    }
    /// Enable writing measurements to the FIFO Buffer for x, y, z axis
    pub fn with_axes(mut self, x_en: bool, y_en: bool, z_en: bool) -> Self {
        self.config.fifo_config0 = self.config.fifo_config0.with_fifo_x(x_en).with_fifo_y(y_en).with_fifo_z(z_en);
        self
    }
    /// Truncates the 4 least significant bits of the reading to store the measurement of each axis in a single byte
    pub fn with_8bit_mode(mut self, enabled: bool) -> Self {
        self.config.fifo_config0 = self.config.fifo_config0.with_fifo_8bit(enabled);
        self
    }
    /// Configure FIFO Data Source
    /// 
    /// Cannot use [DataSource::AccFilt2Lp]. If passed, this will default to AccFilt2
    pub fn with_src(mut self, src: DataSource) -> Self {
        let fifo_source = match src {
            DataSource::AccFilt2Lp => DataSource::AccFilt2,
            _ => src,
        };
        self.config.fifo_config0 = self.config.fifo_config0.with_fifo_src(fifo_source);
        self
    }
    /// Enable sending a clock reading if more frames are requested than the buffer contains (> `get_fifo_len()`)
    pub fn with_send_time_on_empty(mut self, enabled: bool) -> Self {
        self.config.fifo_config0 = self.config.fifo_config0.with_send_time_on_empty(enabled);
        self
    }
    /// Define the overflow behavior
    /// 
    /// Enabled = newest frames are dropped (not written)
    /// 
    /// Disabled = oldest frames are overwritten first
    pub fn with_stop_on_full(mut self, enabled: bool) -> Self {
        self.config.fifo_config0 = self.config.fifo_config0.with_stop_on_full(enabled);
        self
    }
    /// Automatically flush FIFO Buffer when changing power mode
    pub fn with_auto_flush(mut self, enabled: bool) -> Self {
        self.config.fifo_config0 = self.config.fifo_config0.with_flush_on_pwr_mode_change(enabled);
        self
    }

    // FifoConfig1 & FifoConfig2

    /// Set the fill threshold for the FIFO watermark interrupt
    /// 
    /// Interupt will be active if FIFO length is > this value
    /// 
    /// Clamped to \[0, 1024\] See also [IntConfig].with_ffull_int()
    pub fn with_watermark_thresh(mut self, threshold: u16) -> Self {
        let thresh = threshold.clamp(0, 1024);
        let bytes = thresh.to_le_bytes();
        self.config.fifo_config1 = self.config.fifo_config1.with_fifo_wtrmk_threshold(bytes[0]);
        self.config.fifo_config2 = self.config.fifo_config2.with_fifo_wtrmk_threshold(bytes[1]);
        self
    }

    pub fn write(self) -> Result<(), E> {
        if self.device.config.fifo_config.fifo_config0.bits() != self.config.fifo_config0.bits() {
            self.device.interface.write_register(self.config.fifo_config0)?;
            self.device.config.fifo_config.fifo_config0 = self.config.fifo_config0;
        }
        let wm1_changes = self.device.config.fifo_config.fifo_config1.bits() != self.config.fifo_config1.bits();
        let wm2_changes = self.device.config.fifo_config.fifo_config2.bits() != self.config.fifo_config2.bits();
        let fifo_wm_changes = wm1_changes || wm2_changes;
        let mut tmp_int_config = self.device.config.int_config.get_config0().clone();

        // If enabled, temporarily disable the FIFO Watermark Interrupt to change the config
        if self.device.config.int_config.get_config0().fwm_int() && fifo_wm_changes {
            tmp_int_config = tmp_int_config.with_fwm_int(false);
            self.device.interface.write_register(tmp_int_config)?;
        }
        if wm1_changes {
            self.device.interface.write_register(self.config.fifo_config1)?;
            self.device.config.fifo_config.fifo_config1 = self.config.fifo_config1;
        }
        if wm2_changes {
            self.device.interface.write_register(self.config.fifo_config2)?;
            self.device.config.fifo_config.fifo_config2 = self.config.fifo_config2;
        }
        // Re-enable the interrupt if it was changed
        if self.device.config.int_config.get_config0().bits() != tmp_int_config.bits() {
            self.device.interface.write_register(self.device.config.int_config.get_config0())?;
        }
        if self.device.config.fifo_config.fifo_pwr_config.bits() != self.config.fifo_pwr_config.bits() {
            self.device.interface.write_register(self.config.fifo_pwr_config)?;
            self.device.config.fifo_config.fifo_pwr_config = self.config.fifo_pwr_config
        }
        Ok(())
    }
}