use crate::dp;

pub struct FourDigitDisplay {
    pub digit1: u8,
    pub digit2: u8,
    pub digit3: u8,
    pub digit4: u8,
    pub current_digit: u8,
}

impl FourDigitDisplay {
    pub fn init() -> Self {
        Self {
            digit1: 0,
            digit2: 0,
            digit3: 0,
            digit4: 0,
            current_digit: 1,
        }
    }

    pub fn set_digit(&mut self, digit: u8, value: u8) -> bool {
        match digit {
            1 => self.digit1 = value,
            2 => self.digit2 = value,
            3 => self.digit3 = value,
            4 => self.digit4 = value,
            _ => return false,
        }
        true
    }

    // No blocking delay: safe to call directly from the ISR.
    pub fn display_next(&mut self) {
        self.display_digit(self.current_digit);
        self.current_digit = self.current_digit % 4 + 1;
    }

    pub fn display_digit(&self, digit: u8) {
        let value = match digit {
            1 => self.digit1,
            2 => self.digit2,
            3 => self.digit3,
            4 => self.digit4,
            _ => return,
        }
        .min(18);

        // Segment bitmasks for each hex numeral plus 'r', 'U', 'L' at indices 16-18.
        const SEGMENTS: [(u8, u8); 19] = [
            (0x0F, 0xC0), // 0  A B C D E F
            (0x06, 0x00), // 1      B C
            (0x0D, 0xA0), // 2  A B   D E   G
            (0x0F, 0x20), // 3  A B C D     G
            (0x06, 0x60), // 4      B C   F G
            (0x0B, 0x60), // 5  A   C D   F G
            (0x0B, 0xE0), // 6  A   C D E F G
            (0x0E, 0x00), // 7  A B C
            (0x0F, 0xE0), // 8  A B C D E F G
            (0x0F, 0x60), // 9  A B C D   F G
            (0x0E, 0xE0), // A  A B C   E F G
            (0x03, 0xE0), // b      C D E F G
            (0x09, 0xC0), // C  A     D E F
            (0x07, 0xA0), // d    B C D E   G
            (0x09, 0xE0), // E  A     D E F G
            (0x08, 0xE0), // F  A       E F G
            (0x00, 0xA0), // r          E   G
            (0x07, 0xC0), // U    B C D E F
            (0x01, 0xC0), // L        D E F
        ];

        let (portb_seg, portd_seg) = SEGMENTS[value as usize];

        dp().PORTD.portd.modify(|r, w| unsafe { w.bits(r.bits() | 0x1C) }); // PD4:2 high
        dp().PORTC.portc.modify(|r, w| unsafe { w.bits(r.bits() | 0x01) }); // PC0 high
        dp().PORTB.portb.modify(|r, w| unsafe {
            w.bits((r.bits() & 0xF0) | portb_seg)
        });
        dp().PORTD.portd.modify(|r, w| unsafe {
            w.bits((r.bits() & 0x1F) | portd_seg)
        });

        match digit {
            1 => dp().PORTD.portd.modify(|r, w| unsafe { w.bits(r.bits() & !0x10) }), // PD4
            2 => dp().PORTD.portd.modify(|r, w| unsafe { w.bits(r.bits() & !0x08) }), // PD3
            3 => dp().PORTD.portd.modify(|r, w| unsafe { w.bits(r.bits() & !0x04) }), // PD2
            4 => dp().PORTC.portc.modify(|r, w| unsafe { w.bits(r.bits() & !0x01) }), // PC0
            _ => {}
        }
    }
}
