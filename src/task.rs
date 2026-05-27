use crate::rgb::RGBDisplay;
use crate::button::ButtonPress;
use crate::calibrator::Calibrator;

pub enum SonarState {
    Start,
    Measure
}
pub enum DisplayState {
    Start,
    Active,
}
pub enum LEDState {
    Start,
    LEDDisplay(RGBDisplay)
}
pub enum ButtonState {
   Start,
   Button(ButtonPress)
}
pub enum LEDSelectorState {
    Start,
    Active,
}
pub enum LEDDriverState {
    Start,
    Active,
}
pub enum CalibratorState {
    Start,
    Active(Calibrator),
}

// Lab-6 part-2 states
pub enum JoystickTaskState { Start, Active }
pub enum LCDTaskState      { Start, Active }
pub enum StepperTaskState  { Start, Active }

pub enum State {
    Sonar(SonarState),
    Display(DisplayState),
    LED(LEDState),
    Button(ButtonState),
    LEDSelector(LEDSelectorState),
    LEDDriver(LEDDriverState),
    Calibrator(CalibratorState),
    Joystick(JoystickTaskState),
    LCDDisplay(LCDTaskState),
    StepperCtrl(StepperTaskState),
}

pub struct Task {
    pub current_state: State,
    pub period: u32,
    pub elapsed_time: u32,
    pub tick: fn(State) -> State,
}

impl Task {
    pub fn new(p: u32, start_state: State, tick: fn(State) -> State) -> Self {
        Self {
            current_state: start_state,
            period: p,
            elapsed_time: 0,
            tick,
        }
    }

    fn ticker(&mut self) {
        if self.elapsed_time >= self.period {
            self.elapsed_time = 0;
            let next = (self.tick)(unsafe { core::ptr::read(&self.current_state) });
            unsafe { core::ptr::write(&mut self.current_state, next) };
        }
    }

    fn update_time(&mut self, amt: u32) {
        self.elapsed_time += amt;
    }

    pub fn update_time_and_tick(&mut self, amt: u32) {
        self.update_time(amt);
        self.ticker();
    }
}
