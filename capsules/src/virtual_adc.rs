use kernel::common::cells::OptionalCell;
use kernel::common::{List, ListLink, ListNode};
use kernel::hil;
use kernel::ReturnCode;

pub struct MuxAdc<'a, A: hil::adc::Adc> {
    adc: &'a A,
    devices: List<'a, AdcUser<'a, A>>,
    inflight: OptionalCell<&'a AdcUser<'a, A>>,
}

impl<'a, A: hil::adc::Adc> hil::adc::Client for MuxAdc<'a, A> {
    fn sample_ready(&self, sample: u16) {
        for node in self.devices.iter() {
            self.inflight.map(|inflight| {
                if node.channel == inflight.channel {
                    node.operation.map(|operation| match operation {
                        Operation::SingleSample => {
                            node.operation.clear();
                            node.client.map(|client| client.sample_ready(sample));
                        }
                    });
                }
            });
        }
        self.inflight.clear();
        self.do_next_op();
    }
}

impl<'a, A: hil::adc::Adc> MuxAdc<'a, A> {
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
                let started = node.operation.map_or(false, |operation| match operation {
                    Operation::SingleSample => {
                        self.adc.sample(&node.channel);
                        true
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
        let adc_user = AdcUser {
            mux: mux,
            channel: channel,
            operation: OptionalCell::empty(),
            next: ListLink::empty(),
            client: OptionalCell::empty(),
        };
        adc_user
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
    fn sample(&self) -> ReturnCode {
        self.operation.set(Operation::SingleSample);
        self.mux.do_next_op();
        ReturnCode::SUCCESS
    }

    fn stop_sampling(&self) -> ReturnCode {
        self.operation.clear();
        self.mux.do_next_op();
        ReturnCode::SUCCESS
    }

    fn sample_continuous(&self) -> ReturnCode {
        ReturnCode::ENOSUPPORT
    }

    fn get_resolution_bits(&self) -> usize {
        12
    }

    fn get_voltage_reference_mv(&self) -> Option<usize> {
        Some(3300)
    }
    fn set_client(&self, client: &'static dyn hil::adc::Client) {
        self.client.set(client);
    }
}
