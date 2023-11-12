#![allow(missing_docs)]

use embedded_hal_async::delay::DelayUs;

use crate::{
    interface::{AsyncReadFromRegister, AsyncWriteToRegister},
    registers::{
        AccXLSB, ChipId, Command, ErrReg, Event, FifoData, FifoLength0, SelfTest, StatusReg,
        StepCount0, StepStatus, TempData,
    },
    AccConfigBuilder, ActChgConfigBuilder, Activity, AutoLpConfigBuilder, AutoWakeupConfigBuilder,
    BMA400Error, Config, ConfigError, FifoConfigBuilder, FifoFrames, GenIntConfigBuilder,
    IntConfigBuilder, IntPinConfigBuilder, IntStatus0, IntStatus1, IntStatus2, Measurement,
    OrientChgConfigBuilder, Status, TapConfigBuilder, WakeupIntConfigBuilder,
};

pub struct AsyncBMA400<T> {
    pub(crate) interface: T,
    pub(crate) config: Config,
}

macro_rules! bma_result {
    ($t:ty) => {
       core::result::Result<$t, BMA400Error<InterfaceError, PinError>>
    };
}

macro_rules! config_async {
    ($fn:ident, $t:ident) => {
        pub fn $fn(&mut self) -> $t<&mut Self> {
            $t::new_async(self)
        }
    };
}

impl<T, InterfaceError, PinError> AsyncBMA400<T>
where
    T: AsyncReadFromRegister<Error = BMA400Error<InterfaceError, PinError>>
        + AsyncWriteToRegister<Error = BMA400Error<InterfaceError, PinError>>,
{
    pub async fn get_id(&mut self) -> bma_result!(u8) {
        let mut id = [0];
        self.interface.read_register(ChipId, &mut id).await?;
        Ok(id[0])
    }

    pub async fn get_cmd_error(&mut self) -> bma_result!(bool) {
        let mut err_byte = [0];
        self.interface.read_register(ErrReg, &mut err_byte).await?;
        Ok(err_byte[0] & 0b10 != 0)
    }

    pub async fn get_status(&mut self) -> bma_result!(Status) {
        let mut status_byte = [0];
        self.interface
            .read_register(StatusReg, &mut status_byte)
            .await?;
        Ok(Status::new(status_byte[0]))
    }

    pub async fn get_unscaled_data(&mut self) -> bma_result!(Measurement) {
        let mut bytes = [0; 6];
        self.interface.read_register(AccXLSB, &mut bytes).await?;
        Ok(Measurement::from_bytes_unscaled(&bytes))
    }

    pub async fn get_data(&mut self) -> bma_result!(Measurement) {
        let mut bytes = [0; 6];
        self.interface.read_register(AccXLSB, &mut bytes).await?;
        Ok(Measurement::from_bytes_scaled(self.config.scale(), &bytes))
    }

    pub async fn get_sensor_clock(&mut self) -> bma_result!(u32) {
        let mut bytes = [0; 3];
        self.interface.read_register(AccXLSB, &mut bytes).await?;
        let bytes = [bytes[0], bytes[1], bytes[2], 0];
        Ok(u32::from_le_bytes(bytes))
    }

    pub async fn get_reset_status(&mut self) -> bma_result!(bool) {
        let mut buffer = [0];
        self.interface.read_register(Event, &mut buffer).await?;
        Ok(buffer[0] & 0x01 != 0)
    }

    pub async fn get_int_status0(&mut self) -> bma_result!(IntStatus0) {
        let mut status_byte = [0];
        self.interface
            .read_register(StatusReg, &mut status_byte)
            .await?;
        Ok(IntStatus0::new(status_byte[0]))
    }

    pub async fn get_int_status1(&mut self) -> bma_result!(IntStatus1) {
        let mut status_byte = [0];
        self.interface
            .read_register(StatusReg, &mut status_byte)
            .await?;
        Ok(IntStatus1::new(status_byte[0]))
    }

    pub async fn get_int_status2(&mut self) -> bma_result!(IntStatus2) {
        let mut status_byte = [0];
        self.interface
            .read_register(StatusReg, &mut status_byte)
            .await?;
        Ok(IntStatus2::new(status_byte[0]))
    }

    pub async fn get_fifo_len(&mut self) -> bma_result!(u16) {
        let mut buffer = [0; 2];
        self.interface
            .read_register(FifoLength0, &mut buffer)
            .await?;
        let bytes = [buffer[0], buffer[1] & 0b0000_0111];
        Ok(u16::from_le_bytes(bytes))
    }

    pub async fn read_fifo_frames<'a>(
        &mut self,
        buffer: &'a mut [u8],
    ) -> bma_result!(FifoFrames<'a>) {
        if self.config.is_fifo_read_disabled() {
            return Err(ConfigError::FifoReadWhilePwrDisable.into());
        }
        self.interface.read_register(FifoData, buffer).await?;
        Ok(FifoFrames::new(buffer))
    }

    pub async fn flush_fifo(&mut self) -> bma_result!(()) {
        self.interface.write_register(Command::FlushFifo).await?;
        Ok(())
    }

    pub async fn get_step_count(&mut self) -> bma_result!(u32) {
        let mut buffer = [0; 3];
        self.interface
            .read_register(StepCount0, &mut buffer)
            .await?;
        Ok(u32::from_le_bytes([buffer[0], buffer[1], buffer[2], 0]))
    }

    pub async fn clear_step_count(&mut self) -> bma_result!(()) {
        self.interface
            .write_register(Command::ClearStepCount)
            .await?;
        Ok(())
    }

    pub async fn get_step_activity(&mut self) -> bma_result!(Activity) {
        let mut buffer = [0];
        self.interface
            .read_register(StepStatus, &mut buffer)
            .await?;
        let activity = match buffer[0] & 0b11 {
            0x00 => Activity::Still,
            0x01 => Activity::Walk,
            _ => Activity::Run,
        };
        Ok(activity)
    }

    pub async fn get_raw_temp(&mut self) -> bma_result!(i8) {
        let mut temp = [0];
        self.interface.read_register(TempData, &mut temp).await?;
        let t = i8::from_le_bytes(temp);
        Ok(t)
    }

    pub async fn get_temp_celcius(&mut self) -> bma_result!(f32) {
        Ok(f32::from(self.get_raw_temp().await?) * 0.5 + 23.0)
    }

    config_async!(config_accel, AccConfigBuilder);
    config_async!(config_interrupts, IntConfigBuilder);
    config_async!(config_int_pins, IntPinConfigBuilder);
    config_async!(config_fifo, FifoConfigBuilder);
    config_async!(config_auto_lp, AutoLpConfigBuilder);
    config_async!(config_autowkup, AutoWakeupConfigBuilder);
    config_async!(config_wkup_int, WakeupIntConfigBuilder);
    config_async!(config_orientchg_int, OrientChgConfigBuilder);

    pub fn config_gen1_int(&mut self) -> GenIntConfigBuilder<&mut Self> {
        GenIntConfigBuilder::new_gen1_async(self)
    }

    pub fn config_gen2_int(&mut self) -> GenIntConfigBuilder<&mut Self> {
        GenIntConfigBuilder::new_gen2_async(self)
    }

    config_async!(config_actchg_int, ActChgConfigBuilder);
    config_async!(config_tap, TapConfigBuilder);

    pub async fn perform_self_test(&mut self, timer: &mut impl DelayUs) -> bma_result!(()) {
        self.config
            .async_setup_self_test(&mut self.interface)
            .await?;
        timer.delay_ms(2).await;

        self.interface
            .write_register(SelfTest::from_bits_truncate(0x07))
            .await?;

        timer.delay_ms(50).await;

        let m_pos = self.get_unscaled_data().await?;

        self.interface
            .write_register(SelfTest::from_bits_truncate(0x0F))
            .await?;

        timer.delay_ms(50).await;

        let m_neg = self.get_unscaled_data().await?;

        // Calculate difference
        let (x, y, z) = (m_pos.x - m_neg.x, m_pos.y - m_neg.y, m_pos.z - m_neg.z);

        // Disable self test
        self.interface.write_register(SelfTest::default()).await?;

        // Wait 50ms
        timer.delay_ms(50).await;

        // Re-enable interrupts and previous config
        self.config
            .async_cleanup_self_test(&mut self.interface)
            .await?;

        // Evaluate results
        if x > 1500 && y > 1200 && z > 250 {
            Ok(())
        } else {
            Err(BMA400Error::SelfTestFailedError)
        }
    }

    pub async fn soft_reset(&mut self) -> bma_result!(()) {
        self.interface.write_register(Command::SoftReset).await?;
        self.config = Config::default();
        let mut buffer = [0];
        self.interface.read_register(Event, &mut buffer).await?;
        Ok(())
    }
}

impl<T> AsyncBMA400<T> {
    pub fn destroy(self) -> T {
        self.interface
    }
}
