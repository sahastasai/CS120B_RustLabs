use crate::task::{Task, State, SonarState, DisplayState, LEDState, ButtonState,
                   LEDSelectorState, LEDDriverState, CalibratorState};
use crate::calibrator::Calibrator;
use crate::rgb::RGBDisplay;
use crate::add_task;
use crate::sonar::sonar_read;
use crate::button::ButtonPress;

pub fn init_tasks() {
    if cfg!(feature = "part1") {
    // Sonar: reads distance every 1000 ms
    let sonar_start = State::Sonar(SonarState::Start);
    fn sonar_tick(a: State) -> State {
        match a {
            State::Sonar(b) => match b {
                SonarState::Start   => {}
                SonarState::Measure => unsafe {
                    crate::DISTANCE = sonar_read(crate::dp());
                    crate::logger::log_ranges(crate::CLOSE_RANGE, crate::MED_RANGE);
                    crate::logger::log_distance(crate::DISTANCE, crate::CLOSE_RANGE, crate::MED_RANGE);
                }
            },
            _ => unreachable!(),
        }
        State::Sonar(SonarState::Measure)
    }
    add_task(Task::new(1000, sonar_start, sonar_tick));

    // Display: updates every 1 ms
    let display_start = State::Display(DisplayState::Start);
    fn display_tick(a: State) -> State {
        match a {
            State::Display(b) => match b {
                DisplayState::Start => State::Display(DisplayState::Active),
                DisplayState::Active => unsafe {
                    let dist = crate::DISTANCE;
                    let d = &mut crate::DISPLAY;
                    if !crate::CALIBRATION {
                        if dist < 8.0 {
                            d.set_digit(4, 12);
                            d.set_digit(3, 16);
                            d.set_digit(2, 10);
                            d.set_digit(1, 11);
                        } else if dist > 25.0 {
                            d.set_digit(4, 15);
                            d.set_digit(3, 17);
                            d.set_digit(2, 18);
                            d.set_digit(1, 18);
                        } else {
                            d.set_digit(1, 0);
                            d.set_digit(2, 0);
                            d.set_digit(3, 0);
                            d.set_digit(4, 0);
                        }
                    }
                    d.display_next();
                    State::Display(DisplayState::Active)
                },
            },
            _ => unreachable!(),
        }
    }
    add_task(Task::new(1, display_start, display_tick));

    // Simple static-purple LED task — only when part2 is not active.
    if !cfg!(feature = "part2") {
    let led_start = State::LED(LEDState::Start);
    fn led_tick(a: State) -> State {
        match a {
            State::LED(b) => match b {
                LEDState::Start => {
                    return State::LED(LEDState::LEDDisplay(RGBDisplay::purple()));
                }
                LEDState::LEDDisplay(mut c) => {
                    c.display_color();
                    return State::LED(LEDState::LEDDisplay(c));
                }
            },
            _ => unreachable!(),
        }
    }
    add_task(Task::new(1, led_start, led_tick));
    }

    if cfg!(feature = "part2") {

    // Selector: decides which color and brightness level to use.
    let selector_start = State::LEDSelector(LEDSelectorState::Start);
    fn led_selector_tick(a: State) -> State {
        match a {
            State::LEDSelector(b) => match b {
                LEDSelectorState::Start => State::LEDSelector(LEDSelectorState::Active),
                LEDSelectorState::Active => {
                    unsafe {
                        let dist  = crate::DISTANCE;
                        let close = crate::CLOSE_RANGE as f64;
                        let med   = crate::MED_RANGE   as f64;
                        let cm    = dist as u32;
                        if dist < close {
                            crate::LED_DISPLAY.turn_purple();
                            crate::LED_DISPLAY.adjust_brightness(10);
                            crate::logger::log_led_selection(b"RedP", 10, cm);
                        } else if dist <= med {
                            crate::LED_DISPLAY.turn_blue();
                            crate::LED_DISPLAY.adjust_brightness(5);
                            crate::logger::log_led_selection(b"Blue", 10, cm);
                        } else {
                            crate::LED_DISPLAY.turn_red();
                            crate::LED_DISPLAY.adjust_brightness(1);
                            crate::logger::log_led_selection(b"Green", 10, cm);
                        }
                    }
                    State::LEDSelector(LEDSelectorState::Active)
                }
            },
            _ => unreachable!(),
        }
    }
    add_task(Task::new(250, selector_start, led_selector_tick));

    // Driver: runs the PWM loop against the shared LED_DISPLAY.
    let driver_start = State::LEDDriver(LEDDriverState::Start);
    fn led_driver_tick(a: State) -> State {
        match a {
            State::LEDDriver(b) => match b {
                LEDDriverState::Start  => State::LEDDriver(LEDDriverState::Active),
                LEDDriverState::Active => {
                    unsafe { crate::LED_DISPLAY.display_color(); }
                    State::LEDDriver(LEDDriverState::Active)
                }
            },
            _ => unreachable!(),
        }
    }
    add_task(Task::new(1, driver_start, led_driver_tick));

    } // end if part2 LED tasks
    } // end if part1

    if cfg!(feature = "part2") {
        // Button: adjusts CLOSE_RANGE / MED_RANGE on press.
        let button_start = State::Button(ButtonState::Start);
        fn button_tick(a: State) -> State {
            match a {
            State::Button(b) => match b {
                ButtonState::Start => {
                    return State::Button(ButtonState::Button(ButtonPress::init()));
                }
                ButtonState::Button(mut c) => {
                    if c.detect_and_handle_press() > 0 {
                        unsafe {
                            crate::CLOSE_RANGE = c.close_range;
                            crate::MED_RANGE   = c.med_range;
                        }
                    }
                    return State::Button(ButtonState::Button(c));
                }
            }
            _ => unreachable!()
            }
        }
        add_task(Task::new(250, button_start, button_tick));
    }

    if cfg!(feature = "part3") {
        let cal_start = State::Calibrator(CalibratorState::Start);
        fn calibrator_tick(a: State) -> State {
            match a {
                State::Calibrator(b) => match b {
                    CalibratorState::Start => State::Calibrator(CalibratorState::Active(Calibrator::init())),
                    CalibratorState::Active(mut c) => {
                        c.calibrator_loop();
                        State::Calibrator(CalibratorState::Active(c))
                    }
                },
                _ => unreachable!(),
            }
        }
        add_task(Task::new(200, cal_start, calibrator_tick));
    }
}
