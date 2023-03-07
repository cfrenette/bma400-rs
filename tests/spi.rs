use embedded_hal_mock::{
    spi::{Mock as MockSPI, Transaction},
    pin::{Mock as MockPin, State, Transaction as PinTransaction},
};
use bma400::spi::SPIInterface;
use bma400::{BMA400, types::*};

fn new(expected_io: &[Transaction], expected_pin: &[PinTransaction]) -> BMA400<SPIInterface<MockSPI, MockPin>> {
    BMA400::new_spi(MockSPI::new(expected_io), MockPin::new(expected_pin)).unwrap()
}

#[test]
fn init_bad_chip_id() {
    let mut expected_io = Vec::new();
    let mut expected_pin = Vec::new();
    expected_pin.push(PinTransaction::set(State::Low));
    expected_io.push(Transaction::transfer(vec![0x80, 0x00], vec![0x00, 0x00]));
    expected_io.push(Transaction::transfer(vec![0x00], vec![0x00]));
    expected_pin.push(PinTransaction::set(State::High));
    expected_pin.push(PinTransaction::set(State::Low));
    expected_io.push(Transaction::transfer(vec![0x80, 0x00], vec![0x00, 0x00]));
    expected_io.push(Transaction::transfer(vec![0x00], vec![0x89]));
    expected_pin.push(PinTransaction::set(State::High));
    expected_pin.push(PinTransaction::set(State::Low));
    let result = BMA400::new_spi(MockSPI::new(&expected_io), MockPin::new(&expected_pin));
    assert!(matches!(result, Err(BMA400Error::ChipIdReadFailed)));
}

fn init(expected_io: &mut Vec<Transaction>, expected_pin: &mut Vec<PinTransaction>) {
    expected_pin.push(PinTransaction::set(State::Low));
    expected_io.push(Transaction::transfer(vec![0x80, 0x00], vec![0x00, 0x00]));
    expected_io.push(Transaction::transfer(vec![0x00], vec![0x00]));
    expected_pin.push(PinTransaction::set(State::High));
    expected_pin.push(PinTransaction::set(State::Low));
    expected_io.push(Transaction::transfer(vec![0x80, 0x00], vec![0x00, 0x00]));
    expected_io.push(Transaction::transfer(vec![0x00], vec![0x90]));
    expected_pin.push(PinTransaction::set(State::High));
}

#[test]
fn get_chip_id() {
    let mut expected_io = Vec::new();
    let mut expected_pin = Vec::new();
    init(&mut expected_io, &mut expected_pin);
    expected_pin.push(PinTransaction::set(State::Low));
    expected_io.push(Transaction::transfer(vec![0x80, 0x00], vec![0x00, 0x00]));
    expected_io.push(Transaction::transfer(vec![0x00], vec![0x90]));
    expected_pin.push(PinTransaction::set(State::High));
    let mut device = new(&expected_io, &expected_pin);
    let id = device.get_id().unwrap();
    assert_eq!(id, 0x90);
}

#[test]
fn get_cmd_error() {
    let mut expected_io = Vec::new();
    let mut expected_pin = Vec::new();
    init(&mut expected_io, &mut expected_pin);
    expected_pin.push(PinTransaction::set(State::Low));
    expected_io.push(Transaction::transfer(vec![0x82, 0x00], vec![0x00, 0x00]));
    expected_io.push(Transaction::transfer(vec![0x00], vec![0xFD]));
    expected_pin.push(PinTransaction::set(State::High));

    expected_pin.push(PinTransaction::set(State::Low));
    expected_io.push(Transaction::transfer(vec![0x82, 0x00], vec![0x00, 0x00]));
    expected_io.push(Transaction::transfer(vec![0x00], vec![0x02]));
    expected_pin.push(PinTransaction::set(State::High));

    let mut device = new(&expected_io, &expected_pin);
    let status = device.get_cmd_error().unwrap();
    assert!(!status);
    let status = device.get_cmd_error().unwrap();
    assert!(status);
}

#[test]
fn get_status() {
    let mut expected_io = Vec::new();
    let mut expected_pin = Vec::new();
    init(&mut expected_io, &mut expected_pin);
    // drdy Set
    expected_pin.push(PinTransaction::set(State::Low));
    expected_io.push(Transaction::transfer(vec![0x83, 0x00], vec![0x00, 0x00]));
    expected_io.push(Transaction::transfer(vec![0x00], vec![0x80]));
    expected_pin.push(PinTransaction::set(State::High));

    // cmd_rdy Set
    expected_pin.push(PinTransaction::set(State::Low));
    expected_io.push(Transaction::transfer(vec![0x83, 0x00], vec![0x00, 0x00]));
    expected_io.push(Transaction::transfer(vec![0x00], vec![0x10]));
    expected_pin.push(PinTransaction::set(State::High));

    // power_mode == LowPower
    expected_pin.push(PinTransaction::set(State::Low));
    expected_io.push(Transaction::transfer(vec![0x83, 0x00], vec![0x00, 0x00]));
    expected_io.push(Transaction::transfer(vec![0x00], vec![0x02]));
    expected_pin.push(PinTransaction::set(State::High));

    // power_mode == Normal
    expected_pin.push(PinTransaction::set(State::Low));
    expected_io.push(Transaction::transfer(vec![0x83, 0x00], vec![0x00, 0x00]));
    expected_io.push(Transaction::transfer(vec![0x00], vec![0x04]));
    expected_pin.push(PinTransaction::set(State::High));

    // interrupt triggered Set
    expected_pin.push(PinTransaction::set(State::Low));
    expected_io.push(Transaction::transfer(vec![0x83, 0x00], vec![0x00, 0x00]));
    expected_io.push(Transaction::transfer(vec![0x00], vec![0x01]));
    expected_pin.push(PinTransaction::set(State::High));

    let mut device = new(&expected_io, &expected_pin);

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
    let mut expected_io = Vec::new();
    let mut expected_pin = Vec::new();
    init(&mut expected_io, &mut expected_pin);
    expected_pin.push(PinTransaction::set(State::Low));
    expected_io.push(Transaction::transfer(vec![0x84, 0x00], vec![0x00, 0x00]));
    expected_io.push(Transaction::transfer(vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00], vec![0x01, 0x08, 0xFF, 0x0F, 0xFF, 0x07]));
    expected_pin.push(PinTransaction::set(State::High));
    let mut device = new(&expected_io, &expected_pin);
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
    let mut expected_io = Vec::new();
    let mut expected_pin = Vec::new();
    init(&mut expected_io, &mut expected_pin);
    expected_pin.push(PinTransaction::set(State::Low));
    if let Scale::Range4G = scale {
        // The default setting is 4G so we shouldn't see any configuration write
        expected_io.push(Transaction::transfer(vec![0x84, 0x00], vec![0x00, 0x00]));
        expected_io.push(Transaction::transfer(vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00], vec![0x01, 0x08, 0xFF, 0x0F, 0xFF, 0x07]));
    } else {
        expected_io.push(Transaction::write(vec![0x1A, byte]));
        expected_pin.push(PinTransaction::set(State::High));
        expected_pin.push(PinTransaction::set(State::Low));
        expected_io.push(Transaction::transfer(vec![0x84, 0x00], vec![0x00, 0x00]));
        expected_io.push(Transaction::transfer(vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00], vec![0x01, 0x08, 0xFF, 0x0F, 0xFF, 0x07]))
    }
    expected_pin.push(PinTransaction::set(State::High));
    let mut device = new(&expected_io, &expected_pin);
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
    let mut expected_io = Vec::new();
    let mut expected_pin = Vec::new();
    init(&mut expected_io, &mut expected_pin);
    expected_pin.push(PinTransaction::set(State::Low));
    expected_io.push(Transaction::transfer(vec![0x8A, 0x00], vec![0x00, 0x00]));
    expected_io.push(Transaction::transfer(vec![0x00, 0x00, 0x00], vec![0xF8, 0xFF, 0xFF]));
    expected_pin.push(PinTransaction::set(State::High));
    let mut device = new(&expected_io, &expected_pin);
    let t = device.get_sensor_clock().unwrap();
    assert_eq!(t, 0xFFFFF8);
}

#[test]
fn get_reset_status() {
    let mut expected_io = Vec::new();
    let mut expected_pin = Vec::new();
    init(&mut expected_io, &mut expected_pin);

    // No Reset Detected
    expected_pin.push(PinTransaction::set(State::Low));
    expected_io.push(Transaction::transfer(vec![0x8D, 0x00], vec![0x00, 0x00]));
    expected_io.push(Transaction::transfer(vec![0x00], vec![0x00]));
    expected_pin.push(PinTransaction::set(State::High));

    // Reset Detected
    expected_pin.push(PinTransaction::set(State::Low));
    expected_io.push(Transaction::transfer(vec![0x8D, 0x00], vec![0x00, 0x00]));
    expected_io.push(Transaction::transfer(vec![0x00], vec![0x01]));
    expected_pin.push(PinTransaction::set(State::High));

    let mut device = new(&expected_io, &expected_pin);

    let reset = device.get_reset_status().unwrap();
    assert!(!reset);
    let reset = device.get_reset_status().unwrap();
    assert!(reset);
}

#[test]
fn get_int_status0() {
    let mut expected_io = Vec::new();
    let mut expected_pin = Vec::new();
    init(&mut expected_io, &mut expected_pin);

    // drdy set
    expected_pin.push(PinTransaction::set(State::Low));
    expected_io.push(Transaction::transfer(vec![0x8E, 0x00], vec![0x00, 0x00]));
    expected_io.push(Transaction::transfer(vec![0x00], vec![0x80]));
    expected_pin.push(PinTransaction::set(State::High));

    // fwm set
    expected_pin.push(PinTransaction::set(State::Low));
    expected_io.push(Transaction::transfer(vec![0x8E, 0x00], vec![0x00, 0x00]));
    expected_io.push(Transaction::transfer(vec![0x00], vec![0x40]));
    expected_pin.push(PinTransaction::set(State::High));

    // ffull set
    expected_pin.push(PinTransaction::set(State::Low));
    expected_io.push(Transaction::transfer(vec![0x8E, 0x00], vec![0x00, 0x00]));
    expected_io.push(Transaction::transfer(vec![0x00], vec![0x20]));
    expected_pin.push(PinTransaction::set(State::High));

    // ieng_ovrrn set
    expected_pin.push(PinTransaction::set(State::Low));
    expected_io.push(Transaction::transfer(vec![0x8E, 0x00], vec![0x00, 0x00]));
    expected_io.push(Transaction::transfer(vec![0x00], vec![0x10]));
    expected_pin.push(PinTransaction::set(State::High));

    // gen2 set
    expected_pin.push(PinTransaction::set(State::Low));
    expected_io.push(Transaction::transfer(vec![0x8E, 0x00], vec![0x00, 0x00]));
    expected_io.push(Transaction::transfer(vec![0x00], vec![0x08]));
    expected_pin.push(PinTransaction::set(State::High));

    // gen1 set
    expected_pin.push(PinTransaction::set(State::Low));
    expected_io.push(Transaction::transfer(vec![0x8E, 0x00], vec![0x00, 0x00]));
    expected_io.push(Transaction::transfer(vec![0x00], vec![0x04]));
    expected_pin.push(PinTransaction::set(State::High));

    // orientch set
    expected_pin.push(PinTransaction::set(State::Low));
    expected_io.push(Transaction::transfer(vec![0x8E, 0x00], vec![0x00, 0x00]));
    expected_io.push(Transaction::transfer(vec![0x00], vec![0x02]));
    expected_pin.push(PinTransaction::set(State::High));

    // wkup set
    expected_pin.push(PinTransaction::set(State::Low));
    expected_io.push(Transaction::transfer(vec![0x8E, 0x00], vec![0x00, 0x00]));
    expected_io.push(Transaction::transfer(vec![0x00], vec![0x01]));
    expected_pin.push(PinTransaction::set(State::High));

    let mut device = new(&expected_io, &expected_pin);

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
    let mut expected_io = Vec::new();
    let mut expected_pin = Vec::new();
    init(&mut expected_io, &mut expected_pin);

    // ieng_ovrrn set
    expected_pin.push(PinTransaction::set(State::Low));
    expected_io.push(Transaction::transfer(vec![0x8F, 0x00], vec![0x00, 0x00]));
    expected_io.push(Transaction::transfer(vec![0x00], vec![0x10]));
    expected_pin.push(PinTransaction::set(State::High));

    // d_tap set
    expected_pin.push(PinTransaction::set(State::Low));
    expected_io.push(Transaction::transfer(vec![0x8F, 0x00], vec![0x00, 0x00]));
    expected_io.push(Transaction::transfer(vec![0x00], vec![0x08]));
    expected_pin.push(PinTransaction::set(State::High));

    // s_tap set
    expected_pin.push(PinTransaction::set(State::Low));
    expected_io.push(Transaction::transfer(vec![0x8F, 0x00], vec![0x00, 0x00]));
    expected_io.push(Transaction::transfer(vec![0x00], vec![0x04]));
    expected_pin.push(PinTransaction::set(State::High));

    // step_int == 2
    expected_pin.push(PinTransaction::set(State::Low));
    expected_io.push(Transaction::transfer(vec![0x8F, 0x00], vec![0x00, 0x00]));
    expected_io.push(Transaction::transfer(vec![0x00], vec![0x02]));
    expected_pin.push(PinTransaction::set(State::High));

    // step_int == 1
    expected_pin.push(PinTransaction::set(State::Low));
    expected_io.push(Transaction::transfer(vec![0x8F, 0x00], vec![0x00, 0x00]));
    expected_io.push(Transaction::transfer(vec![0x00], vec![0x01]));
    expected_pin.push(PinTransaction::set(State::High));

    let mut device = new(&expected_io, &expected_pin);

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
    let mut expected_io = Vec::new();
    let mut expected_pin = Vec::new();
    init(&mut expected_io, &mut expected_pin);

    // ieng_ovrrn set
    expected_pin.push(PinTransaction::set(State::Low));
    expected_io.push(Transaction::transfer(vec![0x90, 0x00], vec![0x00, 0x00]));
    expected_io.push(Transaction::transfer(vec![0x00], vec![0x10]));
    expected_pin.push(PinTransaction::set(State::High));

    // actch_z set
    expected_pin.push(PinTransaction::set(State::Low));
    expected_io.push(Transaction::transfer(vec![0x90, 0x00], vec![0x00, 0x00]));
    expected_io.push(Transaction::transfer(vec![0x00], vec![0x04]));
    expected_pin.push(PinTransaction::set(State::High));

    // actch_y set
    expected_pin.push(PinTransaction::set(State::Low));
    expected_io.push(Transaction::transfer(vec![0x90, 0x00], vec![0x00, 0x00]));
    expected_io.push(Transaction::transfer(vec![0x00], vec![0x02]));
    expected_pin.push(PinTransaction::set(State::High));

    // actch_x set
    expected_pin.push(PinTransaction::set(State::Low));
    expected_io.push(Transaction::transfer(vec![0x90, 0x00], vec![0x00, 0x00]));
    expected_io.push(Transaction::transfer(vec![0x00], vec![0x01]));
    expected_pin.push(PinTransaction::set(State::High));

    let mut device = new(&expected_io, &expected_pin);

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
    let mut expected_io = Vec::new();
    let mut expected_pin = Vec::new();
    init(&mut expected_io, &mut expected_pin);

    // Read full FIFO (1024, non-zero values in reserved space)
    expected_pin.push(PinTransaction::set(State::Low));
    expected_io.push(Transaction::transfer(vec![0x92, 0x00], vec![0x00, 0x00]));
    expected_io.push(Transaction::transfer(vec![0x00, 0x00], vec![0x00, 0xF4]));
    expected_pin.push(PinTransaction::set(State::High));

    // Read FIFO (640, non-zero values in both bytes)
    expected_pin.push(PinTransaction::set(State::Low));
    expected_io.push(Transaction::transfer(vec![0x92, 0x00], vec![0x00, 0x00]));
    expected_io.push(Transaction::transfer(vec![0x00, 0x00], vec![0x80, 0x02]));
    expected_pin.push(PinTransaction::set(State::High));

    let mut device = new(&expected_io, &expected_pin);

    let len = device.get_fifo_len().unwrap();
    assert_eq!(len, 1024);
    let len = device.get_fifo_len().unwrap();
    assert_eq!(len, 640);
}

#[test]
fn read_fifo_frames() {
    let mut expected_io = Vec::new();
    let mut expected_pin = Vec::new();
    init(&mut expected_io, &mut expected_pin);
    expected_pin.push(PinTransaction::set(State::Low));
    expected_io.push(Transaction::transfer(vec![0x94, 0x00], vec![0x00, 0x00]));
    expected_io.push(Transaction::transfer(Vec::from([0u8; 15]), vec![0x48, 0x6E, 0x9E, 0x01, 0x80, 0x0F, 0xFF, 0x0F, 0x7F, 0xA0, 0xF8, 0xFF, 0xFF, 0x80, 0x00]));
    expected_pin.push(PinTransaction::set(State::High));
    let mut device = new(&expected_io, &expected_pin);
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
    let mut expected_io = Vec::new();
    let mut expected_pin = Vec::new();
    init(&mut expected_io, &mut expected_pin);
}
