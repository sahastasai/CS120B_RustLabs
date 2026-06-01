use crate::task::{Task, State,
                   JoystickTaskState, StepperTaskState,
                   StepperLedState, ServoLedState, ButtonSpeedState, IdleBuzzerState};
use crate::add_task;

pub fn init_tasks() {
    use crate::joystick::{read_vry, read_vrx, dir_from_vry, JoystickDir};
    use crate::dp;

    fn joystick_tick(a: State) -> State {
        match a {
            State::Joystick(JoystickTaskState::Start) =>
                State::Joystick(JoystickTaskState::Active),
            State::Joystick(JoystickTaskState::Active) => {
                unsafe {
                    crate::JOYSTICK_DIR = dir_from_vry(read_vry());
                    crate::SERVO_ADC    = read_vrx();
                }
                State::Joystick(JoystickTaskState::Active)
            }
            _ => unreachable!(),
        }
    }
    add_task(Task::new(10, State::Joystick(JoystickTaskState::Start), joystick_tick));

    fn stepper_tick(a: State) -> State {
        static mut ACC: u32 = 0;
        match a {
            State::StepperCtrl(StepperTaskState::Start) =>
                State::StepperCtrl(StepperTaskState::Active),
            State::StepperCtrl(StepperTaskState::Active) => {
                unsafe {
                    match crate::JOYSTICK_DIR {
                        JoystickDir::Up | JoystickDir::Down => {
                            crate::STEPPER_IDLE_MS = 0;
                            ACC += 4_000 / crate::stepper::STEP_PERIOD;
                            while ACC >= 1_000 {
                                ACC -= 1_000;
                                match crate::JOYSTICK_DIR {
                                    JoystickDir::Up   => crate::stepper::step_cw(),
                                    _                 => crate::stepper::step_ccw(),
                                }
                            }
                        }
                        _ => {
                            ACC = 0;
                            crate::stepper::stop();
                            if crate::STEPPER_IDLE_MS < u32::MAX {
                                crate::STEPPER_IDLE_MS += 1;
                            }
                        }
                    }
                }
                State::StepperCtrl(StepperTaskState::Active)
            }
            _ => unreachable!(),
        }
    }
    add_task(Task::new(1, State::StepperCtrl(StepperTaskState::Start), stepper_tick));

    // Stepper LEDs: pin 5, pin 7, pin 8
    fn stepper_led_tick(a: State) -> State {
        static mut PHASE: u8 = 0;
        match a {
            State::StepperLed(StepperLedState::Start) =>
                State::StepperLed(StepperLedState::Active),
            State::StepperLed(StepperLedState::Active) => {
                unsafe {
                    PHASE = (PHASE + 1) % 3;
                    let (pb, pd): (u8, u8) = match crate::JOYSTICK_DIR {
                        JoystickDir::Down => match PHASE {
                            0 => (0x00, 0x20),
                            1 => (0x00, 0xA0),
                            _ => (0x01, 0xA0),
                        },
                        JoystickDir::Up => match PHASE {
                            0 => (0x01, 0x00),
                            1 => (0x01, 0x80),
                            _ => (0x01, 0xA0),
                        },
                        _ => match PHASE {
                            0 => (0x00, 0x80),
                            1 => (0x01, 0x20),
                            _ => (0x00, 0x00),
                        },
                    };
                    dp().PORTB.portb.modify(|r, w| w.bits((r.bits() & !0x01) | pb));
                    dp().PORTD.portd.modify(|r, w| w.bits((r.bits() & !0xA0) | pd));
                }
                State::StepperLed(StepperLedState::Active)
            }
            _ => unreachable!(),
        }
    }
    add_task(Task::new(200, State::StepperLed(StepperLedState::Start), stepper_led_tick));

    // Servo + Servo LEDs — held position while stick is in centre dead-zone.
    fn servo_led_tick(a: State) -> State {
        static mut LAST_X: u16 = 512;
        match a {
            State::ServoLed(ServoLedState::Start) =>
                State::ServoLed(ServoLedState::Active),
            State::ServoLed(ServoLedState::Active) => {
                unsafe {
                    let adc = crate::SERVO_ADC;
                    let x = if adc < 400 || adc > 624 { LAST_X = adc; adc } else { LAST_X };
                    crate::servo::set_from_adc(x);
                    let led = if x >= 768 { 0x10u8 }   // PD4
                              else if x >= 256 { 0x08 } // PD3
                              else { 0x04 };            // PD2
                    dp().PORTD.portd.modify(|r, w| w.bits((r.bits() & !0x1C) | led));
                }
                State::ServoLed(ServoLedState::Active)
            }
            _ => unreachable!(),
        }
    }
    add_task(Task::new(50, State::ServoLed(ServoLedState::Start), servo_led_tick));

    #[cfg(feature = "part2")]
    {
        fn button_tick(a: State) -> State {
            static mut PREV_DEC: bool = false;
            static mut PREV_INC: bool = false;
            match a {
                State::ButtonSpeed(ButtonSpeedState::Start) =>
                    State::ButtonSpeed(ButtonSpeedState::Active),
                State::ButtonSpeed(ButtonSpeedState::Active) => {
                    unsafe {
                        let pinc = dp().PORTC.pinc.read().bits();
                        let dec = pinc & 0x01 == 0; // A0 active-LOW
                        let inc = pinc & 0x02 == 0; // A1 active-LOW
                        if inc && !PREV_INC
                           && crate::stepper::STEP_PERIOD > crate::stepper::STEP_PERIOD_MIN
                        {
                            crate::stepper::STEP_PERIOD -= crate::stepper::STEP_PERIOD_DELTA;
                            crate::logger::log_step_period(crate::stepper::STEP_PERIOD);
                            #[cfg(feature = "part3")]
                            crate::buzzer::play_increase();
                        }
                        if dec && !PREV_DEC
                           && crate::stepper::STEP_PERIOD < crate::stepper::STEP_PERIOD_MAX
                        {
                            crate::stepper::STEP_PERIOD += crate::stepper::STEP_PERIOD_DELTA;
                            crate::logger::log_step_period(crate::stepper::STEP_PERIOD);
                            #[cfg(feature = "part3")]
                            crate::buzzer::play_decrease();
                        }
                        PREV_DEC = dec;
                        PREV_INC = inc;
                    }
                    State::ButtonSpeed(ButtonSpeedState::Active)
                }
                _ => unreachable!(),
            }
        }
        add_task(Task::new(50, State::ButtonSpeed(ButtonSpeedState::Start), button_tick));
    }

    #[cfg(feature = "part3")]
    {
        fn buzzer_tick(a: State) -> State {
            static mut NEXT_MS:   u32 = 10_000;
            static mut LOG_TICKS: u32 = 0;
            match a {
                State::IdleBuzzer(IdleBuzzerState::Start) =>
                    State::IdleBuzzer(IdleBuzzerState::Active),
                State::IdleBuzzer(IdleBuzzerState::Active) => {
                    unsafe {
                        crate::buzzer::step();

                        let idle = crate::STEPPER_IDLE_MS;
                        if idle == 0 {
                            NEXT_MS = 10_000;
                        } else if idle >= NEXT_MS && !crate::buzzer::is_playing() {
                            crate::buzzer::play_idle();
                            crate::logger::log_idle_alert(idle);
                            NEXT_MS += 5_000;
                        }

                        // 50 ms × 20 = 1 s
                        LOG_TICKS = LOG_TICKS.wrapping_add(1);
                        if idle > 0 && LOG_TICKS % 20 == 0 {
                            crate::logger::log_idle(idle);
                        }
                    }
                    State::IdleBuzzer(IdleBuzzerState::Active)
                }
                _ => unreachable!(),
            }
        }
        add_task(Task::new(50, State::IdleBuzzer(IdleBuzzerState::Start), buzzer_tick));
    }
}
