use crate::dp;

#[derive(Clone, Copy, PartialEq)]
pub enum JoystickDir {
    Left,
    Right,
    Up,
    Down,
    Center,
}

// Single blocking ADC conversion. ADCSRA must already have ADEN set.
fn adc_read(channel: u8) -> u16 {
    let dp = dp();
    // Select channel, AVCC reference (0x40 | ch)
    dp.ADC.admux.write(|w| unsafe { w.bits(0x40 | (channel & 0x07)) });
    // Start conversion: ADEN=1, ADSC=1, ADPS=111 (prescaler 128 → 125 kHz at 16 MHz)
    dp.ADC.adcsra.write(|w| unsafe { w.bits(0xC7) });
    // Wait for ADSC to clear (conversion complete)
    while dp.ADC.adcsra.read().bits() & 0x40 != 0 {}
    // Read 10-bit result (ADCL must be read first — avr-device combined read handles this)
    dp.ADC.adc.read().bits()
}

/// Joystick is mounted sideways: VRy (A1) = left/right, VRx (A0) = up/down.
pub fn read_dir() -> JoystickDir {
    let vry = adc_read(1); // A1 = left/right axis
    if vry < 300 {
        return JoystickDir::Left;
    } else if vry > 700 {
        return JoystickDir::Right;
    }
    let vrx = adc_read(0); // A0 = up/down axis
    if vrx < 300 {
        JoystickDir::Up
    } else if vrx > 700 {
        JoystickDir::Down
    } else {
        JoystickDir::Center
    }
}

/// SW on A2 (PC2), active LOW with pull-up.
pub fn read_pressed() -> bool {
    dp().PORTC.pinc.read().bits() & 0x04 == 0
}
