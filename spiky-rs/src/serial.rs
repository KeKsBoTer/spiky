use core::fmt::{self, Write};
use core::mem::MaybeUninit;

use arduino_nano33iot as bsp;

use bsp::ehal::can::nb;
use bsp::hal::clock::GenericClockController;
use bsp::hal::prelude::nb::block;
use bsp::hal::time::Hertz;
use bsp::pac::{self, interrupt, PM, USB};
use bsp::{uart, usb_allocator, Rx, Tx, Uart, UsbDm, UsbDp};
use cortex_m::prelude::_embedded_hal_serial_Write;
use usb_device;
use usbd_serial::{self, USB_CLASS_CDC};

#[cfg(not(feature = "use_semihosting"))]
use panic_halt as _;

use bsp::hal::usb::UsbBus;
use usb_device::bus::UsbBusAllocator;
use usb_device::prelude::*;
use usbd_serial::SerialPort;

use cortex_m::peripheral::NVIC;

pub trait Serial: Write {
    fn available(&self) -> bool;
}

pub struct SerialUSB<'a> {
    bus: UsbDevice<'a, UsbBus>,
    serial: SerialPort<'a, UsbBus>,
}

static mut USB_SERIAL: (
    MaybeUninit<SerialUSB>,
    MaybeUninit<UsbBusAllocator<UsbBus>>,
    bool,
) = (MaybeUninit::uninit(), MaybeUninit::uninit(), false);

impl<'a> SerialUSB<'a> {
    pub fn take(
        usb: USB,
        clocks: &mut GenericClockController,
        pm: &mut PM,
        dm: impl Into<UsbDm>,
        dp: impl Into<UsbDp>,
        nvic: &mut NVIC,
    ) -> Option<&'a mut SerialUSB<'static>> {
        cortex_m::interrupt::free(|_| {
            let used = unsafe { USB_SERIAL.2 };
            if used {
                None
            } else {
                unsafe {
                    USB_SERIAL.2 = true;
                    USB_SERIAL.1 = MaybeUninit::new(usb_allocator(usb, clocks, pm, dm, dp));
                    let allocator = USB_SERIAL.1.assume_init_ref();

                    let serial = SerialPort::new(allocator);
                    let bus = UsbDeviceBuilder::new(allocator, UsbVidPid(0x2222, 0x3333))
                        .manufacturer("Fake company")
                        .product("Serial port")
                        .serial_number("TEST")
                        .device_class(USB_CLASS_CDC)
                        .build();

                    nvic.set_priority(interrupt::USB, 1);
                    NVIC::unmask(interrupt::USB);

                    USB_SERIAL.0 = MaybeUninit::new(SerialUSB { serial, bus });
                    Some(USB_SERIAL.0.assume_init_mut())
                }
            }
        })
    }
}

impl<'a> Write for &'a mut SerialUSB<'a> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        // // loop {
        // match self.serial.write(s.as_bytes()) {
        //     Ok(_) => return Ok(()),
        //     Err(e) => match e {
        //         UsbError::WouldBlock => { /* continue*/ }
        //         _ => return Err(fmt::Error),
        //     },
        // }
        // block!(self.serial.write(s.as_bytes()).map_err(|e| match e {
        //     UsbError::WouldBlock => nb::,
        //     e => e,
        // }));
        // // }
        // Ok(())
        return loop {
            match self.serial.write(s.as_bytes()) {
                Ok(_) => break Ok(()),
                Err(e) => match e {
                    UsbError::WouldBlock => {}
                    _e => break Err(fmt::Error),
                },
            }
        };
    }
}
impl<'a> Serial for &'a mut SerialUSB<'a> {
    fn available(&self) -> bool {
        self.bus.state() == UsbDeviceState::Configured
    }
}

impl<'a> Drop for SerialUSB<'a> {
    fn drop(&mut self) {
        cortex_m::interrupt::free(|_| {
            NVIC::mask(interrupt::USB);
            unsafe {
                USB_SERIAL.0.assume_init_drop();
                USB_SERIAL.1.assume_init_drop();
                USB_SERIAL.2 = false;
            }
        })
    }
}
#[interrupt]
unsafe fn USB() {
    let used = USB_SERIAL.2;
    if used {
        let serial = USB_SERIAL.0.assume_init_mut();
        serial.bus.poll(&mut [&mut serial.serial]);
        let mut buf = [0u8; 16];
        let _ = serial.serial.read(&mut buf);
    }
}

pub struct SerialUART {
    uart: Uart,
}

impl SerialUART {
    pub fn new(
        clocks: &mut GenericClockController,
        sercom5: pac::SERCOM5,
        pm: &mut pac::PM,
        rx: impl Into<Rx>,
        tx: impl Into<Tx>,
    ) -> Self {
        let uart = uart(clocks, Hertz(9600), sercom5, pm, rx, tx);
        SerialUART { uart }
    }
}

impl Write for SerialUART {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for b in s.as_bytes() {
            block!(self.uart.write(*b)).map_err(|_| core::fmt::Error)?;
        }
        Ok(())
    }
}

impl Serial for SerialUART {
    fn available(&self) -> bool {
        true
    }
}
