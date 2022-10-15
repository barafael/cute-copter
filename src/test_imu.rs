use crate::Mpu;
use mpu6050::device::{AccelRange, GyroRange, ACCEL_HPF, CLKSEL};
use stm32f1xx_hal::delay::Delay;

pub(crate) fn test_imu(mpu: &mut Mpu, delay: &mut Delay) {
    // Test power management
    defmt::println!("Test power management");

    // Test gyro config
    defmt::println!("Test gyro config");
    assert_eq!(mpu.get_gyro_range().unwrap(), GyroRange::D250);
    mpu.set_gyro_range(GyroRange::D500).unwrap();
    assert_eq!(mpu.get_gyro_range().unwrap(), GyroRange::D500);

    // Test accel config
    defmt::println!("Test accel config");
    assert_eq!(mpu.get_accel_range().unwrap(), AccelRange::G2);
    mpu.set_accel_range(AccelRange::G4).unwrap();
    assert_eq!(mpu.get_accel_range().unwrap(), AccelRange::G4);

    // accel_hpf: per default RESET/no filter, see ACCEL_CONFIG
    defmt::println!("Test accel hpf");
    assert_eq!(mpu.get_accel_hpf().unwrap(), ACCEL_HPF::_RESET);
    mpu.set_accel_hpf(ACCEL_HPF::_1P25).unwrap();
    assert_eq!(mpu.get_accel_hpf().unwrap(), ACCEL_HPF::_1P25);
    mpu.set_accel_hpf(ACCEL_HPF::_2P5).unwrap();
    assert_eq!(mpu.get_accel_hpf().unwrap(), ACCEL_HPF::_2P5);
    mpu.set_accel_hpf(ACCEL_HPF::_5).unwrap();
    assert_eq!(mpu.get_accel_hpf().unwrap(), ACCEL_HPF::_5);
    mpu.set_accel_hpf(ACCEL_HPF::_0P63).unwrap();
    assert_eq!(mpu.get_accel_hpf().unwrap(), ACCEL_HPF::_0P63);
    mpu.set_accel_hpf(ACCEL_HPF::_HOLD).unwrap();
    assert_eq!(mpu.get_accel_hpf().unwrap(), ACCEL_HPF::_HOLD);

    // test sleep. Default no, in wake()
    defmt::println!("Test sleep");
    assert_eq!(mpu.get_sleep_enabled().unwrap(), false);
    mpu.set_sleep_enabled(true).unwrap();
    assert_eq!(mpu.get_sleep_enabled().unwrap(), true);
    mpu.set_sleep_enabled(false).unwrap();
    assert_eq!(mpu.get_sleep_enabled().unwrap(), false);

    // test temp enable/disable
    defmt::println!("Test temp enable/disable");
    mpu.set_temp_enabled(false).unwrap();
    assert_eq!(mpu.get_temp_enabled().unwrap(), false);
    //assert_eq!(mpu.get_temp().unwrap(), 36.53); // This fails?
    mpu.set_temp_enabled(true).unwrap();
    assert_eq!(mpu.get_temp_enabled().unwrap(), true);
    assert_ne!(mpu.get_temp().unwrap(), 36.53);

    // Test clksel: GXAXIS per default, set in wake()
    defmt::println!("Test CLKSEL");
    assert_eq!(mpu.get_clock_source().unwrap(), CLKSEL::GXAXIS);
    mpu.set_clock_source(CLKSEL::GYAXIS).unwrap();
    assert_eq!(mpu.get_clock_source().unwrap(), CLKSEL::GYAXIS);
    mpu.set_clock_source(CLKSEL::GZAXIS).unwrap();
    assert_eq!(mpu.get_clock_source().unwrap(), CLKSEL::GZAXIS);
    mpu.set_clock_source(CLKSEL::OSCILL).unwrap();
    assert_eq!(mpu.get_clock_source().unwrap(), CLKSEL::OSCILL);
    mpu.set_clock_source(CLKSEL::STOP).unwrap();
    assert_eq!(mpu.get_clock_source().unwrap(), CLKSEL::STOP);
    mpu.set_clock_source(CLKSEL::RESERV).unwrap();
    assert_eq!(mpu.get_clock_source().unwrap(), CLKSEL::RESERV);
    mpu.set_clock_source(CLKSEL::EXT_19P2).unwrap();
    assert_eq!(mpu.get_clock_source().unwrap(), CLKSEL::EXT_19P2);
    mpu.set_clock_source(CLKSEL::EXT_32p7).unwrap();
    assert_eq!(mpu.get_clock_source().unwrap(), CLKSEL::EXT_32p7);

    // reset
    defmt::println!("Test reset");
    mpu.reset_device(delay).unwrap();
    assert_eq!(mpu.get_accel_hpf().unwrap(), ACCEL_HPF::_RESET);
    assert_eq!(mpu.get_accel_range().unwrap(), AccelRange::G2);
    assert_eq!(mpu.get_gyro_range().unwrap(), GyroRange::D250);
    assert_eq!(mpu.get_sleep_enabled().unwrap(), true);
    assert_eq!(mpu.get_temp_enabled().unwrap(), true);

    defmt::println!("Test successful");
}
