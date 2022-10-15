#![no_main]
#![no_std]

use cortex_m_rt::entry;
use stm32f1xx_hal::{gpio::PinState, pac, prelude::*};

use defmt_rtt as _; // global logger
use panic_probe as _;
use stm32f1xx_hal as _;

#[entry]
fn main() -> ! {
    defmt::println!("Hello, blinky!");

    let dp = pac::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let rcc = dp.RCC.constrain();

    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let mut gpiob = dp.GPIOB.split();

    let mut led_1 = gpiob
        .pb1
        .into_open_drain_output_with_state(&mut gpiob.crl, PinState::High);
    let mut led_2 = gpiob
        .pb2
        .into_open_drain_output_with_state(&mut gpiob.crl, PinState::High);
    let mut led_3 = gpiob
        .pb8
        .into_open_drain_output_with_state(&mut gpiob.crh, PinState::High);
    let mut led_4 = gpiob
        .pb9
        .into_open_drain_output_with_state(&mut gpiob.crh, PinState::High);

    let mut delay = cp.SYST.delay(&clocks);

    let mut idx = 0;
    loop {
        defmt::trace!("loop");
        delay.delay_ms(1_000_u16);
        match idx % 4 {
            0 => {
                led_4.set_high();
                led_1.set_low();
            }
            1 => {
                led_1.set_high();
                led_2.set_low();
            }
            2 => {
                led_2.set_high();
                led_3.set_low();
            }
            3 => {
                led_3.set_high();
                led_4.set_low();
            }
            _ => unreachable!(),
        }
        idx += 1;
    }
}
