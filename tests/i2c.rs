use embedded_hal_mock::{
    i2c::{Mock, Transaction},
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
