use crate::dp;

const SERVO_CENTER: u16 = 2999;
const SERVO_LEFT:   u16 = 2000;
const SERVO_RIGHT:  u16 = 2000;

pub fn init() {
    let p = dp();
    p.TC1.tccr1a.write(|w| unsafe { w.bits((1 << 7) | (1 << 1)) });           // COM1A1 | WGM11
    p.TC1.tccr1b.write(|w| unsafe { w.bits((1 << 4) | (1 << 3) | (1 << 1)) }); // WGM13 | WGM12 | CS11
    p.TC1.icr1.write( |w| unsafe { w.bits(39999u16) });
    p.TC1.ocr1a.write(|w| unsafe { w.bits(SERVO_CENTER) });
}

pub fn set_from_adc(adc: u16) -> u16 {
    let ocr = if adc <= 512 {
        let d = ((512 - adc) as u32 * SERVO_LEFT as u32 / 512) as u16;
        SERVO_CENTER.saturating_sub(d)
    } else {
        let d = ((adc - 512) as u32 * SERVO_RIGHT as u32 / 511) as u16;
        SERVO_CENTER.saturating_add(d).min(39999)
    };
    dp().TC1.ocr1a.write(|w| unsafe { w.bits(ocr) });
    ocr
}
