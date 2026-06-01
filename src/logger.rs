use crate::dp;

// 16 MHz / (16 * 9600) - 1 = 103
const UBRR_9600: u16 = 103;

pub fn init_uart() {
    let p = dp();
    p.USART0.ubrr0.write(|w| unsafe { w.bits(UBRR_9600) });
    p.USART0.ucsr0b.write(|w| w.txen0().set_bit());
    send_byte(b'!'); // startup probe — if this appears, UART works
    send_byte(b'\n');
}

fn send_byte(byte: u8) {
    while dp().USART0.ucsr0a.read().udre0().bit_is_clear() {}
    dp().USART0.udr0.write(|w| unsafe { w.bits(byte) });
}

fn send_bytes(s: &[u8]) {
    for &b in s { send_byte(b); }
}

fn write_u32(mut n: u32) {
    if n == 0 {
        send_byte(b'0');
        return;
    }
    let mut buf = [0u8; 10];
    let mut len = 0usize;
    while n > 0 {
        buf[len] = b'0' + (n % 10) as u8;
        n /= 10;
        len += 1;
    }
    let mut i = len;
    while i > 0 {
        i -= 1;
        send_byte(buf[i]);
    }
}

pub fn log_step_period(period_ms: u32) {
    send_bytes(b"step_period=");
    write_u32(period_ms);
    send_bytes(b" ms/phase\n");
}

pub fn log_idle(ms: u32) {
    send_bytes(b"idle=");
    write_u32(ms);
    send_bytes(b" ms\n");
}

pub fn log_idle_alert(ms: u32) {
    send_bytes(b"idle=");
    write_u32(ms);
    send_bytes(b" ms ALERT\n");
}

pub fn log_led_selection(color: &[u8], brightness: u32, dist_cm: u32) {
    send_bytes(b"LED:");
    send_bytes(color);
    send_bytes(b" b=");
    write_u32(brightness);
    send_bytes(b" d=");
    write_u32(dist_cm);
    send_byte(b'\n');
}

pub fn log_ranges(close: u32, med: u32) {
    send_bytes(b"CLOSE=");
    write_u32(close);
    send_bytes(b" MED=");
    write_u32(med);
    send_byte(b'\n');
}

pub fn log_distance(dist: f64, close: u32, med: u32) {
    let cm = dist as u32;
    match cm {
        997 => send_bytes(b"ERR:no_echo\n"),
        998 => send_bytes(b"ERR:stuck_hi\n"),
        _ => {
            write_u32(cm);
            send_byte(b' ');
            if dist < close as f64 {
                send_byte(b'C');
            } else if dist <= med as f64 {
                send_byte(b'M');
            } else {
                send_byte(b'H');
            }
            send_byte(b'\n');
        }
    }
}

