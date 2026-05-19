use avr_device::asm::delay_cycles;
use avr_device::atmega328p::Peripherals;

const CYCLES_IN_US: u32 = 16;
const WAITING_TIME_IN_US: u32 = 10;
static mut TIMER_OVERFLOW: usize = 0;
const SOUND_SPEED: f64 = 932.46;
const TIMER_OVERFLOW_MULTIPLIER: f64 = 65535.0;

// TRIG = PB4 (pin 12), ECHO = PB5 (pin 13)
const TRIG_MASK: u8 = 0x10;
const ECHO_MASK: u8 = 0x20;

// TC1 overflows every 65536/16MHz ≈ 4ms; 5 overflows ≈ 20ms covers HC-SR04 max range.
const TIMEOUT_OVERFLOWS: u8 = 5;

fn wait_echo_high(dp: &Peripherals) -> bool {
    let mut ovf: u8 = 0;
    loop {
        if dp.PORTB.pinb.read().bits() & ECHO_MASK != 0 {
            return true;
        }
        if dp.TC1.tifr1.read().tov1().bit_is_set() {
            dp.TC1.tifr1.write(|w| w.tov1().set_bit());
            ovf += 1;
            if ovf >= TIMEOUT_OVERFLOWS {
                return false;
            }
        }
    }
}

fn wait_echo_low(dp: &Peripherals) -> bool {
    let mut ovf: u8 = 0;
    loop {
        if dp.PORTB.pinb.read().bits() & ECHO_MASK == 0 {
            return true;
        }
        if dp.TC1.tifr1.read().tov1().bit_is_set() {
            dp.TC1.tifr1.write(|w| w.tov1().set_bit());
            unsafe { TIMER_OVERFLOW += 1; }
            ovf += 1;
            if ovf >= TIMEOUT_OVERFLOWS {
                return false;
            }
        }
    }
}

pub fn sonar_read(dp: &Peripherals) -> f64 {
    // Trigger: pulse PB4 high for 10 µs
    dp.PORTB.portb.modify(|r, w| unsafe { w.bits(r.bits() | TRIG_MASK) });
    delay_cycles(CYCLES_IN_US * WAITING_TIME_IN_US);
    dp.PORTB.portb.modify(|r, w| unsafe { w.bits(r.bits() & !TRIG_MASK) });

    // Start TC1 free-running at 16 MHz
    dp.TC1.tccr1b.write(|w| w.cs1().direct());
    dp.TC1.tcnt1.write(|w| unsafe { w.bits(0u16) });
    dp.TC1.tifr1.write(|w| w.tov1().set_bit());

    if !wait_echo_high(dp) {
        return 997.0; // sentinel: echo never went HIGH (trigger not reaching sensor, or wiring)
    }

    // Reset counter at rising edge for accurate pulse-width timing
    dp.TC1.tcnt1.write(|w| unsafe { w.bits(0u16) });
    dp.TC1.tifr1.write(|w| w.tov1().set_bit());
    unsafe { TIMER_OVERFLOW = 0; }

    if !wait_echo_low(dp) {
        return 998.0; // sentinel: echo went HIGH but never LOW (object beyond range or stuck)
    }

    let ticks = dp.TC1.tcnt1.read().bits() as f64;
    let overflow = unsafe { TIMER_OVERFLOW };
    (ticks + (TIMER_OVERFLOW_MULTIPLIER * overflow as f64)) / SOUND_SPEED
}
