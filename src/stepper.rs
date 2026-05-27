use crate::dp;

// These are the steps denoted with what pin(s) on the stepper they activate.
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

// Mask covering the four stepper pins
const STEPPER_MASK: u8 = 0x3C;

const MAX_STEPS: i32 = 1024; // In Rust, with my step size, the amount of half steps ended up being
                             // 8 times higher than the script provided!

static mut STEP_IDX: u8 = 0;
pub(crate) static mut STEPPER_POS: i32 = 0;

fn apply_step() {
    let pattern = unsafe { STEPS[STEP_IDX as usize] };
    dp().PORTB.portb.modify(|r, w| unsafe {
        w.bits((r.bits() & !STEPPER_MASK) | pattern)
    });
}

pub fn step_ccw() {
    unsafe {
        #[cfg(feature = "part3")]
        if STEPPER_POS >= MAX_STEPS { return; }
        STEP_IDX = (STEP_IDX + 1) % 8;
        #[cfg(feature = "part3")]
        { STEPPER_POS += 1; }
    }
    apply_step();
}

pub fn step_cw() {
    unsafe {
        #[cfg(feature = "part3")]
        if STEPPER_POS <= -MAX_STEPS { return; }
        STEP_IDX = (STEP_IDX + 7) % 8; // +7 mod 8 == -1 mod 8
        #[cfg(feature = "part3")]
        { STEPPER_POS -= 1; }
    }
    apply_step();
}

/// Reset position to zero 
pub fn reset_position() {
    unsafe { STEPPER_POS = 0; }
}

pub fn is_halted() -> bool {
    unsafe { STEPPER_POS >= MAX_STEPS || STEPPER_POS <= -MAX_STEPS }
}

pub fn stop() {
    dp().PORTB.portb.modify(|r, w| unsafe {
        w.bits(r.bits() & !STEPPER_MASK)
    });
}
