#![deny(unsafe_code)]
#![no_main]
#![cfg_attr(not(test), no_std)]

use mpu6050_dmp::sensor::Mpu6050;
use nrf24_rs::config::{DataPipe, NrfConfig, PALevel, PayloadSize};
use nrf24_rs::Nrf24l01;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};
use state_machine::Copter;
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

pub const MODE: SpiMode = nrf24_rs::SPI_MODE;
const MESSAGE: &[u8] = b"Here's a message!";

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

    let mut writer = flash.writer(
        stm32f1xx_hal::flash::SectorSize::Sz1K,
        stm32f1xx_hal::flash::FlashSize::Sz128K,
    );
    let config = { state_machine::load_from_flash(&mut writer).unwrap_or_default() };
    let mut _copter = Copter::from_config(config);

    let mut controller = pid_loop::PID::<f32, 1>::new(0.5, 0.5, 0.5, 0.0, 0.0);

    let sck = gpiob.pb13.into_alternate_push_pull(&mut gpiob.crh);
    let miso = gpiob.pb14;
    let mosi = gpiob.pb15.into_alternate_push_pull(&mut gpiob.crh);
    let cs = gpiob.pb12.into_push_pull_output(&mut gpiob.crh);

    let spi = Spi::spi2(dp.SPI2, (sck, miso, mosi), MODE, 1.MHz(), clocks);

    let config = NrfConfig::default()
        .channel(8)
        .pa_level(PALevel::Min)
        // We will use a payload size the size of our message
        .payload_size(PayloadSize::Static(MESSAGE.len() as u8));

    let mut gpioa = dp.GPIOA.split();
    let chip_enable = gpioa.pa11.into_push_pull_output(&mut gpioa.crh);

    // Initialize the chip
    let mut nrf = Nrf24l01::new(spi, chip_enable, cs, &mut delay, config).unwrap();
    if !nrf.is_connected().unwrap() {
        panic!("Chip is not connected.");
    }

    nrf.open_reading_pipe(DataPipe::DP0, b"Node1").unwrap();

    loop {
        led.set_low();
        while sensor.get_fifo_count().unwrap() < 28 {
            continue;
        }
        let ypr = {
            let mut buf = [0; 28];
            let buf = sensor.read_fifo(&mut buf).unwrap();

            let quat = Quaternion::from_bytes(&buf[..16]).unwrap();
            YawPitchRoll::from(quat)
        };
        led.toggle();
        rprintln!("{:.5}, {:.5}, {:.5}", ypr.yaw, ypr.pitch, ypr.roll);

        if nrf.data_available().unwrap() {
            let mut buffer = [0; MESSAGE.len()];
            nrf.read(&mut buffer).unwrap();
            rprintln!("{:?}", buffer);
        }
        led.toggle();

        let desired: (f32, f32, f32) = { (0.0, 0.0, 0.0) };

        let _correction_yaw = controller.next(desired.0, ypr.yaw);
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
