/// NES APU Triangle Channel
/// Reference: https://www.nesdev.org/wiki/APU_Triangle

const TRIANGLE_TABLE: [f32; 32] = [
    15.0, 14.0, 13.0, 12.0, 11.0, 10.0, 9.0, 8.0,
    7.0, 6.0, 5.0, 4.0, 3.0, 2.0, 1.0, 0.0,
    0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0,
    8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0,
];

const LENGTH_TABLE: [u8; 32] = [
    10, 254, 20, 2, 40, 4, 80, 6, 160, 8, 60, 10, 14, 12, 26, 14,
    12, 16, 24, 18, 48, 20, 96, 22, 192, 24, 72, 26, 16, 28, 32, 30,
];

pub struct Triangle {
    enabled: bool,

    // Timer
    timer_period: u16,
    timer_counter: u16,

    // Length Counter
    length_counter: u8,
    length_halt: bool,

    // Linear counter
    linear_counter: u8,
    linear_reload: u8,

    // Phase -- Index TRIANGLE_TABLE
    duty_pos: u8,
}

impl Triangle {
    pub fn new() -> Self {
        Self {
            enabled: false,
            timer_period: 0,
            timer_counter: 0,
            length_counter: 0,
            length_halt: false,
            linear_counter: 0,
            linear_reload: 0,
            duty_pos: 0,
        }
    }

    pub fn write_reg0(&mut self, val: u8) {
        self.length_halt = val & 0x80 != 0;
        self.linear_reload = val & 0x7F;
    }

    pub fn write_reg2(&mut self, val: u8) {
        self.timer_period = (self.timer_period & 0x700) | val as u16;
    }

    pub fn write_reg3(&mut self, val: u8) {
        self.timer_period = (self.timer_period & 0x0FF) | ((val as u16 & 0x07) << 8);
        if self.enabled && !self.length_halt {
            self.length_counter = LENGTH_TABLE[((val >> 3) & 0x1F) as usize];
        }
        self.duty_pos = 0;
        self.linear_reload = self.linear_counter;
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_enabled(&mut self, val: bool) {
        self.enabled = val;
        if !val {
            self.length_counter = 0;
        }
    }

    /// Called every CPU cycle — timer decreases by 2 (double speed)
    pub fn tick(&mut self) {
        if self.timer_counter == 0 {
            self.timer_counter = self.timer_period;
            self.duty_pos = (self.duty_pos + 1) & 31;
        } else {
            self.timer_counter = self.timer_counter.saturating_sub(2);
        }
    }

    /// Quarter frame clock: linear counter
    pub fn clock_quarter_frame(&mut self) {
        if self.linear_reload > 0 {
            self.linear_counter = self.linear_reload;
        } else {
            self.linear_counter = self.linear_counter.saturating_sub(1);
        }
        if self.length_halt {
            self.linear_reload = 0;
        }
    }

    /// Half frame clock: length counter
    pub fn clock_half_frame(&mut self) {
        if !self.length_halt && self.length_counter > 0 {
            self.length_counter -= 1;
        }
    }

    /// Returns output level 0.0 ~ 1.0
    pub fn output(&self) -> f32 {
        if !self.enabled || self.length_counter == 0 || self.linear_counter == 0 {
            return 0.0;
        }
        TRIANGLE_TABLE[self.duty_pos as usize] / 15.0
    }
}
