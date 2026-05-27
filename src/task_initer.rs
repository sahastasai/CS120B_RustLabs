use crate::task::{Task, State, SonarState, DisplayState, LEDState, ButtonState,
                   LEDSelectorState, LEDDriverState, CalibratorState,
                   JoystickTaskState, LCDTaskState, StepperTaskState};
use crate::calibrator::Calibrator;
use crate::rgb::RGBDisplay;
use crate::add_task;
use crate::sonar::sonar_read;
use crate::button::ButtonPress;

// L6 P2 tasks
pub fn init_tasks_p2() {
    use crate::joystick::{read_dir, read_pressed, JoystickDir};

    fn joystick_tick(a: State) -> State {
        match a {
            State::Joystick(JoystickTaskState::Start) =>
                State::Joystick(JoystickTaskState::Active),
            State::Joystick(JoystickTaskState::Active) => {
                unsafe {
                    crate::JOYSTICK_DIR     = read_dir();
                    crate::JOYSTICK_PRESSED = read_pressed();
                }
                State::Joystick(JoystickTaskState::Active)
            }
            _ => unreachable!(),
        }
    }
    add_task(Task::new(100, State::Joystick(JoystickTaskState::Start), joystick_tick));

    
    fn lcd_tick(a: State) -> State {
        match a {
            State::LCDDisplay(LCDTaskState::Start) =>
                State::LCDDisplay(LCDTaskState::Active),
            State::LCDDisplay(LCDTaskState::Active) => {
                unsafe {
                    let dir     = crate::JOYSTICK_DIR;
                    let pressed = crate::JOYSTICK_PRESSED;

                    #[cfg(feature = "part3")]
                    let halted = crate::stepper::is_halted();

                    let mut changed = dir != crate::LCD_LAST_DIR
                                   || pressed != crate::LCD_LAST_PRESSED;
                    #[cfg(feature = "part3")]
                    { changed = changed || halted != crate::LCD_LAST_HALTED; }

                    if changed {
                        crate::LCD_LAST_DIR     = dir;
                        crate::LCD_LAST_PRESSED = pressed;
                        #[cfg(feature = "part3")]
                        { crate::LCD_LAST_HALTED = halted; }

                        // Top row — always overwrite all 16 cols (no clear needed)
                        crate::lcd::goto_xy(0, 0);
                        match dir {
                            JoystickDir::Left  => crate::lcd::write_str("<-Left          "),
                            JoystickDir::Right => crate::lcd::write_str("         Right->"),
                            _                  => crate::lcd::write_str("      Idle      "),
                        }

                        // Bottom row
                        crate::lcd::goto_xy(1, 0);
                        #[cfg(not(feature = "part3"))]
                        {
                            if pressed {
                                crate::lcd::write_str("    Pressed!    ");
                            } else {
                                crate::lcd::write_str("                ");
                            }
                        }
                        #[cfg(feature = "part3")]
                        {
                            if pressed {
                                crate::lcd::write_str("     Reset      ");
                            } else if halted {
                                crate::lcd::write_str("    Halted!     ");
                            } else {
                                crate::lcd::write_str("                ");
                            }
                        }
                    }
                }
                State::LCDDisplay(LCDTaskState::Active)
            }
            _ => unreachable!(),
        }
    }
    add_task(Task::new(100, State::LCDDisplay(LCDTaskState::Start), lcd_tick));

    fn stepper_tick(a: State) -> State {
        match a {
            State::StepperCtrl(StepperTaskState::Start) =>
                State::StepperCtrl(StepperTaskState::Active),
            State::StepperCtrl(StepperTaskState::Active) => {
                unsafe {
                    #[cfg(feature = "part3")]
                    {
                        if crate::JOYSTICK_PRESSED {
                            crate::stepper::reset_position();
                            crate::stepper::stop();
                        } else {
                            match crate::JOYSTICK_DIR {
                                JoystickDir::Left  => crate::stepper::step_ccw(),
                                JoystickDir::Right => crate::stepper::step_cw(),
                                _                  => crate::stepper::stop(),
                            }
                        }
                    }
                    #[cfg(not(feature = "part3"))]
                    match crate::JOYSTICK_DIR {
                        JoystickDir::Left  => crate::stepper::step_ccw(),
                        JoystickDir::Right => crate::stepper::step_cw(),
                        _                  => crate::stepper::stop(),
                    }
                }
                State::StepperCtrl(StepperTaskState::Active)
            }
            _ => unreachable!(),
        }
    }
    add_task(Task::new(1, State::StepperCtrl(StepperTaskState::Start), stepper_tick));
}
