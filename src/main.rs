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
pub mod servo;
#[cfg(feature = "part3")] pub mod buzzer;

use task::Task;

const ELAPSED_TIME_PER_TICK: u32 = 1;

static TASKS: Mutex<RefCell<Vec<Task, 8>>> = Mutex::new(RefCell::new(Vec::new()));

static mut DP: Option<Peripherals> = None;

pub(crate) fn dp() -> &'static Peripherals {
    unsafe { DP.as_ref().unwrap_unchecked() }
}

// ── Legacy globals (kept so old modules compile) ──────────────────────────
pub(crate) static mut DISTANCE:    f64 = 0.0;
pub(crate) static mut CLOSE_RANGE: u32 = 10;
pub(crate) static mut MED_RANGE:   u32 = 15;
pub(crate) static mut LED_DISPLAY: rgb::RGBDisplay = rgb::RGBDisplay {
    on_mask: 0x00, internal_counter: 0, total_count: 0,
};
pub(crate) static mut DISPLAY: display::FourDigitDisplay = display::FourDigitDisplay {
    digit1: 0, digit2: 0, digit3: 0, digit4: 0, current_digit: 1,
};
pub(crate) static mut CALIBRATION:    bool = false;
pub(crate) static mut JOYSTICK_DIR:     joystick::JoystickDir = joystick::JoystickDir::Center;
pub(crate) static mut JOYSTICK_PRESSED: bool                  = false;
pub(crate) static mut LCD_LAST_DIR:     joystick::JoystickDir = joystick::JoystickDir::Center;
pub(crate) static mut LCD_LAST_PRESSED: bool = true;
pub(crate) static mut LCD_LAST_HALTED:  bool = false;

// Lab-7 shared state.
pub(crate) static mut SERVO_ADC:       u16 = 512;
pub(crate) static mut STEPPER_IDLE_MS: u32 = 0;

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

    // ── PORTD: PD2-PD7 outputs ────────────────────────────────────────────
    // PD2-4: LED Servo 1-3  |  PD5: LED Stepper 1  |  PD6: Buzzer (OC0A)  |  PD7: LED Stepper 3
    p.PORTD.ddrd.modify(|r, w| unsafe { w.bits(r.bits() | 0xFC) });

    // ── PORTB: PB0-PB5 outputs ────────────────────────────────────────────
    // PB0: LED Stepper 2  |  PB1: Servo PWM (OC1A)  |  PB2-PB5: Stepper IN4-IN1
    p.PORTB.ddrb.modify(|r, w| unsafe { w.bits(r.bits() | 0x3F) });

    // ── PORTC: all inputs ─────────────────────────────────────────────────
    // PC0/PC1: buttons (pull-up, active-LOW)  |  PC2/PC3: VRx/VRy ADC  |  PC4: SW (pull-up)
    p.PORTC.portc.modify(|r, w| unsafe { w.bits((r.bits() & !0x0C) | 0x13) });
    p.PORTC.ddrc.modify( |r, w| unsafe { w.bits(r.bits() & !0x1F) });

    // ADC: AVCC ref, prescaler /128. Disable digital input buffers on A2/A3 (VRx/VRy).
    p.ADC.admux.write( |w| unsafe { w.bits(0x40) });
    p.ADC.adcsra.write(|w| unsafe { w.bits(0x87) });
    p.ADC.didr0.write( |w| unsafe { w.bits(0x0C) });

    // Servo: Timer1 Fast PWM on OC1A (PB1), ICR1=39999 → 50 Hz.
    servo::init();

    // Buzzer: Timer0 Fast PWM on OC0A (PD6), prescaler chosen per note.
    #[cfg(feature = "part3")]
    buzzer::init();

    // UART @ 9600 baud on PD1 (TX) — for debug logging via USB-serial bridge.
    logger::init_uart();

    // Timer2: 1 ms CTC task-scheduler tick.
    p.TC2.tccr2a.write(|w| w.wgm2().ctc());
    p.TC2.ocr2a.write( |w| unsafe { w.bits(249) });
    p.TC2.tccr2b.write(|w| w.cs2().prescale_64());
    p.TC2.timsk2.write(|w| w.ocie2a().set_bit());

    unsafe { avr_device::interrupt::enable() };
    task_initer::init_tasks();

    loop {}
}
