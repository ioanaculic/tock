//! Driver for BMP280 Temperature and Pressure Sensor
//!
//! Author: Cosmin Daniel Radu <cosmindanielradu19@gmail.com>
//!
//!

use core::cell::Cell;
use enum_primitive::cast::FromPrimitive;
use enum_primitive::enum_from_primitive;
use kernel::common::cells::{OptionalCell, TakeCell};
use kernel::hil::i2c;
use kernel::ReturnCode;

pub static BASE_ADDR: u8 = 0x77;

#[repr(u16)]
enum_from_primitive! {
    enum Registers {
        DIGT1 = 0x88,
        DIGT2 = 0x8A,
        DIGT3 = 0x8C,
        DIGP1 = 0x8E,
        DIGP2 = 0x90,
        DIGP3 = 0x92,
        DIGP4 = 0x94,
        DIGP5 = 0x96,
        DIGP6 = 0x98,
        DIGP7 = 0x9A,
        DIGP8 = 0x9C,
        DIGP9 = 0x9E,
        CHIPID = 0xD0,
        VERSION = 0xD1,
        SOFTRESET = 0xE0,
        /// R calibration = 0xE1-0xF0
        CAL26 = 0xE1,
        STATUS = 0xF3,
        CONTROL = 0xF4,
        CONFIG = 0xF5,
        PRESSUREDATA = 0xF7,
        TEMPDATA = 0xFA,
    }
}
#[repr(u16)]
enum_from_primitive! {
    pub enum SensorSampling {
        /// No over-sampling.
        SAMPLINGNONE = 0x00,
        /// 1x over-sampling.
        SAMPLINGX1 = 0x01,
        /// 2x over-sampling.
        SAMPLINGX2 = 0x02,
        /// 4x over-sampling.
        SAMPLINGX4 = 0x03,
        /// 8x over-sampling.
        SAMPLINGX8 = 0x04,
        /// 16x over-sampling.
        SAMPLINGX16 = 0x05
    }
}
#[repr(u16)]
enum_from_primitive! {
    pub enum SensorMode {
        /// Sleep mode.
        SLEEP = 0x00,
        /// Forced mode.
        FORCED = 0x01,
        /// Normal mode.
        NORMAL = 0x03,
        /// Software reset.
        SOFTRESET = 0xB6
    }
}
#[repr(u16)]
enum_from_primitive! {
    pub enum SensorFilter {
        /// No filtering.
        FILTEROFF = 0x00,
        /// 2x filtering.
        FILTERX2 = 0x01,
        /// 4x filtering.
        FILTERX4 = 0x02,
        /// 8x filtering.
        FILTERX8 = 0x03,
        /// 16x filtering.
        FILTERX16 = 0x04
    }
}
#[repr(u16)]
enum_from_primitive! {
    pub enum StandbyDuration {
        /// 1 ms standby.
        STANDBY1 = 0x00,
        /// 62.5 ms standby.
        STANDBY63 = 0x01,
        /// 125 ms standby.
        STANDBY125 = 0x02,
        /// 250 ms standby.
        STANDBY250 = 0x03,
        /// 500 ms standby.
        STANDBY500 = 0x04,
        /// 1000 ms standby.
        STANDBY1000 = 0x05,
        /// 2000 ms standby.
        STANDBY2000 = 0x06,
        /// 4000 ms standby.
        STANDBY4000 = 0x07
    }
}

#[derive(Clone, Copy, PartialEq)]
enum State {
    Init,
    Reset,
    ReadTemperature,
    ReadPressure,
    Idle,
    ReadStatus,
    READDIGT1,
    READDIGT2,
    READDIGT3,
    READDIGP1,
    READDIGP2,
    READDIGP3,
    READDIGP4,
    READDIGP5,
    READDIGP6,
    READDIGP7,
    READDIGP8,
    READDIGP9,
}

pub struct BMP280<'a> {
    i2c: &'a dyn i2c::I2CDevice,
    pressure_client: OptionalCell<&'a dyn kernel::hil::sensors::PressureClient>,
    temperature_client: OptionalCell<&'a dyn kernel::hil::sensors::TemperatureClient>,
    state: Cell<State>,
    buffer: TakeCell<'static, [u8]>,
    read_temp: Cell<bool>,
    read_press: Cell<bool>,
    t_fine: Cell<i32>,
    dig_t1: Cell<u16>,
    dig_t2: Cell<i16>,
    dig_t3: Cell<i16>,
    dig_p1: Cell<u16>,
    dig_p2: Cell<i16>,
    dig_p3: Cell<i16>,
    dig_p4: Cell<i16>,
    dig_p5: Cell<i16>,
    dig_p6: Cell<i16>,
    dig_p7: Cell<i16>,
    dig_p8: Cell<i16>,
    dig_p9: Cell<i16>,
}

impl<'a> BMP280<'a> {
    pub fn new(i2c: &'a dyn i2c::I2CDevice, buffer: &'static mut [u8]) -> BMP280<'a> {
        BMP280 {
            i2c: i2c,
            pressure_client: OptionalCell::empty(),
            temperature_client: OptionalCell::empty(),
            state: Cell::new(State::Idle),
            buffer: TakeCell::new(buffer),
            read_temp: Cell::new(false),
            read_press: Cell::new(false),
            t_fine: Cell::new(0),
            dig_t1: Cell::new(0),
            dig_t2: Cell::new(0),
            dig_t3: Cell::new(0),
            dig_p1: Cell::new(0),
            dig_p2: Cell::new(0),
            dig_p3: Cell::new(0),
            dig_p4: Cell::new(0),
            dig_p5: Cell::new(0),
            dig_p6: Cell::new(0),
            dig_p7: Cell::new(0),
            dig_p8: Cell::new(0),
            dig_p9: Cell::new(0),
        }
    }

    pub fn read_status(&self) {
        self.buffer.take().map(|buffer| {
            self.i2c.enable();

            buffer[0] = Registers::STATUS as u8;
            self.i2c.write_read(buffer, 1, 1);

            self.state.set(State::ReadStatus);
        });
    }

    pub fn reset(&self) {
        self.buffer.take().map(|buffer| {
            self.i2c.enable();

            let [high, low] = u16::to_be_bytes(Registers::SOFTRESET as u16);

            buffer[0] = high;
            buffer[1] = low;

            self.i2c.write_read(buffer, 2, 2);
            self.state.set(State::Reset);
        });
    }

    pub fn set_sampling(
        &self,
        mode: SensorMode,
        temp_sampling: SensorSampling,
        press_sampling: SensorSampling,
        filter: SensorFilter,
        duration: StandbyDuration,
    ) -> ReturnCode {
        let mut meas_reg: u8 = 0;
        meas_reg |= ((mode as u8) << 5) as u8;
        meas_reg |= ((temp_sampling as u8) << 2) as u8;
        meas_reg |= ((press_sampling as u8) << 5) as u8;

        let mut config_reg: u8 = 0;
        config_reg |= ((filter as u8) << 5) as u8;
        config_reg |= ((duration as u8) << 2) as u8;

        self.buffer.take().map_or(ReturnCode::ENOMEM, |buffer| {
            self.i2c.enable();

            self.state.set(State::Init);

            buffer[0] = Registers::CONFIG as u8;
            buffer[1] = config_reg;
            buffer[2] = Registers::CONTROL as u8;
            buffer[3] = meas_reg;

            self.i2c.write(buffer, 4);

            ReturnCode::SUCCESS
        })
    }

    fn read_dig(&self, reg: Registers, state: State) -> ReturnCode {
        self.buffer.take().map_or(ReturnCode::ENOMEM, |buffer| {
            self.i2c.enable();

            self.state.set(state);

            buffer[0] = reg as u8;

            self.i2c.write_read(buffer, 1, 2);

            ReturnCode::SUCCESS
        })
    }

    fn get_sample(&self, reg: Registers, state: State) -> ReturnCode {
        self.buffer.take().map_or(ReturnCode::ENOMEM, |buffer| {
            self.state.set(state);
            self.i2c.enable();

            buffer[0] = reg as u8;

            self.i2c.write_read(buffer, 1, 3);

            ReturnCode::SUCCESS
        })
    }

    fn read_temperature(&self) -> ReturnCode {
        if self.read_temp.get() == true {
            ReturnCode::EBUSY
        } else {
            if self.state.get() == State::Idle {
                self.read_temp.set(true);
                self.get_sample(Registers::TEMPDATA, State::ReadTemperature)
            } else {
                self.read_temp.set(true);
                ReturnCode::SUCCESS
            }
        }
    }

    fn read_pressure(&self) -> ReturnCode {
        if self.read_press.get() == true {
            ReturnCode::EBUSY
        } else {
            if self.state.get() == State::Idle {
                self.read_press.set(true);
                self.get_sample(Registers::TEMPDATA, State::ReadTemperature)
            } else {
                self.read_press.set(true);
                ReturnCode::SUCCESS
            }
        }
    }
}

impl i2c::I2CClient for BMP280<'_> {
    fn command_complete(&self, buffer: &'static mut [u8], error: i2c::Error) {
        match error {
            i2c::Error::CommandComplete => {
                let state = self.state.get();

                match state {
                    State::Init => {
                        self.buffer.replace(buffer);
                        self.read_dig(Registers::DIGT1, State::READDIGT1);
                    }
                    State::ReadStatus => {
                        // TODO do soemthing useful with the status
                        self.state.set(State::Idle);
                    }
                    State::ReadTemperature => {
                        let mut adc_t = buffer[0] as i32;
                        adc_t = adc_t << 8;
                        adc_t = adc_t | buffer[1] as i32;
                        adc_t = adc_t << 8;
                        adc_t = adc_t | buffer[2] as i32;
                        adc_t = adc_t >> 4;

                        let var1 = (((adc_t >> 3) - ((self.dig_t1.get() as i32) << 1))
                            * (self.dig_t2.get() as i32))
                            >> 11;

                        let var2 = (((((adc_t >> 4) - (self.dig_t1.get() as i32))
                            * ((adc_t >> 4) - (self.dig_t1.get() as i32)))
                            >> 12)
                            * (self.dig_t3.get() as i32))
                            >> 14;

                        self.t_fine.set(var1 + var2);

                        self.buffer.replace(buffer);
                        if self.read_temp.get() == true {
                            self.read_temp.set(false);
                            let stemp = (self.t_fine.get() * 5 + 128) >> 8;
                            self.temperature_client
                                .map(|cb| cb.callback(stemp as usize));
                        }
                        if self.read_press.get() == true {
                            self.get_sample(Registers::PRESSUREDATA, State::ReadPressure);
                        } else {
                            self.state.set(State::Idle);
                        }
                    }
                    State::ReadPressure => {
                        let mut adc_p = buffer[0] as i32;
                        adc_p = adc_p << 8;
                        adc_p = adc_p | buffer[1] as i32;
                        adc_p = adc_p << 8;
                        adc_p = adc_p | buffer[2] as i32;
                        adc_p = adc_p >> 4;

                        let mut var1 = (self.t_fine.get() as i64) - 128000;
                        let mut var2 = var1 * var1 * (self.dig_p6.get() as i64);
                        var2 = var2 + ((var1 * (self.dig_p5.get() as i64)) << 17);
                        var2 = var2 + ((self.dig_p4.get() as i64) << 35);
                        var1 = ((var1 * var1 * (self.dig_p3.get() as i64)) >> 8)
                            + ((var1 * (self.dig_p2.get() as i64)) << 12);

                        if var1 == 0 {
                            self.read_press.set(false);
                            self.state.set(State::Idle);
                            self.pressure_client.map(|cb| cb.callback(usize::MAX));
                        } else {
                            let mut pressure: i64 = (1048576 - adc_p).into();
                            pressure = (((pressure << 31) - var2) * 3125) / var1;
                            var1 =
                                ((self.dig_p9.get() as i64) * (pressure >> 13) * (pressure >> 13))
                                    >> 25;
                            var2 = ((self.dig_p8.get() as i64) * pressure) >> 19;
                            pressure =
                                ((pressure + var1 + var2) >> 8) + ((self.dig_p7.get() as i64) << 4);
                            pressure = pressure / 256 * 100;
                            self.read_press.set(false);
                            self.state.set(State::Idle);
                            self.pressure_client
                                .map(|cb| cb.callback(pressure as usize));
                        }
                    }
                    State::READDIGT1 => {
                        let mut tmp: u16 = ((buffer[1] as u16) << 8) as u16;
                        tmp |= (buffer[0]) as u16;
                        self.dig_t1.set(tmp);
                        self.buffer.replace(buffer);
                        self.read_dig(Registers::DIGT2, State::READDIGT2);
                    }
                    State::READDIGT2 => {
                        let mut tmp: u16 = ((buffer[1] as u16) << 8) as u16;
                        tmp |= (buffer[0]) as u16;
                        self.dig_t2.set(tmp as i16);
                        self.buffer.replace(buffer);
                        self.read_dig(Registers::DIGT3, State::READDIGT3);
                    }
                    State::READDIGT3 => {
                        let mut tmp: u16 = ((buffer[1] as u16) << 8) as u16;
                        tmp |= (buffer[0]) as u16;
                        self.dig_t3.set(tmp as i16);
                        self.buffer.replace(buffer);
                        self.read_dig(Registers::DIGP1, State::READDIGP1);
                    }
                    State::READDIGP1 => {
                        let mut tmp: u16 = ((buffer[1] as u16) << 8) as u16;
                        tmp |= (buffer[0]) as u16;
                        self.dig_p1.set(tmp);
                        self.buffer.replace(buffer);
                        self.read_dig(Registers::DIGP2, State::READDIGP2);
                    }
                    State::READDIGP2 => {
                        let mut tmp: u16 = ((buffer[1] as u16) << 8) as u16;
                        tmp |= (buffer[0]) as u16;
                        self.dig_p2.set(tmp as i16);
                        self.buffer.replace(buffer);
                        self.read_dig(Registers::DIGP3, State::READDIGP3);
                    }
                    State::READDIGP3 => {
                        let mut tmp: u16 = ((buffer[1] as u16) << 8) as u16;
                        tmp |= (buffer[0]) as u16;
                        self.dig_p3.set(tmp as i16);
                        self.buffer.replace(buffer);
                        self.read_dig(Registers::DIGP4, State::READDIGP4);
                    }
                    State::READDIGP4 => {
                        let mut tmp: u16 = ((buffer[1] as u16) << 8) as u16;
                        tmp |= (buffer[0]) as u16;
                        self.dig_p4.set(tmp as i16);
                        self.buffer.replace(buffer);
                        self.read_dig(Registers::DIGP4, State::READDIGP5);
                    }
                    State::READDIGP5 => {
                        let mut tmp: u16 = ((buffer[1] as u16) << 8) as u16;
                        tmp |= (buffer[0]) as u16;
                        self.dig_p5.set(tmp as i16);
                        self.buffer.replace(buffer);
                        self.read_dig(Registers::DIGP6, State::READDIGP6);
                    }
                    State::READDIGP6 => {
                        let mut tmp: u16 = ((buffer[1] as u16) << 8) as u16;
                        tmp |= (buffer[0]) as u16;
                        self.dig_p6.set(tmp as i16);
                        self.buffer.replace(buffer);
                        self.read_dig(Registers::DIGP7, State::READDIGP7);
                    }
                    State::READDIGP7 => {
                        let mut tmp: u16 = ((buffer[1] as u16) << 8) as u16;
                        tmp |= (buffer[0]) as u16;
                        self.dig_p7.set(tmp as i16);
                        self.buffer.replace(buffer);
                        self.read_dig(Registers::DIGP8, State::READDIGP8);
                    }
                    State::READDIGP8 => {
                        let mut tmp: u16 = ((buffer[1] as u16) << 8) as u16;
                        tmp |= (buffer[0]) as u16;
                        self.dig_p8.set(tmp as i16);
                        self.buffer.replace(buffer);
                        self.read_dig(Registers::DIGP9, State::READDIGP9);
                    }
                    State::READDIGP9 => {
                        let mut tmp: u16 = ((buffer[1] as u16) << 8) as u16;
                        tmp |= (buffer[0]) as u16;
                        self.dig_p9.set(tmp as i16);
                        self.buffer.replace(buffer);
                        self.state.set(State::Idle);
                    }
                    State::Reset => {
                        self.buffer.replace(buffer);
                        self.state.set(State::Idle);
                    }
                    _ => {}
                }
            }
            _ => {
                self.buffer.replace(buffer);
                self.i2c.disable();
                if self.read_temp.get() == true {
                    self.read_temp.set(false);
                    self.temperature_client.map(|cb| cb.callback(usize::MAX));
                }
                if self.read_press.get() == true {
                    self.read_press.set(false);
                    self.pressure_client.map(|cb| cb.callback(usize::MAX));
                }
            }
        }
    }
}

impl<'a> kernel::hil::sensors::PressureDriver<'a> for BMP280<'a> {
    fn set_client(&self, client: &'a dyn kernel::hil::sensors::PressureClient) {
        self.pressure_client.set(client);
    }

    fn read_pressure(&self) -> ReturnCode {
        self.read_pressure()
    }
}

impl<'a> kernel::hil::sensors::TemperatureDriver<'a> for BMP280<'a> {
    fn set_client(&self, client: &'a dyn kernel::hil::sensors::TemperatureClient) {
        self.temperature_client.set(client);
    }

    fn read_temperature(&self) -> ReturnCode {
        self.read_temperature()
    }
}
