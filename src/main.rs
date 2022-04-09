#![deny(unsafe_code)]
#![no_main]
#![cfg_attr(not(test), no_std)]

use mpu6050_dmp::accel::AccelFullScale;
use mpu6050_dmp::gyro::GyroFullScale;
use mpu6050_dmp::sensor::Mpu6050;
use nrf24_rs::config::{DataPipe, NrfConfig, PALevel, PayloadSize};
use nrf24_rs::Nrf24l01;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};
use state_machine::Copter;
use stm32f1xx_hal::flash::{FlashSize, SectorSize};
use stm32f1xx_hal::gpio::PinState;
use stm32f1xx_hal::i2c::{BlockingI2c, DutyCycle, Mode};
use stm32f1xx_hal::pac;
use stm32f1xx_hal::prelude::*;

//mod test_imu;
mod error;
mod state_machine;

use cortex_m_rt::entry;
use mpu6050_dmp::{address::Address, quaternion::Quaternion, yaw_pitch_roll::YawPitchRoll};
use stm32f1xx_hal::spi::Mode as SpiMode;
use stm32f1xx_hal::spi::Spi;
use stm32f1xx_hal::timer::{Channel, Tim2NoRemap};

pub const MODE: SpiMode = nrf24_rs::SPI_MODE;
const MESSAGE: &[u8; 17] = b"Here's a message!";

#[entry]
fn main() -> ! {
    // Setup clocks
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();
    let mut flash = dp.FLASH.constrain();
    let rcc = dp.RCC.constrain();
    let mut afio = dp.AFIO.constrain();

    rtt_init_print!();
    rprintln!("init");

    let clocks = rcc
        .cfgr
        .sysclk(72.MHz())
        .pclk1(48.MHz())
        .freeze(&mut flash.acr);

    let mut delay = cp.SYST.delay(&clocks);

    let mut gpioa = dp.GPIOA.split();
    let mut gpiob = dp.GPIOB.split();
    let mut gpioc = dp.GPIOC.split();

    // Setup LED
    let mut led = gpioc
        .pc13
        .into_push_pull_output_with_state(&mut gpioc.crh, PinState::Low);

    // Setup i2c
    let scl = gpiob.pb6.into_alternate_open_drain(&mut gpiob.crl);
    let sda = gpiob.pb7.into_alternate_open_drain(&mut gpiob.crl);

    let i2c = BlockingI2c::i2c1(
        dp.I2C1,
        (scl, sda),
        &mut afio.mapr,
        Mode::Fast {
            frequency: 400_000.Hz(),
            duty_cycle: DutyCycle::Ratio16to9,
        },
        clocks,
        1000,
        10,
        1000,
        1000,
    );

    // Setup MPU6050 with DMP
    let mut imu = Mpu6050::new(i2c, Address::default()).unwrap();

    imu.set_gyro_full_scale(GyroFullScale::Deg2000).unwrap();
    imu.set_accel_full_scale(AccelFullScale::G16).unwrap();

    // Note: additionally, wait about 15 seconds after turn on while not moving the sensor for self-calibration.
    imu.calibrate_accel(255, &mut delay).unwrap();
    imu.calibrate_gyro(255, &mut delay).unwrap();

    imu.initialize_dmp(&mut delay).unwrap();

    // Setup flash
    let mut writer = flash.writer(SectorSize::Sz1K, FlashSize::Sz128K);

    // Setup config
    let config = { state_machine::load_from_flash(&mut writer).unwrap_or_default() };
    let mut _copter = Copter::from_config(config);

    // Setup PID controllers
    let mut orientation_controller_roll = pid_loop::PID::<f32, 1>::new(0.5, 0.5, 0.5, 0.0, 0.0);
    let mut orientation_controller_pitch = pid_loop::PID::<f32, 1>::new(0.5, 0.5, 0.5, 0.0, 0.0);
    let mut orientation_controller_yaw = pid_loop::PID::<f32, 1>::new(0.5, 0.5, 0.5, 0.0, 0.0);

    let mut rate_controller_roll = pid_loop::PID::<f32, 1>::new(0.5, 0.5, 0.5, 0.0, 0.0);
    let mut rate_controller_pitch = pid_loop::PID::<f32, 1>::new(0.5, 0.5, 0.5, 0.0, 0.0);
    let mut rate_controller_yaw = pid_loop::PID::<f32, 1>::new(0.5, 0.5, 0.5, 0.0, 0.0);

    // Setup SPI
    let sck = gpiob.pb13.into_alternate_push_pull(&mut gpiob.crh);
    let miso = gpiob.pb14;
    let mosi = gpiob.pb15.into_alternate_push_pull(&mut gpiob.crh);
    let cs = gpiob.pb12.into_push_pull_output(&mut gpiob.crh);

    let spi = Spi::spi2(dp.SPI2, (sck, miso, mosi), MODE, 1.MHz(), clocks);

    // Setup NRF radio
    let config = NrfConfig::default()
        .channel(8)
        .pa_level(PALevel::Low)
        .ack_payloads_enabled(true)
        // We will use a payload size the size of our message
        .payload_size(PayloadSize::Static(MESSAGE.len() as u8));

    let chip_enable = gpioa.pa11.into_push_pull_output(&mut gpioa.crh);

    let mut nrf = Nrf24l01::new(spi, chip_enable, cs, &mut delay, config).unwrap();
    if !nrf.is_connected().unwrap() {
        panic!("Chip is not connected.");
    }

    nrf.open_reading_pipe(DataPipe::DP0, b"Node1").unwrap();
    nrf.start_listening().unwrap();

    // Setup PWM outputs for motors
    let c0 = gpioa.pa0.into_alternate_push_pull(&mut gpioa.crl);
    let c1 = gpioa.pa1.into_alternate_push_pull(&mut gpioa.crl);
    let c2 = gpioa.pa2.into_alternate_push_pull(&mut gpioa.crl);
    let c3 = gpioa.pa3.into_alternate_push_pull(&mut gpioa.crl);

    let pins = (c0, c1, c2, c3);

    let mut pwm = dp
        .TIM2
        .pwm_hz::<Tim2NoRemap, _, _>(pins, &mut afio.mapr, 1.kHz(), &clocks);

    // Enable clock on each of the channels
    pwm.enable(Channel::C1);
    pwm.enable(Channel::C2);
    pwm.enable(Channel::C3);
    pwm.enable(Channel::C4);

    let max = pwm.get_max_duty();

    pwm.set_duty(Channel::C1, max / 5);
    pwm.set_duty(Channel::C2, max / 5);
    pwm.set_duty(Channel::C3, max / 5);
    pwm.set_duty(Channel::C4, max / 5);

    // Setup mixer variables.
    let throttle = 0;

    // Front right
    let fr_roll = 0.25f32;
    let fr_pitch = 0.25f32;
    let fr_yaw = 0.25f32;

    // Front left
    let fl_roll = -0.25f32;
    let fl_pitch = 0.25f32;
    let fl_yaw = -0.25f32;

    // Back right
    let br_roll = 0.25f32;
    let br_pitch = -0.25f32;
    let br_yaw = -0.25f32;

    // Back left
    let bl_roll = -0.25f32;
    let bl_pitch = -0.25f32;
    let bl_yaw = 0.25f32;

    rprintln!("Starting copter loop");
    loop {
        led.set_low();
        while imu.get_fifo_count().unwrap() < 28 {
            continue;
        }
        let ypr = {
            let mut buf = [0; 28];
            let buf = imu.read_fifo(&mut buf).unwrap();

            let quat = Quaternion::from_bytes(&buf[..16]).unwrap();
            YawPitchRoll::from(quat)
        };
        led.toggle();
        let rates = imu.gyro().unwrap();

        // TODO: get data from radio.
        let desired: (f32, f32, f32) = { (0.0, 0.0, 0.0) };

        // Roll rate correction.
        let desired_roll_rate = orientation_controller_roll.next(desired.0, ypr.roll * 10.0);
        rprintln!("{},    {}", ypr.roll * 10.0, desired_roll_rate);

        let actual_roll_rate = rates.x();

        let roll_rate_correction = rate_controller_roll.next(desired_roll_rate, actual_roll_rate);

        // Pitch rate correction.
        let desired_pitch_rate = orientation_controller_pitch.next(desired.1, ypr.pitch * 10.0);

        let actual_pitch_rate = rates.y();

        let pitch_rate_correction =
            rate_controller_pitch.next(desired_pitch_rate, actual_pitch_rate);

        // Yaw rate correction.
        let desired_yaw_rate = orientation_controller_yaw.next(desired.2, ypr.yaw * 10.0);

        let actual_yaw_rate = rates.z();

        let yaw_rate_correction = rate_controller_yaw.next(desired_yaw_rate, actual_yaw_rate);

        /*
        rprintln!(
            "{:.2}, {:.2}, {:.2}",
            roll_rate_correction,
            pitch_rate_correction,
            yaw_rate_correction
        );
        */

        if nrf.data_available().unwrap() {
            led.set_high();
            let mut buffer = [0; MESSAGE.len()];
            nrf.read(&mut buffer).unwrap();
            rprintln!("{:?}", buffer);
        }

        let _front_right = (throttle as f32
            + fr_roll * roll_rate_correction
            + fr_pitch * pitch_rate_correction
            + fr_yaw * yaw_rate_correction) as u16;
        let _front_left = (throttle as f32
            + fl_roll * roll_rate_correction
            + fl_pitch * pitch_rate_correction
            + fl_yaw * yaw_rate_correction) as u16;
        let _back_right = (throttle as f32
            + br_roll * roll_rate_correction
            + br_pitch * pitch_rate_correction
            + br_yaw * yaw_rate_correction) as u16;
        let _back_left = (throttle as f32
            + bl_roll * roll_rate_correction
            + bl_pitch * pitch_rate_correction
            + bl_yaw * yaw_rate_correction) as u16;

        // For now, assuming that the throttle variables are in 0.0..100.0.
        /*
        pwm.set_duty(Channel::C1, front_right);
        pwm.set_duty(Channel::C2, front_left);
        pwm.set_duty(Channel::C3, back_right);
        pwm.set_duty(Channel::C4, back_left);
        */
    }

    /*
    loop {
        let config = { state_machine::load_from_flash(&mut writer).unwrap_or_default() };
        let mut copter = Copter::from_config(config);
        let armed = copter.arm(&mut writer).unwrap();
        rprintln!("{:?}", armed);
        copter = armed.disarm().unwrap();
        loop {
            continue;
        }
    }
    */
}
