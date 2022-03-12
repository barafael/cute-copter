#![deny(unsafe_code)]
#![no_main]
#![cfg_attr(not(test), no_std)]

use cute_copter_config_proto::configuration::Config;
use mpu6050_dmp::sensor::Mpu6050;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};
use state_machine::{Armed, Copter, Disarmed};
use stm32f1xx_hal::flash::FlashWriter;
use stm32f1xx_hal::gpio::CRL;
use stm32f1xx_hal::gpio::{gpioc::PC13, Output, PinState, PushPull};
use stm32f1xx_hal::gpio::{Alternate, OpenDrain};
use stm32f1xx_hal::i2c::{BlockingI2c, DutyCycle, Mode};
use stm32f1xx_hal::pac;
use stm32f1xx_hal::pac::I2C1;
use stm32f1xx_hal::prelude::*;

//mod test_imu;
mod error;
mod state_machine;

type Mpu = Mpu6050<
    stm32f1xx_hal::i2c::BlockingI2c<
        I2C1,
        (
            stm32f1xx_hal::gpio::Pin<Alternate<OpenDrain>, CRL, 'B', 6_u8>,
            stm32f1xx_hal::gpio::Pin<Alternate<OpenDrain>, CRL, 'B', 7_u8>,
        ),
    >,
>;
use cortex_m_rt::entry;
use mpu6050_dmp::{address::Address, quaternion::Quaternion, yaw_pitch_roll::YawPitchRoll};

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

    // Setup LED
    let mut gpioc = dp.GPIOC.split();
    let mut led = gpioc
        .pc13
        .into_push_pull_output_with_state(&mut gpioc.crh, PinState::Low);

    let mut gpiob = dp.GPIOB.split();
    let scl = gpiob.pb6.into_alternate_open_drain(&mut gpiob.crl);
    let sda = gpiob.pb7.into_alternate_open_drain(&mut gpiob.crl);

    // Setup i2c
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

    let mut delay = cp.SYST.delay(&clocks);

    let mut sensor = Mpu6050::new(i2c, Address::default()).unwrap();

    sensor.initialize_dmp(&mut delay).unwrap();

    let syst = delay.release().release();

    loop {
        let mut writer = flash.writer(
            stm32f1xx_hal::flash::SectorSize::Sz1K,
            stm32f1xx_hal::flash::FlashSize::Sz128K,
        );
        let config = { state_machine::load_from_flash(&mut writer).unwrap_or_default() };
        let mut copter = Copter::from_config(config);
        loop {
            let len = sensor.get_fifo_count().unwrap();
            if len >= 28 {
                //led.toggle();
                let mut buf = [0; 28];
                let buf = sensor.read_fifo(&mut buf).unwrap();

                let quat = Quaternion::from_bytes(&buf[..16]).unwrap();
                let ypr = YawPitchRoll::from(quat);
                rprintln!("{:.5}, {:.5}, {:.5}", ypr.yaw, ypr.pitch, ypr.roll);
            }
        }
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
