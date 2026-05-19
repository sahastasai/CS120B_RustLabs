#![no_std]
#![no_main]
#![feature(asm_experimental_arch)]

use avr_device::atmega328p::Peripherals;
use panic_halt as _;
use crate::task;
const ELAPSED_TIME_PER_TICK: u32 = 1;


/// The Timer Interrupt Service. This structure
/// is absolutely not ideal in embedded systems programming
/// because it involves timer interrupts taking longer than necessary
/// interrupting the asynchronous process. From the next lab,
/// I won't be using this. For now, however, this is most congruent
/// with the timerISR function we discuss in class in C.
#[avr_device::interrupt(atmega328p)]
fn TIMER1_COMPA() {
    avr_device::interrupt::free(
            |t| {
                let ts = TASKS
                    .borrow(t)
                    .borrow();

                for ta in ts.iter() {
                    ta.elapsed_time += ELAPSED_TIME_PER_TICK;
                    if(ta.elapsed_time >= ta.period) {
                        ta.current_state = ta.tick(ta.current_state);
                    }

                }
            }
        );
}

