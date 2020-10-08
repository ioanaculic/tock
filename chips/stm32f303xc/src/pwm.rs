//
// structura pwm
// new -> array pini
// in functie de pini, trimit la un tim anume

use crate::gpio;
use crate::tim4;
use kernel::hil;
use kernel::hil::gpio::Configure;
use kernel::ReturnCode;

pub struct TimPwmPin<'a> {
    pub(crate) pin: &'a gpio::Pin<'a>,
    pub(crate) channel: u8,
    tim: &'a dyn hil::pwm::Pwm<Pin = Self>,
}

impl<'a> TimPwmPin<'a> {
    pub fn new(
        pin: &'a gpio::Pin<'a>,
        channel: u8,
        tim: &'a dyn hil::pwm::Pwm<Pin = Self>,
    ) -> TimPwmPin<'a> {
        unsafe {
            if tim as *const _ as *const u8 == &tim4::TIM4 as *const _ as *const u8 {
                if channel < 1 && channel > 4 {
                    panic!("PWM channel not valid!")
                }
                pin.make_output();
                pin.set_mode(gpio::Mode::AlternateFunctionMode);
                pin.set_alternate_function(gpio::AlternateFunction::AF2);
                pin.set_speed(gpio::Speed::HighSpeed);
            }
        }
        TimPwmPin {
            pin: pin,
            channel: channel,
            tim: tim,
        }
    }
}
