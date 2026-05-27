use avr_device::asm::delay_cycles;
use crate::dp;

const MS: u32 = 16_000;      // cycles per ms at 16 MHz
const NIBBLE_HALF: u32 = 800; // ~50 µs per EN half-pulse (HD44780 min is 450 ns)

// Pin mapping — matches LCD-1-1.h: LCD_EN=3 (PD3), LCD_RS=2 (PD2), D4-D7=PD4-PD7
const EN: u8 = 1 << 3;
const RS: u8 = 1 << 2;

#[inline(always)]
fn wr(v: u8) {
    dp().PORTD.portd.write(|w| unsafe { w.bits(v) });
}

// `nibble` carries the payload in bits 7–4. Data + RS + EN are set in one write.
fn send_nibble(nibble: u8, rs: bool) {
    let ctrl: u8 = if rs { RS } else { 0 };
    wr((nibble & 0xF0) | ctrl | EN); // EN high
    delay_cycles(NIBBLE_HALF);
    wr((nibble & 0xF0) | ctrl);      // EN low  ← HD44780 latches on falling edge
    delay_cycles(NIBBLE_HALF);
}

pub fn send_command(cmd: u8) {
    send_nibble(cmd,      false);
    send_nibble(cmd << 4, false);
}

pub fn write_char(ch: u8) {
    send_nibble(ch,      true);
    send_nibble(ch << 4, true);
}

pub fn write_str(s: &str) {
    for b in s.bytes() {
        write_char(b);
    }
}

pub fn clear() {
    send_command(0x01);
    delay_cycles(5 * MS); // clear takes up to 1.52 ms
}

pub fn goto_xy(line: u8, pos: u8) {
    send_command(0x80 | (line << 6) | pos);
}

pub fn delay_ms(ms: u32) {
    delay_cycles(MS * ms);
}

/// Full HD44780 4-bit power-on init (datasheet Fig. 24).
pub fn init() {
    delay_cycles(50 * MS); // >40 ms after VCC stable

    // Three soft-reset nibbles (0x3) to guarantee a known state regardless of
    // what the LCD thinks its bus width is after power-on.
    send_nibble(0x30, false); delay_cycles(5 * MS);
    send_nibble(0x30, false); delay_cycles(MS);
    send_nibble(0x30, false); delay_cycles(MS);

    // Switch to 4-bit interface
    send_nibble(0x20, false); delay_cycles(MS);

    send_command(0x28); delay_cycles(MS);     // function set: 4-bit, 2-line, 5×7
    send_command(0x08); delay_cycles(MS);     // display off
    send_command(0x01); delay_cycles(5 * MS); // clear display + cursor home
    send_command(0x06); delay_cycles(MS);     // entry mode: auto-increment, no shift
    send_command(0x0C); delay_cycles(MS);     // display on, no cursor
}

/// Drives pin 9 (PB1 / OC1A) with fast-PWM for LCD contrast (V0).
/// duty 0–255: lower → lower voltage → higher contrast on a positive LCD at 5 V.
pub fn init_contrast(duty: u8) {
    // Fast PWM 8-bit mode 5: TCCR1A=0x81 (COM1A=10, WGM10), TCCR1B=0x09 (WGM12, CS10=no prescale)
    dp().TC1.tccr1a.write(|w| unsafe { w.bits(0x81) });
    dp().TC1.tccr1b.write(|w| unsafe { w.bits(0x09) });
    dp().TC1.ocr1a.write(|w| unsafe { w.bits(duty as u16) });
}
