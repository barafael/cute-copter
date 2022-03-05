#![deny(unsafe_code)]
#![no_main]
#![no_std]

use mpu6050_dmp::sensor::Mpu6050;
use panic_rtt_target as _;
use rtic::app;
use rtt_target::{rprintln, rtt_init_print};
use stm32f1xx_hal::gpio::CRL;
use stm32f1xx_hal::gpio::{gpioc::PC13, Output, PinState, PushPull};
use stm32f1xx_hal::gpio::{Alternate, OpenDrain};
use stm32f1xx_hal::i2c::{BlockingI2c, DutyCycle, Mode};
use stm32f1xx_hal::pac::I2C1;
use stm32f1xx_hal::prelude::*;
use systick_monotonic::{fugit::Duration, Systick};

//mod test_imu;
mod error;
mod parameter;
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

#[app(device = stm32f1xx_hal::pac, peripherals = true, dispatchers = [SPI1])]
mod app {
    use super::*;
    use mpu6050_dmp::yaw_pitch_roll::YawPitchRoll;

    #[shared]
    struct Shared {
        mpu: Mpu,
    }

    #[local]
    struct Local {
        led: PC13<Output<PushPull>>,
        state: bool,
    }

    #[monotonic(binds = SysTick, default = true)]
    type MonoTimer = Systick<1000>;

    #[init]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        // Setup clocks
        let mut flash = cx.device.FLASH.constrain();
        let rcc = cx.device.RCC.constrain();
        let mut afio = cx.device.AFIO.constrain();

        rtt_init_print!();
        rprintln!("init");

        let clocks = rcc
            .cfgr
            .sysclk(48.MHz())
            .pclk1(24.MHz())
            .freeze(&mut flash.acr);

        // Setup LED
        let mut gpioc = cx.device.GPIOC.split();
        let led = gpioc
            .pc13
            .into_push_pull_output_with_state(&mut gpioc.crh, PinState::Low);

        let mut gpiob = cx.device.GPIOB.split();
        let scl = gpiob.pb6.into_alternate_open_drain(&mut gpiob.crl);
        let sda = gpiob.pb7.into_alternate_open_drain(&mut gpiob.crl);

        // Setup i2c
        let i2c = BlockingI2c::i2c1(
            cx.device.I2C1,
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

        let mut delay = cx.core.SYST.delay(&clocks);

        let mut sensor =
            mpu6050_dmp::sensor::Mpu6050::new(i2c, mpu6050_dmp::address::Address::default())
                .unwrap();

        sensor.initialize_dmp(&mut delay).unwrap();

        let syst = delay.release().release();

        let mono = Systick::new(syst, 36_000_000);

        // Schedule the blinking task
        blink::spawn_after(Duration::<u64, 1, 1000>::from_ticks(1000)).unwrap();
        attitude::spawn_after(Duration::<u64, 1, 1000>::from_ticks(1000)).unwrap();

        (
            Shared { mpu: sensor },
            Local { led, state: false },
            init::Monotonics(mono),
        )
    }

    #[task(local = [led, state])]
    fn blink(cx: blink::Context) {
        rprintln!("blink");
        if *cx.local.state {
            cx.local.led.set_high();
            *cx.local.state = false;
        } else {
            cx.local.led.set_low();
            *cx.local.state = true;
        }
        blink::spawn_after(Duration::<u64, 1, 1000>::from_ticks(1000)).unwrap();
    }

    #[task(shared = [mpu])]
    fn attitude(mut cx: attitude::Context) {
        rprintln!("attitude");
        cx.shared.mpu.lock(|sensor| {
            // get roll and pitch estimate
            let len = sensor.get_fifo_count().unwrap();
            if len >= 28 {
                let mut buf = [0; 28];
                let buf = sensor.read_fifo(&mut buf).unwrap();
                let quat = mpu6050_dmp::quaternion::Quaternion::from_bytes(buf).unwrap();
                let ypr = YawPitchRoll::from(quat);
                rprintln!("{:?}", ypr);
            }
        });

        attitude::spawn_after(Duration::<u64, 1, 1000>::from_ticks(1000)).unwrap();
    }
}
