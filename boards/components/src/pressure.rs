//! Component for any Pressure sensor.
//!
//! Usage
//! -----
//! ```rust
//! let press = PressureComponent::new(board_kernel, nrf52::pressure::TEMP).finalize(());
//! ```

use capsules::pressure::PressureSensor;
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil;
use kernel::static_init;

pub struct PressureComponent<P: 'static + hil::sensors::PressureDriver<'static>> {
    board_kernel: &'static kernel::Kernel,
    press_sensor: &'static P,
}

impl<P: 'static + hil::sensors::PressureDriver<'static>> PressureComponent<P> {
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        press_sensor: &'static P,
    ) -> PressureComponent<P> {
        PressureComponent {
            board_kernel,
            press_sensor,
        }
    }
}

impl<P: 'static + hil::sensors::PressureDriver<'static>> Component for PressureComponent<P> {
    type StaticInput = ();
    type Output = &'static PressureSensor<'static>;

    unsafe fn finalize(self, _s: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        let press = static_init!(
            PressureSensor<'static>,
            PressureSensor::new(
                self.press_sensor,
                self.board_kernel.create_grant(&grant_cap)
            )
        );

        hil::sensors::PressureDriver::set_client(self.press_sensor, press);
        press
    }
}
