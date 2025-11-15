use crate::{
    BMA400, DataSource,
    registers::{FifoConfig0, FifoConfig1, FifoConfig2, FifoPwrConfig},
};

#[derive(Clone, Default)]
pub struct FifoConfig {
    fifo_config0: FifoConfig0,
    fifo_config1: FifoConfig1,
    fifo_config2: FifoConfig2,
    fifo_pwr_config: FifoPwrConfig,
}

impl FifoConfig {
    pub fn is_read_disabled(&self) -> bool {
        self.fifo_pwr_config.fifo_pwr_disable()
    }
    pub fn get_config0(&self) -> FifoConfig0 {
        self.fifo_config0
    }
}

/// Configure the 1024 byte FIFO Buffer Behavior
///
/// - Enable / Disable writing data for axes using [`with_axes()`](FifoConfigBuilder::with_axes)
/// - Enable / Disable 8 bit mode (truncate the 4 least significant bits) to save space in the buffer using [`with_8bit_mode`](FifoConfigBuilder::with_8bit_mode)
/// - [DataSource] for the FIFO Buffer using [`with_src()`](FifoConfigBuilder::with_src)
/// - Enable / Disable sending a clock reading (once) on overreading the buffer using [`with_send_time_on_empty()`](FifoConfigBuilder::with_send_time_on_empty)
/// - Enable / Disable overwriting oldest frames using [`with_stop_on_full()`](FifoConfigBuilder::with_stop_on_full)
/// - Enable / Disable automatic flush on power mode change using [`with_auto_flush()`](FifoConfigBuilder::with_auto_flush)
/// - Set the fill threshold for the FIFO watermark interrupt using [`with_watermark_thresh()`](FifoConfigBuilder::with_watermark_thresh)
/// - Manually Enable / Disable the FIFO read circuit using [`with_read_disabled()`](FifoConfigBuilder::with_read_disabled)
pub struct FifoConfigBuilder<'a, Interface> {
    config: FifoConfig,
    device: &'a mut BMA400<Interface>,
}

#[cfg(not(feature = "embedded-hal-async"))]
impl<'a, Interface, E> FifoConfigBuilder<'a, Interface>
where
    Interface: crate::blocking::WriteToRegister<Error = E>,
{
    /// Write this configuration to device registers
    pub fn write(self) -> Result<(), E> {
        if self.device.config.fifo_config.fifo_config0.bits() != self.config.fifo_config0.bits() {
            self.device
                .interface
                .write_register(self.config.fifo_config0)?;
            self.device.config.fifo_config.fifo_config0 = self.config.fifo_config0;
        }
        let wm1_changes =
            self.device.config.fifo_config.fifo_config1.bits() != self.config.fifo_config1.bits();
        let wm2_changes =
            self.device.config.fifo_config.fifo_config2.bits() != self.config.fifo_config2.bits();
        let fifo_wm_changes = wm1_changes || wm2_changes;
        let mut tmp_int_config = self.device.config.int_config.get_config0();

        // If enabled, temporarily disable the FIFO Watermark Interrupt to change the config
        if self.device.config.int_config.get_config0().fwm_int() && fifo_wm_changes {
            tmp_int_config = tmp_int_config.with_fwm_int(false);
            self.device.interface.write_register(tmp_int_config)?;
        }
        if wm1_changes {
            self.device
                .interface
                .write_register(self.config.fifo_config1)?;
            self.device.config.fifo_config.fifo_config1 = self.config.fifo_config1;
        }
        if wm2_changes {
            self.device
                .interface
                .write_register(self.config.fifo_config2)?;
            self.device.config.fifo_config.fifo_config2 = self.config.fifo_config2;
        }
        // Re-enable the interrupt if it was changed
        if self.device.config.int_config.get_config0().bits() != tmp_int_config.bits() {
            self.device
                .interface
                .write_register(self.device.config.int_config.get_config0())?;
        }
        if self.device.config.fifo_config.fifo_pwr_config.bits()
            != self.config.fifo_pwr_config.bits()
        {
            self.device
                .interface
                .write_register(self.config.fifo_pwr_config)?;
            self.device.config.fifo_config.fifo_pwr_config = self.config.fifo_pwr_config
        }
        Ok(())
    }
}

#[cfg(feature = "embedded-hal-async")]
impl<'a, Interface, E> FifoConfigBuilder<'a, Interface>
where
    Interface: crate::asynch::WriteToRegister<Error = E>,
{
    /// Write this configuration to device registers
    pub async fn write(self) -> Result<(), E> {
        if self.device.config.fifo_config.fifo_config0.bits() != self.config.fifo_config0.bits() {
            self.device
                .interface
                .write_register(self.config.fifo_config0)
                .await?;
            self.device.config.fifo_config.fifo_config0 = self.config.fifo_config0;
        }
        let wm1_changes =
            self.device.config.fifo_config.fifo_config1.bits() != self.config.fifo_config1.bits();
        let wm2_changes =
            self.device.config.fifo_config.fifo_config2.bits() != self.config.fifo_config2.bits();
        let fifo_wm_changes = wm1_changes || wm2_changes;
        let mut tmp_int_config = self.device.config.int_config.get_config0();

        // If enabled, temporarily disable the FIFO Watermark Interrupt to change the config
        if self.device.config.int_config.get_config0().fwm_int() && fifo_wm_changes {
            tmp_int_config = tmp_int_config.with_fwm_int(false);
            self.device.interface.write_register(tmp_int_config).await?;
        }
        if wm1_changes {
            self.device
                .interface
                .write_register(self.config.fifo_config1)
                .await?;
            self.device.config.fifo_config.fifo_config1 = self.config.fifo_config1;
        }
        if wm2_changes {
            self.device
                .interface
                .write_register(self.config.fifo_config2)
                .await?;
            self.device.config.fifo_config.fifo_config2 = self.config.fifo_config2;
        }
        // Re-enable the interrupt if it was changed
        if self.device.config.int_config.get_config0().bits() != tmp_int_config.bits() {
            self.device
                .interface
                .write_register(self.device.config.int_config.get_config0())
                .await?;
        }
        if self.device.config.fifo_config.fifo_pwr_config.bits()
            != self.config.fifo_pwr_config.bits()
        {
            self.device
                .interface
                .write_register(self.config.fifo_pwr_config)
                .await?;
            self.device.config.fifo_config.fifo_pwr_config = self.config.fifo_pwr_config
        }
        Ok(())
    }
}

impl<'a, Interface> FifoConfigBuilder<'a, Interface> {
    pub(crate) fn new(device: &'a mut BMA400<Interface>) -> FifoConfigBuilder<'a, Interface> {
        FifoConfigBuilder {
            config: device.config.fifo_config.clone(),
            device,
        }
    }
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
        self.config.fifo_config0 = self
            .config
            .fifo_config0
            .with_fifo_x(x_en)
            .with_fifo_y(y_en)
            .with_fifo_z(z_en);
        self
    }
    /// Truncates the 4 least significant bits of the reading to store the measurement of each axis
    /// in a single byte
    pub fn with_8bit_mode(mut self, enabled: bool) -> Self {
        self.config.fifo_config0 = self.config.fifo_config0.with_fifo_8bit(enabled);
        self
    }
    /// Configure FIFO Data Source
    ///
    /// Cannot use [DataSource::AccFilt2Lp]. If passed, this will default to AccFilt2
    pub fn with_src(mut self, src: DataSource) -> Self {
        self.config.fifo_config0 = match src {
            DataSource::AccFilt2Lp => self.config.fifo_config0.with_fifo_src(DataSource::AccFilt2),
            _ => self.config.fifo_config0.with_fifo_src(src),
        };
        self
    }
    /// Enable sending a clock reading if more frames are requested than the buffer contains (>
    /// `get_fifo_len()`)
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
        self.config.fifo_config0 = self
            .config
            .fifo_config0
            .with_flush_on_pwr_mode_change(enabled);
        self
    }

    // FifoConfig1 & FifoConfig2

    /// Set the fill threshold for the FIFO watermark interrupt
    ///
    /// Interupt will be active if FIFO length is > this value
    ///
    /// Clamped to \[0, 1024\] See also [`with_ffull_int`](crate::config::IntConfigBuilder::with_ffull_int)
    pub fn with_watermark_thresh(mut self, threshold: u16) -> Self {
        let thresh = threshold.clamp(0, 1024);
        let bytes = thresh.to_le_bytes();
        self.config.fifo_config1 = self.config.fifo_config1.with_fifo_wtrmk_threshold(bytes[0]);
        self.config.fifo_config2 = self.config.fifo_config2.with_fifo_wtrmk_threshold(bytes[1]);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::get_test_device;
    #[test]
    fn test_read_disabled() {
        let mut device = get_test_device();
        let builder = device.config_fifo();
        let builder = builder.with_read_disabled(true);
        assert_eq!(builder.config.fifo_pwr_config.bits(), 0x01);
        let builder = builder.with_read_disabled(false);
        assert_eq!(builder.config.fifo_pwr_config.bits(), 0x00);
    }
    #[test]
    fn test_axes() {
        let mut device = get_test_device();
        let builder = device.config_fifo();
        let builder = builder.with_axes(true, false, false);
        assert_eq!(builder.config.fifo_config0.bits(), 0x20);
        let builder = builder.with_axes(false, true, false);
        assert_eq!(builder.config.fifo_config0.bits(), 0x40);
        let builder = builder.with_axes(false, false, true);
        assert_eq!(builder.config.fifo_config0.bits(), 0x80);
    }
    #[test]
    fn test_8bit_mode() {
        let mut device = get_test_device();
        let builder = device.config_fifo();
        let builder = builder.with_8bit_mode(true);
        assert_eq!(builder.config.fifo_config0.bits(), 0x10);
        let builder = builder.with_8bit_mode(false);
        assert_eq!(builder.config.fifo_config0.bits(), 0x00);
    }
    #[test]
    fn test_src() {
        let mut device = get_test_device();
        let builder = device.config_fifo();
        let builder = builder.with_src(DataSource::AccFilt2);
        assert_eq!(builder.config.fifo_config0.bits(), 0x08);
        let builder = builder.with_src(DataSource::AccFilt1);
        assert_eq!(builder.config.fifo_config0.bits(), 0x00);
        let builder = builder.with_src(DataSource::AccFilt2Lp);
        assert_eq!(builder.config.fifo_config0.bits(), 0x08);
    }
    #[test]
    fn test_send_time_on_empty() {
        let mut device = get_test_device();
        let builder = device.config_fifo();
        let builder = builder.with_send_time_on_empty(true);
        assert_eq!(builder.config.fifo_config0.bits(), 0x04);
        let builder = builder.with_send_time_on_empty(false);
        assert_eq!(builder.config.fifo_config0.bits(), 0x00);
    }
    #[test]
    fn test_stop_on_full() {
        let mut device = get_test_device();
        let builder = device.config_fifo();
        let builder = builder.with_stop_on_full(true);
        assert_eq!(builder.config.fifo_config0.bits(), 0x02);
        let builder = builder.with_stop_on_full(false);
        assert_eq!(builder.config.fifo_config0.bits(), 0x00);
    }
    #[test]
    fn test_auto_flush() {
        let mut device = get_test_device();
        let builder = device.config_fifo();
        let builder = builder.with_auto_flush(true);
        assert_eq!(builder.config.fifo_config0.bits(), 0x01);
        let builder = builder.with_auto_flush(false);
        assert_eq!(builder.config.fifo_config0.bits(), 0x00);
    }
    #[test]
    fn test_watermark_thresh() {
        let mut device = get_test_device();
        let builder = device.config_fifo();
        let builder = builder.with_watermark_thresh(2048);
        assert_eq!(builder.config.fifo_config1.bits(), 0x00);
        assert_eq!(builder.config.fifo_config2.bits(), 0x04);
        let builder = builder.with_watermark_thresh(1023);
        assert_eq!(builder.config.fifo_config1.bits(), 0xFF);
        assert_eq!(builder.config.fifo_config2.bits(), 0x03);
    }
}
