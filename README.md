# cute-copter

A little whimsy copter based on a PCB frame, not at all premium components. Can we make it fly with Rust?

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
