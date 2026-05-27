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
pub mod lcd;
pub mod joystick;
pub mod stepper;

use task::Task;

const ELAPSED_TIME_PER_TICK: u32 = 1;

static TASKS: Mutex<RefCell<Vec<Task, 8>>> = Mutex::new(RefCell::new(Vec::new()));

pub(crate) static mut DISTANCE: f64 = 0.0;
static mut DP: Option<Peripherals> = None;

pub(crate) fn dp() -> &'static Peripherals {
    unsafe { DP.as_ref().unwrap_unchecked() }
}

pub(crate) static mut CLOSE_RANGE: u32 = 10;
pub(crate) static mut MED_RANGE: u32 = 15;
pub(crate) static mut LED_DISPLAY: rgb::RGBDisplay = rgb::RGBDisplay {
    on_mask: 0x00, internal_counter: 0, total_count: 0,
};
pub(crate) static mut DISPLAY: display::FourDigitDisplay = display::FourDigitDisplay {
    digit1: 0, digit2: 0, digit3: 0, digit4: 0, current_digit: 1,
};
pub(crate) static mut CALIBRATION: bool = false;

// Part-2 shared state
pub(crate) static mut JOYSTICK_DIR:     joystick::JoystickDir = joystick::JoystickDir::Center;
pub(crate) static mut JOYSTICK_PRESSED: bool                  = false;
pub(crate) static mut LCD_LAST_DIR:     joystick::JoystickDir = joystick::JoystickDir::Center;
// Initialised to `true` so the first LCD tick always sees a "change" and draws the initial state.
pub(crate) static mut LCD_LAST_PRESSED: bool = true;
pub(crate) static mut LCD_LAST_HALTED:  bool = false;

pub(crate) fn add_task(task: Task) {
    avr_device::interrupt::free(|t| {
        TASKS.borrow(t).borrow_mut().push(task).ok();
    });
}

#[avr_device::interrupt(atmega328p)]
fn TIMER2_COMPA() {
    avr_device::interrupt::free(|t| {
        let mut ts = TASKS.borrow(t).borrow_mut();
        for ta in ts.iter_mut() {
            ta.elapsed_time += ELAPSED_TIME_PER_TICK;
            if ta.elapsed_time >= ta.period {
                ta.elapsed_time = 0;
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

    // ── PORTD: PD2-PD7 outputs (LCD) ──────────────────────────────────────
    p.PORTD.ddrd.modify(|r, w| unsafe { w.bits(r.bits() | 0xFC) });

    // ── PORTB ─────────────────────────────────────────────────────────────
    // Part1: PB0-PB4 out, PB5 in (clear bootloader's LED blink on PB5).
    // Part2: PB1 out (contrast PWM) + PB2-PB5 out (stepper IN4-IN1).
    #[cfg(not(feature = "part2"))]
    p.PORTB.ddrb.modify(|r, w| unsafe { w.bits((r.bits() | 0x1F) & !0x20) });
    #[cfg(feature = "part2")]
    p.PORTB.ddrb.modify(|r, w| unsafe { w.bits(r.bits() | 0x3E) }); // PB1-PB5

    // ── PORTC ─────────────────────────────────────────────────────────────
    // Part1: PC0-PC3 out, PC4/PC5 in with pull-ups.
    // Part2: PC0/PC1 analog in (VRx/VRy), PC2 digital in + pull-up (SW).
    #[cfg(not(feature = "part2"))] {
        p.PORTC.portc.modify(|r, w| unsafe { w.bits((r.bits() | 0x30) & !0x0E) });
        p.PORTC.ddrc.modify(|r, w|  unsafe { w.bits((r.bits() | 0x0F) & !0x30) });
    }
    #[cfg(feature = "part2")] {
        // PC0/PC1: no pull-up (ADC pins), PC2: pull-up (SW active-LOW)
        p.PORTC.portc.modify(|r, w| unsafe { w.bits((r.bits() & !0x03) | 0x04) });
        p.PORTC.ddrc.modify(|r, w|  unsafe { w.bits(r.bits() & !0x07) }); // PC0-PC2 inputs
    }

    lcd::init();
    lcd::init_contrast(30);

    // ── Part-1 loop (rotating phrases) ────────────────────────────────────
    #[cfg(not(feature = "part2"))] {
        let mut phase: u8 = 0;
        loop {
            lcd::clear();
            match phase {
                0 => { lcd::goto_xy(0, 0);  lcd::write_str("Hello World!"); }
                1 => { lcd::goto_xy(1, 4);  lcd::write_str("Rust is fun!"); }
                2 => { lcd::goto_xy(1, 0);  lcd::write_str("AVR rocks!"); }
                _ => { lcd::goto_xy(0, 2);  lcd::write_str("I Love CS/120B"); }
            }
            phase = (phase + 1) % 4;
            lcd::delay_ms(1000);
        }
    }

    // ── Part-2 init ────────────────────────────────────────────────────────
    #[cfg(feature = "part2")] {
        // ADC: AVCC reference, prescaler /128 (125 kHz at 16 MHz), enabled
        p.ADC.admux.write( |w| unsafe { w.bits(0x40) }); // AVCC ref, ch0
        p.ADC.adcsra.write(|w| unsafe { w.bits(0x87) }); // ADEN, ADPS=111
        // Disable digital input buffers on A0/A1 to avoid noise on ADC pins
        p.ADC.didr0.write( |w| unsafe { w.bits(0x03) });

        // Timer2: 1 ms CTC interrupt (same config as lab5)
        p.TC2.tccr2a.write(|w| w.wgm2().ctc());
        p.TC2.ocr2a.write( |w| unsafe { w.bits(249) });
        p.TC2.tccr2b.write(|w| w.cs2().prescale_64());
        p.TC2.timsk2.write(|w| w.ocie2a().set_bit());

        unsafe { avr_device::interrupt::enable() };
        task_initer::init_tasks_p2();
    }

    loop {}
}
