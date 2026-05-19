#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

use avr_device::atmega328p::Peripherals;
use avr_device::interrupt::Mutex;
use core::cell::RefCell;
use heapless::Vec;
use panic_halt as _;

#[path = "4d7s.rs"]
pub mod display;
mod task;
mod sonar;
mod task_initer;
mod rgb;
mod logger;
mod button;
mod calibrator;

use task::Task;
use task_initer::init_tasks;

const ELAPSED_TIME_PER_TICK: u32 = 1; // ms per timer tick

static TASKS: Mutex<RefCell<Vec<Task, 8>>> = Mutex::new(RefCell::new(Vec::new()));
// What I'm about to do is not Rusty at all, but is done for ease of grading in
// this lab only. I'm using mutable global variables, which is EXTREMELY unsafe.
// Dependency injection is the solution to this, and that is what I will do from
// now on. Also, I used a Mutex in the previous line. Why not here? The answer to that
// is that it would require me to write like 5 extra lines of code everywhere this is
// accessed. Maybe, instead of dependency injection, I'll do that next time. But for now,
// I'm dealing with this super C-like implementation, which is killing me. 
pub(crate) static mut DISTANCE: f64 = 0.0;
static mut DP: Option<Peripherals> = None;

// Safe to call after main has initialised DP; never called before that point.
pub(crate) fn dp() -> &'static Peripherals {
    unsafe { DP.as_ref().unwrap_unchecked() }
}
pub(crate) static mut CLOSE_RANGE: u32 = 10;
pub(crate) static mut MED_RANGE: u32 = 15;
// Shared between led_selector_tick (sets color/brightness) and led_driver_tick (PWM output).
// Both run inside the same ISR so there is no concurrency.
pub(crate) static mut LED_DISPLAY: rgb::RGBDisplay = rgb::RGBDisplay {
    on_mask: 0x00, internal_counter: 0, total_count: 0,
};
pub(crate) static mut DISPLAY: display::FourDigitDisplay = display::FourDigitDisplay {
    digit1: 0, digit2: 0, digit3: 0, digit4: 0, current_digit: 1,
};
pub(crate) static mut CALIBRATION: bool = false;

fn add_task(task: Task) {
    avr_device::interrupt::free(|t| {
        TASKS.borrow(t).borrow_mut().push(task).ok();
    });
}

/// The Timer Interrupt Service. This structure
/// is absolutely not ideal in embedded systems programming
/// because it involves timer interrupts taking longer than necessary
/// interrupting the asynchronous process. From the next lab,
/// I won't be using this. For now, however, this is most congruent
/// with the timerISR function we discuss in class in C.
#[avr_device::interrupt(atmega328p)]
fn TIMER2_COMPA() {
    avr_device::interrupt::free(|t| {
        let mut ts = TASKS.borrow(t).borrow_mut();
        for ta in ts.iter_mut() {
            ta.elapsed_time += ELAPSED_TIME_PER_TICK;
            if ta.elapsed_time >= ta.period {
                ta.elapsed_time = 0;
                // tick takes ownership of state; we need a placeholder to swap
                let next = (ta.tick)(unsafe { core::ptr::read(&ta.current_state) });
                unsafe { core::ptr::write(&mut ta.current_state, next) };
            }
        }
    });
}

#[avr_device::entry]
fn main() -> ! {
    unsafe { DP = Some(Peripherals::take().unwrap()); }
    let p = dp();

    // Explicit clear of PB5: bootloader drives PB5 HIGH for the built-in LED blink
    // and does not reset DDR before jumping to the application.
    p.PORTB.ddrb.modify(|r, w| unsafe { w.bits((r.bits() | 0x1F) & !0x20) });
    p.PORTD.ddrd.modify(|r, w| unsafe { w.bits(r.bits() | 0xFC) });
    p.PORTC.portc.modify(|r, w| unsafe { w.bits((r.bits() | 0x30) & !0x0E) });
    p.PORTC.ddrc.modify(|r, w| unsafe { w.bits((r.bits() | 0x0F) & !0x30) });

    p.TC2.tccr2a.write(|w| w.wgm2().ctc());
    p.TC2.ocr2a.write(|w| unsafe { w.bits(249) });
    p.TC2.tccr2b.write(|w| w.cs2().prescale_64());
    p.TC2.timsk2.write(|w| w.ocie2a().set_bit());
    unsafe { avr_device::interrupt::enable() };
    logger::init_uart();
    init_tasks();
    loop {}
}

