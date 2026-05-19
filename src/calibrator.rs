use crate::dp;

const TICK_COUNT_MAXIMUM: u32 = 10; // 10 x 200 ms = 2 s

pub struct Calibrator {
    pub tick_count: u32,
    pub released: bool, // button must be released once after entering before a press can exit
}

impl Calibrator {
    pub fn init() -> Self {
        Self { tick_count: 0, released: false }
    }

    pub fn calibrator_loop(&mut self) {
        let left = (dp().PORTC.pinc.read().bits() & 0x20) == 0;
        unsafe {
            if crate::CALIBRATION {
                // Update thresholds continuously from the live sensor reading.
                let dist = crate::DISTANCE as u32;
                crate::CLOSE_RANGE = dist;
                crate::MED_RANGE   = dist + dist / 2; 

                crate::DISPLAY.set_digit(4, 12); 
                crate::DISPLAY.set_digit(3, 10);
                crate::DISPLAY.set_digit(2, 18);
                crate::DISPLAY.set_digit(1, 11);

                if !left {
                    self.released = true;
                } else if self.released {
                    crate::CALIBRATION = false;
                    self.tick_count    = 0;
                    self.released      = false;
                }
            } else {
                if left {
                    self.tick_count += 1;
                } else {
                    self.tick_count = 0;
                }
                if self.tick_count >= TICK_COUNT_MAXIMUM {
                    crate::CALIBRATION = true;
                    self.tick_count = 0;
                    self.released = false;
                }
            }
        }
    }
}
