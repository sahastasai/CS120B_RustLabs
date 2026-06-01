use crate::dp;

const CS_HIGH: u8 = 0x02;
const CS_MID:  u8 = 0x03;
const CS_LOW:  u8 = 0x04;
const CS_OFF:  u8 = 0x00;
const MAX_NOTES: usize = 3;

type Note = (u8, u8);

static mut NOTES:    [Note; MAX_NOTES] = [(0, 0); MAX_NOTES];
static mut NOTE_CNT: u8 = 0;
static mut NOTE_IDX: u8 = 0;
static mut NOTE_CTR: u8 = 0;

pub fn init() {
    let p = dp();
    p.TC0.tccr0a.write(|w| unsafe { w.bits((1 << 7) | (1 << 1) | (1 << 0)) }); // mode 3 + COM0A1
    p.TC0.tccr0b.write(|w| unsafe { w.bits(0x00) });
    p.TC0.ocr0a.write(|w| unsafe { w.bits(128) });
}

fn set_tone(cs: u8) {
    let p = dp();
    if cs == CS_OFF {
        p.TC0.tccr0b.write(|w| unsafe { w.bits(0x00) });
        p.PORTD.portd.modify(|r, w| unsafe { w.bits(r.bits() & !0x40) });
    } else {
        p.TC0.tccr0b.write(|w| unsafe { w.bits(cs & 0x07) });
    }
}

pub fn is_playing() -> bool {
    unsafe { NOTE_IDX < NOTE_CNT }
}

fn play(seq: &[Note]) {
    unsafe {
        let n = seq.len().min(MAX_NOTES);
        NOTE_CNT = n as u8;
        NOTE_IDX = 0;
        NOTE_CTR = 0;
        for i in 0..n { NOTES[i] = seq[i]; }
        set_tone(NOTES[0].0);
    }
}

// Two notes ascending -- speed up.
pub fn play_increase() {
    play(&[(CS_LOW, 3), (CS_MID, 3)]);
}

// Two notes descending -- speed down.
pub fn play_decrease() {
    play(&[(CS_MID, 3), (CS_LOW, 3)]);
}

// Three notes mid-low-mid -- idle alert.
pub fn play_idle() {
    play(&[(CS_MID, 4), (CS_LOW, 4), (CS_MID, 4)]);
}

// Call every 50 ms
pub fn step() {
    unsafe {
        if NOTE_IDX >= NOTE_CNT { return; }
        NOTE_CTR = NOTE_CTR.saturating_add(1);
        if NOTE_CTR >= NOTES[NOTE_IDX as usize].1 {
            NOTE_CTR = 0;
            NOTE_IDX += 1;
            if NOTE_IDX >= NOTE_CNT {
                set_tone(CS_OFF);
            } else {
                set_tone(NOTES[NOTE_IDX as usize].0);
            }
        }
    }
}
