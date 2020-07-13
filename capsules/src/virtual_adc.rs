use kernel::common::cells::OptionalCell;
use kernel::common::{List, ListLink, ListNode};
use kernel::hil;
use kernel::ReturnCode;

pub struct MuxAdc<'a, A: hil::adc::Adc> {
    adc: &'a A,
    devices: List<'a, AdcUser<'a A>>,
    inflight: OptionalCell<&'a AdcUser<'a A>>,
}

impl<'a, A: hil::adc::Adc> MuxAdc<'a A> {
    pub const fn new(adc: &'a A) -> MuxAdc<'a, A> {
        MuxAdc {
            adc: adc,
            devices: List::new(),
            inflight: OptionalCell::empty(),
        }
    }

    fn do_next_op(&self) {
        if self.inflight.is_none() {
            let mnode = self.devices.iter().find(|node| node.operation.is_some());
            mnode.map(|node| {
                let started = node.operation.take().map_or(false, |operation| {
                    match operation {
                        Operation::SingleSample => {
                            self.adc.sample(&node.channel);
                            true
                        }
                        Operation::Idle => {
                            false
                        }
                    }
                });
                if started {
                    self.inflight.set(node);
                } else {
                    self.do_next_op();
                }
            });
        } else {
            self.inflight.map(|node| {
                node.operation.take().map(|operation| {
                    match operation {
                        Operation::SingleSample => {
                            self.adc.sample(&node.channel);
                        }
                        Operation::Idle => {
                            self.adc.stop();
                            self.inflight.clear();
                        }
                    }
                    self.do_next_op();
                });
            });
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
enum Operation {
    SingleSample,
    Idle,
}

pub struct AdcUser<'a, A: hil::adc::Adc> {
    mux: &'a MuxAdc<'a, A>,
    channel: A::Channel,
    operation: OptionalCell<Operation>,
    next: ListLink<'a, AdcUser<'a, A>>,
    client: OptionalCell<&'a dyn hil::adc::Client>,
}

impl<'a, A: hil::adc::Adc> AdcUser<'a, A> {
    pub const fn new(mux: &'a MuxAdc<'a, A>, channel: A::Channel) -> AdcUser<'a, A> {
        AdcUser {
            mux: mux,
            channel: channel,
            operation: OptionalCell::empty(),
            next: ListLink::empty(),
            client: OptionalCell::empty(),
        }
    }

    pub fn add_to_mux(&'a self) {
        self.mux.devices.push_head(self);
    }
}

impl<'a, A: hil::adc::Adc> ListNode<'a, AdcUser<'a, A>> for AdcUser<'a, A> {
    fn next(&'a self) -> &'a ListLink<'a, AdcUser<'a, A>> {
        &self.next
    }
}

impl<A: hil::adc::Adc> hil::adc::AdcChannel for AdcUser<'_, A> {
    fn sample(&self, channel: usize) -> ReturnCode {
        self.operation.set(Operation::SingleSample);
        self.mux.do_next_op();
        ReturnCode::SUCCESS
    }

    fn stop(&self) -> ReturnCode {
        self.operation.set(Operation::Idle);
        self.mux.do_next_op();
        ReturnCode::SUCCESS
    }

    fn get_channel(&self) -> usize {
        self.mux.pwm.get_channel()
    }
}