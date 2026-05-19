use crate::dp;

const CYCLE_GRANULARITY: u32 = 10;

#[derive(Copy, Clone, PartialEq)]
enum Color { Red, Blue, Green, Blank }

impl Color {
    fn next(self) -> Self {
        match self {
            Color::Red   => Color::Blue,
            Color::Blue  => Color::Green,
            Color::Green => Color::Blank,
            Color::Blank => Color::Red,
        }
    }
}

pub struct NonSimultaneousRGBDisplay {
    pub red_cycle: u32,
    pub blue_cycle: u32,
    pub green_cycle: u32,
    pub current_color: Color,
    pub internal_counter: u32,
}

impl NonSimultaneousRGBDisplay {
    pub fn purple() -> Self {
        Self {
            red_cycle: CYCLE_GRANULARITY / 2,
            blue_cycle: CYCLE_GRANULARITY / 2,
            green_cycle: 0,
            current_color: Color::Red,
            internal_counter: 0,
        }
    }

    pub fn display_color(&mut self) {
        if self.internal_counter >= self.red_cycle && self.current_color == Color::Red {
            self.current_color = self.current_color.next();
            self.internal_counter = 0;
        }
        if self.internal_counter >= self.blue_cycle && self.current_color == Color::Blue {
            self.current_color = self.current_color.next();
            self.internal_counter = 0;
        }
        if self.internal_counter >= self.green_cycle && self.current_color == Color::Green {
            self.current_color = self.current_color.next();
            self.internal_counter = 0;
        }
        if self.internal_counter >= CYCLE_GRANULARITY - (self.red_cycle + self.blue_cycle + self.green_cycle)
            && self.current_color == Color::Blank
        {
            self.current_color = self.current_color.next();
            self.internal_counter = 0;
        }
        match self.current_color {
            Color::Red   => dp().PORTC.portc.modify(|r, w| unsafe { w.bits((r.bits() & !0x0E) | 0x02) }),
            Color::Blue  => dp().PORTC.portc.modify(|r, w| unsafe { w.bits((r.bits() & !0x0E) | 0x08) }),
            Color::Green => dp().PORTC.portc.modify(|r, w| unsafe { w.bits((r.bits() & !0x0E) | 0x04) }),
            Color::Blank => dp().PORTC.portc.modify(|r, w| unsafe { w.bits(r.bits() & !0x0E) }),
        }
        self.internal_counter += 1;
    }
}
pub struct RGBDisplay {
    pub on_mask: u8,
    pub internal_counter: u32,
    pub total_count: u32,
}

impl RGBDisplay {
    pub fn purple() -> Self {
        Self { on_mask: 0x0A, internal_counter: 0, total_count: CYCLE_GRANULARITY / 2 }
    }

    pub fn adjust_brightness(&mut self, brightness: u32) {
        self.total_count = brightness % (CYCLE_GRANULARITY + 1);
    }

    pub fn turn_red(&mut self)    { self.on_mask = 0x02; } 
    pub fn turn_blue(&mut self)   { self.on_mask = 0x08; }
    pub fn turn_green(&mut self)  { self.on_mask = 0x04; }
    pub fn turn_purple(&mut self) { self.on_mask = 0x0A; }
    pub fn add_blue(&mut self)    { self.on_mask |= 0x08; } 

    pub fn display_color(&mut self) {
        self.internal_counter %= CYCLE_GRANULARITY;
        let pc0 = dp().PORTC.portc.read().bits() & 0x01;
        let rgb = if self.internal_counter < self.total_count {
            self.on_mask & 0x0E  
        } else {
            0x00                 
        };
        dp().PORTC.portc.write(|w| unsafe { w.bits(pc0 | 0x30 | rgb) });
        self.internal_counter += 1;
    }
}
