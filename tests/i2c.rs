use embedded_hal_mock::{
    i2c::{Mock, Transaction}, delay::MockNoop,
};
use bma400::i2c::I2CInterface;
use bma400::{BMA400, types::*};

#[cfg(feature = "i2c-default")]
pub const ADDR: u8 = 0b00010100;
#[cfg(feature = "i2c-alt")]
pub const ADDR: u8 = 0b00010101;

fn new(expected: &[Transaction]) -> BMA400<I2CInterface<Mock>> {
    BMA400::new_i2c(Mock::new(expected)).unwrap()
}

#[test]
fn init_bad_chip_id() {
    let mut expected = Vec::new();
    expected.push(Transaction::write_read(ADDR, vec![0x00], vec![0x89]));
    let result = BMA400::new_i2c(Mock::new(&expected));
    assert!(matches!(result, Err(BMA400Error::ChipIdReadFailed)));
}

#[test]
fn get_chip_id() {
    let mut expected = Vec::new();
    expected.push(Transaction::write_read(ADDR, vec![0x00], vec![0x90]));
    expected.push(Transaction::write_read(ADDR, vec![0x00], vec![0x90]));
    let mut device = new(&expected);
    let id = device.get_id().unwrap();
    assert_eq!(id, 0x90);
}

#[test]
fn get_cmd_error() {
    let mut expected = Vec::new();
    expected.push(Transaction::write_read(ADDR, vec![0x00], vec![0x90]));
    expected.push(Transaction::write_read(ADDR, vec![0x02], vec![0xFD]));
    expected.push(Transaction::write_read(ADDR, vec![0x02], vec![0x02]));
    let mut device = new(&expected);
    let status = device.get_cmd_error().unwrap();
    assert!(!status);
    let status = device.get_cmd_error().unwrap();
    assert!(status);
}

#[test]
fn get_status() {
    let mut expected = Vec::new();

    expected.push(Transaction::write_read(ADDR, vec![0x00], vec![0x90]));

    // drdy Set
    expected.push(Transaction::write_read(ADDR, vec![0x03], vec![0x80]));

    // cmd_rdy Set
    expected.push(Transaction::write_read(ADDR, vec![0x03], vec![0x10]));

    // power_mode == LowPower
    expected.push(Transaction::write_read(ADDR, vec![0x03], vec![0x02]));

    // power_mode == Normal
    expected.push(Transaction::write_read(ADDR, vec![0x03], vec![0x04]));

    // interrupt triggered Set
    expected.push(Transaction::write_read(ADDR, vec![0x03], vec![0x01]));

    let mut device = new(&expected);

    // drdy
    let status = device.get_status().unwrap();
    assert!(status.drdy_stat());
    assert!(!status.cmd_rdy());
    assert!(!status.int_active());
    assert!(matches!(status.power_mode(), PowerMode::Sleep));

    // cmd_rdy
    let status = device.get_status().unwrap();
    assert!(!status.drdy_stat());
    assert!(status.cmd_rdy());
    assert!(!status.int_active());
    assert!(matches!(status.power_mode(), PowerMode::Sleep));

    // power_mode == LowPower
    let status = device.get_status().unwrap();
    assert!(!status.drdy_stat());
    assert!(!status.cmd_rdy());
    assert!(!status.int_active());
    assert!(matches!(status.power_mode(), PowerMode::LowPower));
    
    // power_mode == Normal
    let status = device.get_status().unwrap();
    assert!(!status.drdy_stat());
    assert!(!status.cmd_rdy());
    assert!(!status.int_active());
    assert!(matches!(status.power_mode(), PowerMode::Normal));

    // interrupt triggered
    let status = device.get_status().unwrap();
    assert!(!status.drdy_stat());
    assert!(!status.cmd_rdy());
    assert!(status.int_active());
    assert!(matches!(status.power_mode(), PowerMode::Sleep));
}

#[test]
fn get_unscaled_data() {
    let mut expected = Vec::new();
    expected.push(Transaction::write_read(ADDR, vec![0x00], vec![0x90]));
    expected.push(Transaction::write_read(ADDR, vec![0x04], vec![0x01, 0x08, 0xFF, 0x0F, 0xFF, 0x07]));
    let mut device = new(&expected);
    let m = device.get_unscaled_data().unwrap();
    assert_eq!(m.x, -2047);
    assert_eq!(m.y, -1);
    assert_eq!(m.z, 2047);
}

fn get_scaled_data(scale: Scale) -> (i16, i16, i16) {
    let byte = match scale {
        Scale::Range2G => 0x09,
        Scale::Range4G => 0x49,
        Scale::Range8G => 0x89,
        Scale::Range16G => 0xC9,
    };
    let mut expected = Vec::new();
    expected.push(Transaction::write_read(ADDR, vec![0x00], vec![0x90]));
    if let Scale::Range4G = scale {
        // The default setting is 4G so we shouldn't see any configuration write
        expected.push(Transaction::write_read(ADDR, vec![0x04], vec![0x01, 0x08, 0xFF, 0x0F, 0xFF, 0x07]));
    } else {
        expected.push(Transaction::write(ADDR, vec![0x1A, byte]));
        expected.push(Transaction::write_read(ADDR, vec![0x04], vec![0x01, 0x08, 0xFF, 0x0F, 0xFF, 0x07]));
    }
    let mut device = new(&expected);
    device.config_accel().with_scale(scale).write().unwrap();
    let m = device.get_data().unwrap();
    (m.x, m.y, m.z)
}

#[test]
fn get_data() {
    assert_eq!((-2047, -1, 2047), get_scaled_data(Scale::Range2G));
    assert_eq!((-4094, -2, 4094), get_scaled_data(Scale::Range4G));
    assert_eq!((-8188, -4, 8188), get_scaled_data(Scale::Range8G));
    assert_eq!((-16376, -8, 16376), get_scaled_data(Scale::Range16G));
}

#[test]
fn get_sensor_clock() {
    let mut expected = Vec::new();
    expected.push(Transaction::write_read(ADDR, vec![0x00], vec![0x90]));
    expected.push(Transaction::write_read(ADDR, vec![0x0A], vec![0xF8, 0xFF, 0xFF]));
    let mut device = new(&expected);
    let t = device.get_sensor_clock().unwrap();
    assert_eq!(t, 0xFFFFF8);
}

#[test]
fn get_reset_status() {
    let mut expected = Vec::new();
    expected.push(Transaction::write_read(ADDR, vec![0x00], vec![0x90]));

    // No Reset Detected
    expected.push(Transaction::write_read(ADDR, vec![0x0D], vec![0x00]));

    // Reset Detected
    expected.push(Transaction::write_read(ADDR, vec![0x0D], vec![0x01]));
    let mut device = new(&expected);

    let reset = device.get_reset_status().unwrap();
    assert!(!reset);
    let reset = device.get_reset_status().unwrap();
    assert!(reset);
}

#[test]
fn get_int_status() {
    let mut expected = Vec::new();
    expected.push(Transaction::write_read(ADDR, vec![0x00], vec![0x90]));
    
    // drdy set
    expected.push(Transaction::write_read(ADDR, vec![0x0E], vec![0x80]));
    
    // fwm set
    expected.push(Transaction::write_read(ADDR, vec![0x0E], vec![0x40]));

    // ffull set
    expected.push(Transaction::write_read(ADDR, vec![0x0E], vec![0x20]));

    // ieng_ovrrn set
    expected.push(Transaction::write_read(ADDR, vec![0x0E], vec![0x10]));

    // gen2 set
    expected.push(Transaction::write_read(ADDR, vec![0x0E], vec![0x08]));

    // gen1 set
    expected.push(Transaction::write_read(ADDR, vec![0x0E], vec![0x04]));

    // orientch set
    expected.push(Transaction::write_read(ADDR, vec![0x0E], vec![0x02]));

    // wkup set
    expected.push(Transaction::write_read(ADDR, vec![0x0E], vec![0x01]));

    let mut device = new(&expected);

    // drdy
    let status = device.get_int_status0().unwrap();
    assert!(status.drdy_stat());
    assert!(!status.fwm_stat());
    assert!(!status.ffull_stat());
    assert!(!status.ieng_overrun_stat());
    assert!(!status.gen2_stat());
    assert!(!status.gen1_stat());
    assert!(!status.orientch_stat());
    assert!(!status.wkup_stat());

    // fwm
    let status = device.get_int_status0().unwrap();
    assert!(!status.drdy_stat());
    assert!(status.fwm_stat());
    assert!(!status.ffull_stat());
    assert!(!status.ieng_overrun_stat());
    assert!(!status.gen2_stat());
    assert!(!status.gen1_stat());
    assert!(!status.orientch_stat());
    assert!(!status.wkup_stat());

    // ffull
    let status = device.get_int_status0().unwrap();
    assert!(!status.drdy_stat());
    assert!(!status.fwm_stat());
    assert!(status.ffull_stat());
    assert!(!status.ieng_overrun_stat());
    assert!(!status.gen2_stat());
    assert!(!status.gen1_stat());
    assert!(!status.orientch_stat());
    assert!(!status.wkup_stat());

    // ieng_ovrrn
    let status = device.get_int_status0().unwrap();
    assert!(!status.drdy_stat());
    assert!(!status.fwm_stat());
    assert!(!status.ffull_stat());
    assert!(status.ieng_overrun_stat());
    assert!(!status.gen2_stat());
    assert!(!status.gen1_stat());
    assert!(!status.orientch_stat());
    assert!(!status.wkup_stat());

    // gen2
    let status = device.get_int_status0().unwrap();
    assert!(!status.drdy_stat());
    assert!(!status.fwm_stat());
    assert!(!status.ffull_stat());
    assert!(!status.ieng_overrun_stat());
    assert!(status.gen2_stat());
    assert!(!status.gen1_stat());
    assert!(!status.orientch_stat());
    assert!(!status.wkup_stat());

    // gen1
    let status = device.get_int_status0().unwrap();
    assert!(!status.drdy_stat());
    assert!(!status.fwm_stat());
    assert!(!status.ffull_stat());
    assert!(!status.ieng_overrun_stat());
    assert!(!status.gen2_stat());
    assert!(status.gen1_stat());
    assert!(!status.orientch_stat());
    assert!(!status.wkup_stat());

    // orientch
    let status = device.get_int_status0().unwrap();
    assert!(!status.drdy_stat());
    assert!(!status.fwm_stat());
    assert!(!status.ffull_stat());
    assert!(!status.ieng_overrun_stat());
    assert!(!status.gen2_stat());
    assert!(!status.gen1_stat());
    assert!(status.orientch_stat());
    assert!(!status.wkup_stat());

    // wkup
    let status = device.get_int_status0().unwrap();
    assert!(!status.drdy_stat());
    assert!(!status.fwm_stat());
    assert!(!status.ffull_stat());
    assert!(!status.ieng_overrun_stat());
    assert!(!status.gen2_stat());
    assert!(!status.gen1_stat());
    assert!(!status.orientch_stat());
    assert!(status.wkup_stat());
}

#[test]
fn get_int_status1() {
    let mut expected = Vec::new();
    expected.push(Transaction::write_read(ADDR, vec![0x00], vec![0x90]));

    // ieng_ovrrn set
    expected.push(Transaction::write_read(ADDR, vec![0x0F], vec![0x10]));

    // d_tap set
    expected.push(Transaction::write_read(ADDR, vec![0x0F], vec![0x08]));

    // s_tap set
    expected.push(Transaction::write_read(ADDR, vec![0x0F], vec![0x04]));

    // step_int == 2
    expected.push(Transaction::write_read(ADDR, vec![0x0F], vec![0x02]));

    // step_int == 1
    expected.push(Transaction::write_read(ADDR, vec![0x0F], vec![0x01]));

    let mut device = new(&expected);

    // ieng_ovrrn
    let status = device.get_int_status1().unwrap();
    assert!(status.ieng_overrun_stat());
    assert!(!status.d_tap_stat());
    assert!(!status.s_tap_stat());
    assert!(matches!(status.step_int_stat(), StepIntStatus::None));

    // d_tap
    let status = device.get_int_status1().unwrap();
    assert!(!status.ieng_overrun_stat());
    assert!(status.d_tap_stat());
    assert!(!status.s_tap_stat());
    assert!(matches!(status.step_int_stat(), StepIntStatus::None));

    // s_tap
    let status = device.get_int_status1().unwrap();
    assert!(!status.ieng_overrun_stat());
    assert!(!status.d_tap_stat());
    assert!(status.s_tap_stat());
    assert!(matches!(status.step_int_stat(), StepIntStatus::None));

    // step_int == 2
    let status = device.get_int_status1().unwrap();
    assert!(!status.ieng_overrun_stat());
    assert!(!status.d_tap_stat());
    assert!(!status.s_tap_stat());
    assert!(matches!(status.step_int_stat(), StepIntStatus::ManyStepDetect));

    // step_int == 1
    let status = device.get_int_status1().unwrap();
    assert!(!status.ieng_overrun_stat());
    assert!(!status.d_tap_stat());
    assert!(!status.s_tap_stat());
    assert!(matches!(status.step_int_stat(), StepIntStatus::OneStepDetect));
}

#[test]
fn get_int_status2() {
    let mut expected = Vec::new();
    expected.push(Transaction::write_read(ADDR, vec![0x00], vec![0x90]));

    // ieng_ovrrn set
    expected.push(Transaction::write_read(ADDR, vec![0x10], vec![0x10]));

    // actch_z set
    expected.push(Transaction::write_read(ADDR, vec![0x10], vec![0x04]));

    // actch_y set
    expected.push(Transaction::write_read(ADDR, vec![0x10], vec![0x02]));

    // actch_x set
    expected.push(Transaction::write_read(ADDR, vec![0x10], vec![0x01]));
    
    let mut device = new(&expected);

    // ieng_ovrrn
    let status = device.get_int_status2().unwrap();
    assert!(status.ieng_overrun_stat());
    assert!(!status.actch_z_stat());
    assert!(!status.actch_y_stat());
    assert!(!status.actch_x_stat());

    // actch_z
    let status = device.get_int_status2().unwrap();
    assert!(!status.ieng_overrun_stat());
    assert!(status.actch_z_stat());
    assert!(!status.actch_y_stat());
    assert!(!status.actch_x_stat());

    // actch_y
    let status = device.get_int_status2().unwrap();
    assert!(!status.ieng_overrun_stat());
    assert!(!status.actch_z_stat());
    assert!(status.actch_y_stat());
    assert!(!status.actch_x_stat());

    // actch_x
    let status = device.get_int_status2().unwrap();
    assert!(!status.ieng_overrun_stat());
    assert!(!status.actch_z_stat());
    assert!(!status.actch_y_stat());
    assert!(status.actch_x_stat());
}

#[test]
fn get_fifo_len() {
    let mut expected = Vec::new();
    expected.push(Transaction::write_read(ADDR, vec![0x00], vec![0x90]));
    expected.push(Transaction::write_read(ADDR,vec![0x12], vec![0x00, 0xF4]));
    expected.push(Transaction::write_read(ADDR, vec![0x12], vec![0x80, 0x02]));

    let mut device = new(&expected);
    let len = device.get_fifo_len().unwrap();
    assert_eq!(len, 1024);
    let len = device.get_fifo_len().unwrap();
    assert_eq!(len, 640);
}

#[test]
fn read_fifo_frames() {
    let mut expected = Vec::new();
    expected.push(Transaction::write_read(ADDR, vec![0x00], vec![0x90]));
    expected.push(Transaction::write_read(ADDR, vec![0x14], vec![0x48, 0x6E, 0x9E, 0x01, 0x80, 0x0F, 0xFF, 0x0F, 0x7F, 0xA0, 0xF8, 0xFF, 0xFF, 0x80, 0x00]));
    let mut device = new(&expected);
    let mut buffer = [0u8; 15];
    let frames = device.read_fifo_frames(&mut buffer).unwrap();
    let mut count = 0;
    for frame in frames {
        match frame.frame_type() {
            FrameType::Data => {
                assert_eq!(frame.x(), -2047);
                assert_eq!(frame.y(), -1);
                assert_eq!(frame.z(), 2047);
            },
            FrameType::Time => {
                assert_eq!(frame.time(),  0xFFFFF8);
            },
            FrameType::Control => {
                assert!(frame.fifo_chg());
                assert!(frame.acc0_chg());
                assert!(frame.acc1_chg());
            }
        }
        count +=1;
    }
    assert_eq!(count, 3);
}

#[test]
fn flush_fifo() {
    let mut expected = Vec::new();
    expected.push(Transaction::write_read(ADDR, vec![0x00], vec![0x90]));

    expected.push(Transaction::write(ADDR, vec![0x7E, 0xB0]));

    let mut device = new(&expected);
    device.flush_fifo().unwrap();
}

#[test]
fn get_step_count() {
    let mut expected = Vec::new();
    expected.push(Transaction::write_read(ADDR, vec![0x00], vec![0x90]));
    
    expected.push(Transaction::write_read(ADDR, vec![0x15], vec![0x00, 0xFF, 0xF0]));
    let mut device = new(&expected);
    let count = device.get_step_count().unwrap();
    assert_eq!(count, 15793920);
}

#[test]
fn clear_step_count() {
    let mut expected = Vec::new();
    expected.push(Transaction::write_read(ADDR, vec![0x00], vec![0x90]));

    expected.push(Transaction::write(ADDR, vec![0x7E, 0xB1]));
    let mut device = new(&expected);
    device.clear_step_count().unwrap();
}

#[test]
fn get_raw_temp() {
    let mut expected = Vec::new();
    expected.push(Transaction::write_read(ADDR, vec![0x00], vec![0x90]));

    expected.push(Transaction::write_read(ADDR, vec![0x11], vec![0xD0]));
    expected.push(Transaction::write_read(ADDR, vec![0x11], vec![0x7F]));
    let mut device = new(&expected);
    let temp = device.get_raw_temp().unwrap();
    assert_eq!(temp, -48);
    let temp = device.get_raw_temp().unwrap();
    assert_eq!(temp, 127);
}

#[test]
fn config_accel() {
    let mut expected = Vec::new();
    expected.push(Transaction::write_read(ADDR, vec![0x00], vec![0x90]));

    expected.push(Transaction::write(ADDR, vec![0x19, 0xE2]));
    expected.push(Transaction::write(ADDR, vec![0x1A, 0xFB]));
    expected.push(Transaction::write(ADDR, vec![0x1B, 0x08]));

    expected.push(Transaction::write(ADDR, vec![0x19, 0x00]));
    expected.push(Transaction::write(ADDR, vec![0x1A, 0x05]));
    expected.push(Transaction::write(ADDR, vec![0x1B, 0x00]));

    let mut device = new(&expected);
    
    // Set Everything
    device.config_accel()
    .with_filt1_bw(Filter1Bandwidth::Low)
    .with_osr_lp(OversampleRate::OSR3)
    .with_power_mode(PowerMode::Normal)
    .with_scale(Scale::Range16G)
    .with_osr(OversampleRate::OSR3)
    .with_odr(OutputDataRate::Hz800)
    .with_reg_dta_src(DataSource::AccFilt2Lp).write().unwrap();

    // Un-Set Everything
    device.config_accel()
    .with_filt1_bw(Filter1Bandwidth::High)
    .with_osr_lp(OversampleRate::OSR0)
    .with_power_mode(PowerMode::Sleep)
    .with_scale(Scale::Range2G)
    .with_osr(OversampleRate::OSR0)
    .with_odr(OutputDataRate::Hz12_5)
    .with_reg_dta_src(DataSource::AccFilt1).write().unwrap();
}

#[test]
fn config_interrupts() {
    let mut expected = Vec::new();
    expected.push(Transaction::write_read(ADDR, vec![0x00], vec![0x90]));

    expected.push(Transaction::write(ADDR, vec![0x56, 0x10]));
    expected.push(Transaction::write(ADDR, vec![0x1F, 0xEE]));
    expected.push(Transaction::write(ADDR, vec![0x20, 0x9D]));

    expected.push(Transaction::write(ADDR, vec![0x1F, 0x00]));
    expected.push(Transaction::write(ADDR, vec![0x20, 0x00]));

    let mut device = new(&expected);

    // Set Activity Change to use AccFilt2 so we can enable it
    device.config_actchg_int()
    .with_src(DataSource::AccFilt2).write().unwrap();

    // Set Everything
    device.config_interrupts()
    .with_actch_int(true)
    .with_d_tap_int(true)
    .with_dta_rdy_int(true)
    .with_ffull_int(true)
    .with_fwm_int(true)
    .with_gen1_int(true)
    .with_gen2_int(true)
    .with_latch_int(true)
    .with_orientch_int(true)
    .with_s_tap_int(true)
    .with_step_int(true).write().unwrap();

    // Un-Set Everything
    device.config_interrupts()
    .with_actch_int(false)
    .with_d_tap_int(false)
    .with_dta_rdy_int(false)
    .with_ffull_int(false)
    .with_fwm_int(false)
    .with_gen1_int(false)
    .with_gen2_int(false)
    .with_latch_int(false)
    .with_orientch_int(false)
    .with_s_tap_int(false)
    .with_step_int(false).write().unwrap();
}

#[test]
fn config_int_pins() {
    let mut expected = Vec::new();
    expected.push(Transaction::write_read(ADDR, vec![0x00], vec![0x90]));

    expected.push(Transaction::write(ADDR, vec![0x21, 0xFF]));
    expected.push(Transaction::write(ADDR, vec![0x22, 0xFF]));
    expected.push(Transaction::write(ADDR, vec![0x23, 0xDD]));
    expected.push(Transaction::write(ADDR, vec![0x24, 0x66]));

    expected.push(Transaction::write(ADDR, vec![0x21, 0x00]));
    expected.push(Transaction::write(ADDR, vec![0x23, 0xD0]));
    expected.push(Transaction::write(ADDR, vec![0x24, 0x60]));

    expected.push(Transaction::write(ADDR, vec![0x22, 0x00]));
    expected.push(Transaction::write(ADDR, vec![0x23, 0x00]));
    expected.push(Transaction::write(ADDR, vec![0x24, 0x00]));

    let mut device = new(&expected);

    // Set Everything
    device.config_int_pins()
    .with_drdy(InterruptPins::Both)
    .with_fifo_wm(InterruptPins::Both)
    .with_ffull(InterruptPins::Both)
    .with_ieng_ovrrn(InterruptPins::Both)
    .with_gen2(InterruptPins::Both)
    .with_gen1(InterruptPins::Both)
    .with_orientch(InterruptPins::Both)
    .with_wkup(InterruptPins::Both)
    .with_actch(InterruptPins::Both)
    .with_tap(InterruptPins::Both)
    .with_step(InterruptPins::Both)
    .with_int1_cfg(PinOutputConfig::OpenDrain(PinOutputLevel::ActiveHigh))
    .with_int2_cfg(PinOutputConfig::OpenDrain(PinOutputLevel::ActiveHigh))
    .write().unwrap();

    // Un-Set Pin1
    device.config_int_pins()
    .with_drdy(InterruptPins::Int2)
    .with_fifo_wm(InterruptPins::Int2)
    .with_ffull(InterruptPins::Int2)
    .with_ieng_ovrrn(InterruptPins::Int2)
    .with_gen2(InterruptPins::Int2)
    .with_gen1(InterruptPins::Int2)
    .with_orientch(InterruptPins::Int2)
    .with_wkup(InterruptPins::Int2)
    .with_actch(InterruptPins::Int2)
    .with_tap(InterruptPins::Int2)
    .with_step(InterruptPins::Int2)
    .with_int1_cfg(PinOutputConfig::PushPull(PinOutputLevel::ActiveLow))
    .write().unwrap();

    // Un-Set Pin2
    device.config_int_pins()
    .with_drdy(InterruptPins::None)
    .with_fifo_wm(InterruptPins::None)
    .with_ffull(InterruptPins::None)
    .with_ieng_ovrrn(InterruptPins::None)
    .with_gen2(InterruptPins::None)
    .with_gen1(InterruptPins::None)
    .with_orientch(InterruptPins::None)
    .with_wkup(InterruptPins::None)
    .with_actch(InterruptPins::None)
    .with_tap(InterruptPins::None)
    .with_step(InterruptPins::None)
    .with_int2_cfg(PinOutputConfig::PushPull(PinOutputLevel::ActiveLow))
    .write().unwrap();
}

#[test]
fn config_fifo() {
    let mut expected = Vec::new();
    expected.push(Transaction::write_read(ADDR, vec![0x00], vec![0x90]));

    expected.push(Transaction::write(ADDR, vec![0x26, 0xFF]));
    expected.push(Transaction::write(ADDR, vec![0x27, 0xFF]));
    expected.push(Transaction::write(ADDR, vec![0x28, 0x03]));
    expected.push(Transaction::write(ADDR, vec![0x29, 0x01]));

    expected.push(Transaction::write(ADDR, vec![0x26, 0x00]));
    expected.push(Transaction::write(ADDR, vec![0x27, 0x00]));
    expected.push(Transaction::write(ADDR, vec![0x28, 0x00]));
    expected.push(Transaction::write(ADDR, vec![0x29, 0x00]));

    let mut device = new(&expected);

    // Set Everything
    device.config_fifo()
    .with_8bit_mode(true)
    .with_axes(true, true, true)
    .with_src(DataSource::AccFilt2)
    .with_send_time_on_empty(true)
    .with_stop_on_full(true)
    .with_auto_flush(true)
    .with_watermark_thresh(1023)
    .with_read_disabled(true)
    .write().unwrap();

    // Un-Set Everything
    device.config_fifo()
    .with_8bit_mode(false)
    .with_axes(false, false, false)
    .with_src(DataSource::AccFilt1)
    .with_send_time_on_empty(false)
    .with_stop_on_full(false)
    .with_auto_flush(false)
    .with_watermark_thresh(0)
    .with_read_disabled(false)
    .write().unwrap();
}

#[test]
fn config_auto_lp() {
    let mut expected = Vec::new();
    expected.push(Transaction::write_read(ADDR, vec![0x00], vec![0x90]));

    expected.push(Transaction::write(ADDR, vec![0x2A, 0xFF]));
    expected.push(Transaction::write(ADDR, vec![0x2B, 0xFB]));

    expected.push(Transaction::write(ADDR, vec![0x2A, 0x00]));
    expected.push(Transaction::write(ADDR, vec![0x2B, 0x00]));

    let mut device = new(&expected);

    // Set Everything
    device.config_auto_lp()
    .with_timeout(0xFFF)
    .with_auto_lp_trigger(AutoLPTimeoutTrigger::TimeoutEnabledGen2IntReset)
    .with_drdy_trigger(true)
    .with_gen1_int_trigger(true)
    .write().unwrap();

    // Un-Set Everything
    device.config_auto_lp()
    .with_timeout(0)
    .with_auto_lp_trigger(AutoLPTimeoutTrigger::TimeoutDisabled)
    .with_drdy_trigger(false)
    .with_gen1_int_trigger(false)
    .write().unwrap();
}

#[test]
fn config_autowkup() {
    let mut expected = Vec::new();
    expected.push(Transaction::write_read(ADDR, vec![0x00], vec![0x90]));

    expected.push(Transaction::write(ADDR, vec![0x2C, 0xFF]));
    expected.push(Transaction::write(ADDR, vec![0x2D, 0xF6]));

    expected.push(Transaction::write(ADDR, vec![0x2C, 0x00]));
    expected.push(Transaction::write(ADDR, vec![0x2D, 0x00]));

    let mut device = new(&expected);

    // Set Everything
    device.config_autowkup()
    .with_wakeup_period(0xFFF)
    .with_periodic_wakeup(true)
    .with_activity_int(true)
    .write().unwrap();

    // Un-Set Everything
    device.config_autowkup()
    .with_wakeup_period(0)
    .with_periodic_wakeup(false)
    .with_activity_int(false)
    .write().unwrap();
}

fn self_test_setup(expected: &mut Vec<Transaction>) {
    // Disable Interrupts
    expected.push(Transaction::write(ADDR, vec![0x1F, 0x00]));
    expected.push(Transaction::write(ADDR, vec![0x20, 0x00]));
    expected.push(Transaction::write(ADDR, vec![0x2D, 0xF4]));

    // Disable FIFO
    expected.push(Transaction::write(ADDR, vec![0x26, 0x1F]));

    // Set PowerMode
    expected.push(Transaction::write(ADDR, vec![0x19, 0xE2]));

    // Set Range = 4G, OSR = OSR3, ODR = 100Hz
    expected.push(Transaction::write(ADDR, vec![0x1A, 0x78]));
}

fn restore_config(expected: &mut Vec<Transaction>) {
    // Restore AccConfig0
    expected.push(Transaction::write(ADDR, vec![0x19, 0xE0]));

    // Restore AccConfig1
    expected.push(Transaction::write(ADDR, vec![0x1A, 0x09]));

    // Restore IntConfig0
    expected.push(Transaction::write(ADDR, vec![0x1F, 0xEE]));

    // Restore IntConfig1
    expected.push(Transaction::write(ADDR, vec![0x20, 0x9D]));

    // Restore AutoWkupConfig1
    expected.push(Transaction::write(ADDR, vec![0x2D, 0xF6]));

    // Restore FifoConfig
    expected.push(Transaction::write(ADDR, vec![0x26, 0xFF]));
}

fn self_test(x_fail: bool, y_fail: bool, z_fail: bool, expected: &mut Vec<Transaction>) {
    
    const PASS_X_POS: i16 = 767;
    const PASS_X_NEG: i16 = -734;
    const PASS_Y_POS: i16 = 401;
    const PASS_Y_NEG: i16 = -800;
    const PASS_Z_POS: i16 = 550;
    const PASS_Z_NEG: i16 = 299;

    const FAIL_X_NEG: i16 = -733;
    const FAIL_Y_POS: i16 = 400;
    const FAIL_Z_NEG: i16 = 300;

    let x_pos = PASS_X_POS;
    let x_neg = if x_fail {FAIL_X_NEG} else {PASS_X_NEG};
    let y_pos = if y_fail {FAIL_Y_POS} else {PASS_Y_POS};
    let y_neg = PASS_Y_NEG;
    let z_pos = PASS_Z_POS;
    let z_neg = if z_fail {FAIL_Z_NEG} else {PASS_Z_NEG};

    //Disable Interrupts, Set Test Config
    self_test_setup(expected);

    // Set Positive Test Parameters
    expected.push(Transaction::write(ADDR, vec![0x7D, 0x07]));

    // Read Results
    expected.push(Transaction::write_read(ADDR, vec![0x04], vec![x_pos.to_le_bytes()[0], x_pos.to_le_bytes()[1], y_pos.to_le_bytes()[0], y_pos.to_le_bytes()[1], z_pos.to_le_bytes()[0], z_pos.to_le_bytes()[1]]));

    // Write Negative Test Parameters
    expected.push(Transaction::write(ADDR, vec![0x7D, 0x0F]));

    // Read Results
    expected.push(Transaction::write_read(ADDR, vec![0x04], vec![x_neg.to_le_bytes()[0], x_neg.to_le_bytes()[1], y_neg.to_le_bytes()[0], y_neg.to_le_bytes()[1], z_neg.to_le_bytes()[0], z_neg.to_le_bytes()[1]]));

    // Disable Self-Test
    expected.push(Transaction::write(ADDR, vec![0x7D, 0x00]));

    restore_config(expected);
}

#[test]
fn perform_self_test() {
    let mut expected = Vec::new();
    expected.push(Transaction::write_read(ADDR, vec![0x00], vec![0x90]));

    // Enable all interrupts, fifo, etc to
    // test restoring configuration post-test

    // Actch Int Data Src = AccFilt2
    expected.push(Transaction::write(ADDR, vec![0x56, 0x10]));
    
    // Set all non-power mode settings in AccConfig0
    expected.push(Transaction::write(ADDR, vec![0x19, 0xE0]));

    // Set Range = 2G, OSR = OSR0, ODR = 200Hz
    expected.push(Transaction::write(ADDR, vec![0x1A, 0x09]));

    // Set IntConfig0
    expected.push(Transaction::write(ADDR, vec![0x1F, 0xEE]));

    // Set IntConfig1
    expected.push(Transaction::write(ADDR, vec![0x20, 0x9D]));

    // Set Wakeup Int, Settings
    expected.push(Transaction::write(ADDR, vec![0x2D, 0xF6]));

    // Enable FIFO, Settings
    expected.push(Transaction::write(ADDR, vec![0x26, 0xFF]));

    self_test(false, false, false, &mut expected);
    self_test(true, false, false, &mut expected);
    self_test(false, true, false, &mut expected);
    self_test(false, false, true, &mut expected);

    let mut device = new(&expected);

    // ActChgConfig
    device.config_actchg_int()
    .with_src(DataSource::AccFilt2).write().unwrap();

    // AccConfig
    device.config_accel()
    .with_filt1_bw(Filter1Bandwidth::Low)
    .with_osr_lp(OversampleRate::OSR3)
    .with_scale(Scale::Range2G)
    .with_osr(OversampleRate::OSR0)
    .with_odr(OutputDataRate::Hz200).write().unwrap();

    // IntConfig
    device.config_interrupts()
    .with_actch_int(true)
    .with_d_tap_int(true)
    .with_dta_rdy_int(true)
    .with_ffull_int(true)
    .with_fwm_int(true)
    .with_gen1_int(true)
    .with_gen2_int(true)
    .with_latch_int(true)
    .with_orientch_int(true)
    .with_s_tap_int(true)
    .with_step_int(true).write().unwrap();

    // Wakeup Int
    device.config_autowkup()
    .with_periodic_wakeup(true)
    .with_wakeup_period(0x0F)
    .with_activity_int(true).write().unwrap();

    // FIFO
    device.config_fifo()
    .with_axes(true, true, true)
    .with_8bit_mode(true)
    .with_src(DataSource::AccFilt2)
    .with_send_time_on_empty(true)
    .with_stop_on_full(true)
    .with_auto_flush(true).write().unwrap();

    let mut timer = MockNoop::new();

    // Pass
    let result = device.perform_self_test(&mut timer);
    assert!(matches!(result, Ok(())));

    // Fail X
    let result = device.perform_self_test(&mut timer);
    assert!(matches!(result, Err(BMA400Error::SelfTestFailedError)));

    // Fail Y
    let result = device.perform_self_test(&mut timer);
    assert!(matches!(result, Err(BMA400Error::SelfTestFailedError)));

    // Fail Z
    let result = device.perform_self_test(&mut timer);
    assert!(matches!(result, Err(BMA400Error::SelfTestFailedError)));
}

#[test]
fn soft_reset() {
    let mut expected = Vec::new();
    expected.push(Transaction::write_read(ADDR, vec![0x00], vec![0x90]));

    expected.push(Transaction::write(ADDR, vec![0x7E, 0xB6]));
    expected.push(Transaction::write_read(ADDR, vec![0x0D], vec![0x01]));

    let mut device = new(&expected);
    device.soft_reset().unwrap();
}
