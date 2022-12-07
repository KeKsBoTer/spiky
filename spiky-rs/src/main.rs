#![no_std]
#![no_main]

use core::any::Any;
use core::fmt::Write;

use arduino_nano33iot as bsp;
use bsp::hal::gpio::v2::{
    Alternate, AlternateE, AlternateF, Input, Interrupt, Output, Pin, PushPull, E, PA06, PA20,
    PB10, PB11,
};
use bsp::hal::gpio::{PinId, PullDown};
use bsp::hal::pwm::{Channel, Pwm1};
use bsp::hal::time::Seconds;
use bsp::hal::{self, pwm};
use bsp::hal::{prelude::*, thumbv6m};

use bsp::pac::port::{self, PMUX0_};
use bsp::pac::rtc::mode1::per;
use bsp::pac::{interrupt, pac0, NVIC, PORT, RTC};
use lsm6ds3::LSM6DS3;
#[cfg(not(feature = "use_semihosting"))]
use panic_halt as _;
#[cfg(feature = "use_semihosting")]
use panic_semihosting as _;

use bsp::entry;
use hal::clock::GenericClockController;
use hal::pac::{CorePeripherals, Peripherals};

use hal::delay::Delay;
use serial::Serial;

mod lsm6ds3;
mod serial;

#[entry]
fn main() -> ! {
    let mut peripherals = Peripherals::take().unwrap();
    let mut core = CorePeripherals::take().unwrap();
    let mut clocks = GenericClockController::with_internal_32kosc(
        peripherals.GCLK,
        &mut peripherals.PM,
        &mut peripherals.SYSCTRL,
        &mut peripherals.NVMCTRL,
    );

    let pins = bsp::Pins::new(peripherals.PORT);
    // let mut _d7: Pin<PA06, AlternateE> = pins.d7.into_alternate();

    // let button_pin: Pin<_, _> = pins.d2.into_pull_down_input();

    // let mut samples = [0.; 16];
    // let mut sample_idx = 0;
    // let mut pos = [0f32; 3];
    // let mut amplitudes = [0u32; 8];
    // let mut led: bsp::Led = pins.led_sck.into();

    // let mut serial = serial::SerialUART::new(
    //     &mut clocks,
    //     peripherals.SERCOM5,
    //     &mut peripherals.PM,
    //     pins.rx,
    //     pins.tx,
    // );
    let mut _serial = serial::SerialUSB::take(
        peripherals.USB,
        &mut clocks,
        &mut peripherals.PM,
        pins.usb_dm,
        pins.usb_dp,
        &mut core.NVIC,
    )
    .unwrap();

    // let mut delay = Delay::new(core.SYST, &mut clocks);

    // let gen = clocks.gclk0();
    // let mut pwm = Pwm1::new(
    //     &clocks.tcc0_tcc1(&gen).unwrap(),
    //     2.khz(),
    //     peripherals.TCC1,
    //     &mut peripherals.PM,
    // );

    // let mut imu = lsm6ds3::LSM6DS3::new(
    //     &mut clocks,
    //     peripherals.SERCOM4,
    //     &mut peripherals.PM,
    //     pins.sda,
    //     pins.scl,
    // )
    // .unwrap();

    // pwm.set_duty(Channel::_0, pwm.get_max_duty() / 2);
    // // pins.d7.into_mode()
    // while !serial.available() {}

    loop {
        // if button_pin.is_high().unwrap() {
        //     pwm.enable(Channel::_0);
        // } else {
        //     pwm.disable(Channel::_0);
        // }

        // if false && imu.acceleration_available().unwrap() {
        //     imu.read_acceleration(&mut pos).unwrap();
        //     let x = pos[0];
        //     samples[sample_idx] = x;
        //     sample_idx += 1;
        //     // let y = pos[1];
        //     // let z = pos[2];
        //     if sample_idx == samples.len() {
        //         sample_idx = 0;

        //         let spectrum = microfft::real::rfft_16(&mut samples);
        //         spectrum[0].im = 0.0;
        //         for (i, c) in spectrum.iter().enumerate() {
        //             amplitudes[i] = c.norm_sqr() as u32;
        //         }
        //         // writeln!(
        //         //     &mut serial,
        //         //     "[{}{}{}{}{}{}{}{}]",
        //         //     amplitudes[0],
        //         //     amplitudes[1],
        //         //     amplitudes[2],
        //         //     amplitudes[3],
        //         //     amplitudes[4],
        //         //     amplitudes[5],
        //         //     amplitudes[6],
        //         //     amplitudes[7]
        //         // )
        //         // .unwrap();
        //         // writeln!(&mut serial, "worked!").unwrap();
        //     }
        // }
        // // match serial.write_str("test") {
        // //     Ok(_) => led.set_low().unwrap(),
        // //     Err(_) => led.set_high().unwrap(),
        // // }
        // serial.write_char('A');
        // // write!(&mut serial, "test").unwrap();
    }
}
