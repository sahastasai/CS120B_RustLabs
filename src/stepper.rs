use crate::dp;

const STEPS: [u8; 8] = [
    0x20, // 1000 — IN1
    0x30, // 1100 — IN1+IN2
    0x10, // 0100 — IN2
    0x18, // 0110 — IN2+IN3
    0x08, // 0010 — IN3
    0x0C, // 0011 — IN3+IN4
    0x04, // 0001 — IN4
    0x24, // 1001 — IN4+IN1
];

const STEPPER_MASK: u8 = 0x3C;

static mut STEP_IDX: u8 = 0;

// Period label in C-reference ms/phase; task scales 8x via accumulator.
pub(crate) const STEP_PERIOD_MIN: u32 = 5;
pub(crate) const STEP_PERIOD_MAX: u32 = 30;
pub(crate) const STEP_PERIOD_DELTA: u32 = 5;
pub(crate) static mut STEP_PERIOD: u32 = 15;

fn apply_step() {
    let pattern = unsafe { STEPS[STEP_IDX as usize] };
    dp().PORTB.portb.modify(|r, w| unsafe {
        w.bits((r.bits() & !STEPPER_MASK) | pattern)
    });
}

pub fn step_ccw() {
    unsafe { STEP_IDX = (STEP_IDX + 1) % 8; }
    apply_step();
}

pub fn step_cw() {
    unsafe { STEP_IDX = (STEP_IDX + 7) % 8; } // +7 mod 8 == -1 mod 8
    apply_step();
}

pub fn stop() {
    dp().PORTB.portb.modify(|r, w| unsafe {
        w.bits(r.bits() & !STEPPER_MASK)
    });
}
