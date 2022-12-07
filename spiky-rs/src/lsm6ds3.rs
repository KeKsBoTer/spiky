use arduino_nano33iot::{
    hal::{
        clock::GenericClockController,
        sercom::{I2CError, I2CMaster4},
    },
    i2c_master, pac, Scl, Sda,
};

use arduino_nano33iot::hal::prelude::*;
const LSM6DS3_ADDRESS: u8 = 0x6A;

const LSM6DS3_WHO_AM_I_REG: u8 = 0x0F;

const LSM6DS3_CTRL1_XL: u8 = 0x10;
const LSM6DS3_CTRL2_G: u8 = 0x11;
const LSM6DS3_CTRL6_C: u8 = 0x15;
const LSM6DS3_CTRL7_G: u8 = 0x16;
const LSM6DS3_CTRL8_XL: u8 = 0x17;
const LSM6DS3_STATUS_REG: u8 = 0x1E;

const LSM6DS3_OUTX_L_G: u8 = 0x22;
const LSM6DS3_OUTX_H_G: u8 = 0x23;

const LSM6DS3_OUTY_L_G: u8 = 0x24;
const LSM6DS3_OUTY_H_G: u8 = 0x25;

const LSM6DS3_OUTZ_L_G: u8 = 0x26;
const LSM6DS3_OUTZ_H_G: u8 = 0x27;

const LSM6DS3_OUTX_L_XL: u8 = 0x28;
const LSM6DS3_OUTX_H_XL: u8 = 0x29;

const LSM6DS3_OUTY_L_XL: u8 = 0x2A;
const LSM6DS3_OUTY_H_XL: u8 = 0x2B;

const LSM6DS3_OUTZ_L_XL: u8 = 0x2C;
const LSM6DS3_OUTZ_H_XL: u8 = 0x2D;

pub struct LSM6DS3 {
    i2c: I2CMaster4<Sda, Scl>,
}

#[derive(Debug)]
pub enum LSM6DS3Error {
    I2C(I2CError),
    WhoAmI,
}

impl LSM6DS3 {
    pub fn new(
        clocks: &mut GenericClockController,
        sercom4: pac::SERCOM4,
        pm: &mut pac::PM,
        sda: impl Into<Sda>,
        scl: impl Into<Scl>,
    ) -> Result<LSM6DS3, LSM6DS3Error> {
        let mut i2c = i2c_master(clocks, 10.khz(), sercom4, pm, sda, scl);
        let mut buf = [0u8; 1];

        let mut imu = LSM6DS3 { i2c };

        match imu.read_register(LSM6DS3_WHO_AM_I_REG) {
            Ok(who_am_i) => {
                if who_am_i != 0x69 {
                    return Err(LSM6DS3Error::WhoAmI);
                }
            }
            Err(err) => return Err(LSM6DS3Error::I2C(err)),
        };

        //set the gyroscope control register to work at 104 Hz, 2000 dps and in bypass mode
        imu.write_register(LSM6DS3_CTRL2_G, 0x4C)
            .map_err(|err| LSM6DS3Error::I2C(err))?;

        // Set the Accelerometer control register to work at 104 Hz, 4 g,and in bypass mode and enable ODR/4
        // low pass filter (check figure9 of LSM6DS3's datasheet)
        imu.write_register(LSM6DS3_CTRL1_XL, 0x4A)
            .map_err(|err| LSM6DS3Error::I2C(err))?;

        // set gyroscope power mode to high performance and bandwidth to 16 MHz
        imu.write_register(LSM6DS3_CTRL7_G, 0x00)
            .map_err(|err| LSM6DS3Error::I2C(err))?;

        // Set the ODR config register to ODR/4
        imu.write_register(LSM6DS3_CTRL8_XL, 0x09)
            .map_err(|err| LSM6DS3Error::I2C(err))?;

        return Ok(imu);
    }

    pub fn read_acceleration(&mut self, acc: &mut [f32; 3]) -> Result<(), I2CError> {
        let mut buf = [0u8; 6];
        self.read_registers(LSM6DS3_OUTX_L_XL, &mut buf)?;

        let x = u16::from_le_bytes([buf[0], buf[1]]);
        let y = u16::from_le_bytes([buf[2], buf[3]]);
        let z = u16::from_le_bytes([buf[4], buf[5]]);
        acc[0] = x as f32 * 4.0 / 32768.0;
        acc[1] = y as f32 * 4.0 / 32768.0;
        acc[2] = z as f32 * 4.0 / 32768.0;
        Ok(())
    }

    pub fn acceleration_available(&mut self) -> Result<bool, I2CError> {
        match self.read_register(LSM6DS3_STATUS_REG) {
            Ok(code) => Ok((code & 0x01) == 1),
            Err(err) => Err(err),
        }
    }

    pub fn acceleration_sample_rate(&self) -> f32 {
        return 104.0;
    }

    fn read_register(&mut self, address: u8) -> Result<u8, I2CError> {
        let mut buf = [0u8; 1];
        self.read_registers(address, &mut buf)?;
        return Ok(buf[0]);
    }
    fn read_registers(&mut self, address: u8, data: &mut [u8]) -> Result<(), I2CError> {
        self.i2c.write_read(LSM6DS3_ADDRESS, &[address], data)
    }

    fn write_register(&mut self, address: u8, value: u8) -> Result<(), I2CError> {
        self.i2c.write(LSM6DS3_ADDRESS, &[address, value])
    }
}

impl Drop for LSM6DS3 {
    fn drop(&mut self) {
        self.i2c.write(LSM6DS3_CTRL2_G, &[0x00]).unwrap();
        self.i2c.write(LSM6DS3_CTRL1_XL, &[0x00]).unwrap();
    }
}
