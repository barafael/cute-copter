# cute-copter

A little whimsy copter based on a PCB frame, not at all premium components. Can we make it fly with Rust?

I'm also working on the transmitter for this copter here: https://github.com/barafael/cute-copter-remote

# Hardware

https://de.aliexpress.com/item/4001108687090.html?gatewayAdapt=glo2deu&spm=a2g0o.order_list.0.0.21ef5c5f4uzTWV

# Controller

STM32F103 LQFP-48

# IMU Sensor

* MPU6050 on I2C1, PB6/PB7

# Radio

* NRF24L01 on SPI2, (sck, miso, mosi): (PB13, PB14, PB15)
* csn: PB12
* chip_enable: PA11

# Flash
PID parameters and so on are stored on last page of flash (page 127) and loaded on startup, stored on arming the drone.

# Pin Designations
These were reverse engineered with a multimeter, oscilloscope and a crystal ball, so no guarantees. Work in progress, too.
NRF radio and MPU6050 are confirmed working with this pinout, motors probably too (but can't test rn due to being stupid and blowing up drone control board power supply).

![image](https://user-images.githubusercontent.com/6966738/160289131-38dee0a0-e433-4212-8979-465aec81422b.png)
