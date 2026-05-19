use crate::dp;
use crate::CLOSE_RANGE;
use crate::MED_RANGE;
pub struct ButtonPress {
    pub close_range: u32,
    pub med_range: u32,
    pub prev_tick: bool,
    pub prev_tick_count: u32
}

impl ButtonPress {
    pub fn init() -> Self {
        Self {
            close_range: unsafe {CLOSE_RANGE},
            med_range: unsafe {MED_RANGE},
            prev_tick: false,
            prev_tick_count: 0
        }
    }
    pub fn detect_and_handle_press(&mut self) -> u8 {
        let increase = (dp().PORTC.pinc.read().bits() & 0x10) == 0;
        let decrease = (dp().PORTC.pinc.read().bits() & 0x20) == 0;

        if(increase && !self.prev_tick) {
            self.close_range += 1;
            self.med_range += 1;
            self.prev_tick = true;
            return 2;
        }
        if(decrease && !self.prev_tick) {
            self.close_range -= 1;
            self.med_range -= 1;
            self.prev_tick = true;
            return 1;
        }
        self.prev_tick = false;
        0
    }
}
