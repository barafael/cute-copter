# cute-copter

A little whimsy copter based on a PCB frame, not at all premium components. Can we make it fly with Rust?

I'm also working on the transmitter for this copter here: https://github.com/barafael/cute-copter-remote

# Hardware

As of August 2022, the copter is not available on AliExpress.
~~If you find this MoebiusTech product somewhere, please let me know!~~
As of October 15th, it's back :)

https://de.aliexpress.com/item/4001108687090.html?gatewayAdapt=glo2deu&spm=a2g0o.order_list.0.0.21ef5c5f4uzTWV

# Controller

STM32F103 LQFP-48

# IMU Sensor

* MPU6050 on I2C1, PB6/PB7

# Radio

* NRF24L01 on SPI2, (sck, miso, mosi): (PB13, PB14, PB15)
* csn: PB12
* chip_enable: PA11

# [TODO] UART

By elimination, there could only be 2 UARTs active:

* USART1 on (tx, rx): (PA9, PA10)
* USART3 on (tx, rx): (PB10, PB11)

Alternate pins for USART1 are PB7 and PB6 are already used for I2C1.

USART2 and USART3 do not have alternate pins. 

USART2 pins are PA2 and PA3 are already used by PWM pins.

I don't know if there is an active UART on the remaining pins, need to measure. But wouldn't it be cool to see some logs at least?

# Flash
PID parameters and so on are stored on last page of flash (page 127) and loaded on startup, stored on arming the drone.

# Pin Designations
These were reverse engineered with a multimeter, oscilloscope and a crystal ball, so no guarantees. Work in progress, too.
NRF radio and MPU6050 are confirmed working with this pinout, motors too.

![image](https://user-images.githubusercontent.com/6966738/160289131-38dee0a0-e433-4212-8979-465aec81422b.png)

# Status
- [x] IMU data
-  [x] Driver needs some love; soldered probes to the I2C pullups to check what data is being exchanged between MCU and IMU
-  [x] Take a deep look at: https://github.com/MoebiusTech/Stm32f103rct6
- [ ] Handle events and asynchronicity: RTIC, embassy, or anything better than the current blocking waiting main loop style. DMA and interrupts might just be enough.
- [x] Radio connection
-  [ ] DMA for radio packets
 - [ ] Fully fleshed out but basic radio protocol
- [x] PWM for the motors
-  [ ] DMA for PWM (not really needed is it?]
- [x] Flash for persistence

# IMU Traces

![image](https://user-images.githubusercontent.com/6966738/182554755-49569ae1-2900-46c3-8a50-de45c6bce58e.png)

https://github.com/barafael/cute-copter/raw/main/imu-0.sal

Is capturing a trace from openly accessible solder joints IP theft? I don't care, also, on the transmitter there is a fake STM32. This may be whataboutism, but meh...
