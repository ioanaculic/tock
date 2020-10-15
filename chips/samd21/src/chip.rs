//! Interrupt mapping and DMA channel setup.

use core::fmt::Write;
use cortexm0;
use kernel::common::deferred_call;
use kernel::Chip;

pub struct Samd21 {
    mpu: (),
    userspace_kernel_boundary: cortexm0::syscall::SysCall,
    scheduler_timer: cortexm0::systick::SysTick,
}

impl Samd21 {
    pub unsafe fn new() -> Samd21 {
        // usart::USART0.set_dma(&dma::DMA_CHANNELS[0], &dma::DMA_CHANNELS[1]);
        // dma::DMA_CHANNELS[0].initialize(&mut usart::USART0, dma::DMAWidth::Width8Bit);
        // dma::DMA_CHANNELS[1].initialize(&mut usart::USART0, dma::DMAWidth::Width8Bit);

        // usart::USART1.set_dma(&dma::DMA_CHANNELS[2], &dma::DMA_CHANNELS[3]);
        // dma::DMA_CHANNELS[2].initialize(&mut usart::USART1, dma::DMAWidth::Width8Bit);
        // dma::DMA_CHANNELS[3].initialize(&mut usart::USART1, dma::DMAWidth::Width8Bit);

        // usart::USART2.set_dma(&dma::DMA_CHANNELS[4], &dma::DMA_CHANNELS[5]);
        // dma::DMA_CHANNELS[4].initialize(&mut usart::USART2, dma::DMAWidth::Width8Bit);
        // dma::DMA_CHANNELS[5].initialize(&mut usart::USART2, dma::DMAWidth::Width8Bit);

        // usart::USART3.set_dma(&dma::DMA_CHANNELS[6], &dma::DMA_CHANNELS[7]);
        // dma::DMA_CHANNELS[6].initialize(&mut usart::USART3, dma::DMAWidth::Width8Bit);
        // dma::DMA_CHANNELS[7].initialize(&mut usart::USART3, dma::DMAWidth::Width8Bit);

        // spi::SPI.set_dma(&dma::DMA_CHANNELS[8], &dma::DMA_CHANNELS[9]);
        // dma::DMA_CHANNELS[8].initialize(&mut spi::SPI, dma::DMAWidth::Width8Bit);
        // dma::DMA_CHANNELS[9].initialize(&mut spi::SPI, dma::DMAWidth::Width8Bit);

        // i2c::I2C0.set_dma(&dma::DMA_CHANNELS[10]);
        // dma::DMA_CHANNELS[10].initialize(&mut i2c::I2C0, dma::DMAWidth::Width8Bit);

        // i2c::I2C1.set_dma(&dma::DMA_CHANNELS[11]);
        // dma::DMA_CHANNELS[11].initialize(&mut i2c::I2C1, dma::DMAWidth::Width8Bit);

        // i2c::I2C2.set_dma(&dma::DMA_CHANNELS[12]);
        // dma::DMA_CHANNELS[12].initialize(&mut i2c::I2C2, dma::DMAWidth::Width8Bit);

        // adc::ADC0.set_dma(&dma::DMA_CHANNELS[13]);
        // dma::DMA_CHANNELS[13].initialize(&mut adc::ADC0, dma::DMAWidth::Width16Bit);

        Samd21 {
            mpu: (),
            userspace_kernel_boundary: cortexm0::syscall::SysCall::new(),
            scheduler_timer: cortexm0::systick::SysTick::new(),
        }
    }
}

impl Chip for Samd21 {
    type MPU = ();
    type UserspaceKernelBoundary = cortexm0::syscall::SysCall;
    type SchedulerTimer = cortexm0::systick::SysTick;
    type WatchDog = ();

    fn service_pending_interrupts(&self) {
        unsafe {
            loop {
                // if let Some(task) = deferred_call::DeferredCall::next_pending() {
                //     match task {
                //         Task::Flashcalw => flashcalw::FLASH_CONTROLLER.handle_interrupt(),
                //     }
                // } else 
                if let Some(interrupt) = cortexm0::nvic::next_pending() {
                    match interrupt {
                        _ => {
                            panic!("unhandled interrupt {}", interrupt);
                        }
                    }
                    let n = cortexm0::nvic::Nvic::new(interrupt);
                    n.clear_pending();
                    n.enable();
                } else {
                    break;
                }
            }
        }
    }

    fn has_pending_interrupts(&self) -> bool {
        unsafe { cortexm0::nvic::has_pending() || deferred_call::has_tasks() }
    }

    fn mpu(&self) -> &() {
        &self.mpu
    }

    fn scheduler_timer(&self) -> &Self::SchedulerTimer {
        &self.scheduler_timer
    }

    fn watchdog(&self) -> &Self::WatchDog {
        &()
    }

    fn userspace_kernel_boundary(&self) -> &cortexm0::syscall::SysCall {
        &self.userspace_kernel_boundary
    }

    fn sleep(&self) {
        // if pm::deep_sleep_ready() {
        //     unsafe {
        //         cortexm0::scb::set_sleepdeep();
        //     }
        // } else {
        //     unsafe {
        //         cortexm0::scb::unset_sleepdeep();
        //     }
        // }

        unsafe {
            cortexm0::support::wfi();
        }
    }

    unsafe fn atomic<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        cortexm0::support::atomic(f)
    }

    unsafe fn print_state(&self, writer: &mut dyn Write) {
        cortexm0::print_cortexm0_state(writer);
    }
}
