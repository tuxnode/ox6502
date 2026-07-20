/// NES APU Triangle Channel
/// Reference: https://www.nesdev.org/wiki/APU_Triangle

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

    pub fn set_enabled(&mut self, val: bool) {
        self.enabled = val;
        if !val {
            self.length_counter = 0;
        }
    }
}
