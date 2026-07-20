/// NES APU Pulse Channel
/// Reference: https://www.nesdev.org/wiki/APU_Pulse

const DUTY_TABLE: [[f32; 8]; 4] = [
    [0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], // 12.5%
    [0.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0], // 25%
    [0.0, 1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0], // 50%
    [1.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 1.0], // 75%
];

const LENGTH_TABLE: [u8; 32] = [
    10, 254, 20, 2, 40, 4, 80, 6, 160, 8, 60, 10, 14, 12, 26, 14,
    12, 16, 24, 18, 48, 20, 96, 22, 192, 24, 72, 26, 16, 28, 32, 30,
];

pub struct Pulse {
    enabled: bool,

    // Duty cycle
    duty_table: [f32; 8],
    duty_pos: u8,

    // Timer
    timer_period: u16,
    timer_counter: u16,

    // Length counter
    length_counter: u8,
    length_halt: bool,

    // Envelope
    constant_volume: bool,
    volume_period: u8,
    volume_divider: u8,
    volume_output: u8,
    envelope_loop: bool,
    envelope_start: bool,

    // Sweep
    sweep_enabled: bool,
    sweep_period: u8,
    sweep_negate: bool,
    sweep_shift: u8,
    sweep_divider: u8,
    sweep_reload: bool,
    sweep_mute: bool,
}

impl Pulse {
    pub fn new() -> Self {
        Self {
            enabled: false,
            duty_table: DUTY_TABLE[0],
            duty_pos: 0,
            timer_period: 0,
            timer_counter: 0,
            length_counter: 0,
            length_halt: false,
            constant_volume: false,
            volume_period: 0,
            volume_divider: 0,
            volume_output: 0,
            envelope_loop: false,
            envelope_start: false,
            sweep_enabled: false,
            sweep_period: 0,
            sweep_negate: false,
            sweep_shift: 0,
            sweep_divider: 0,
            sweep_reload: false,
            sweep_mute: false,
        }
    }

    /// $4000/$4004: DDLCNNNN
    pub fn write_reg0(&mut self, val: u8) {
        self.length_halt = val & 0x20 != 0;
        self.envelope_loop = val & 0x20 != 0;
        self.constant_volume = val & 0x10 != 0;
        self.volume_period = val & 0x0F;
        let duty_idx = (val >> 6) as usize;
        self.duty_table = DUTY_TABLE[duty_idx];
        self.envelope_start = true;
    }

    /// $4001/$4005: EPPPNSSS
    pub fn write_reg1(&mut self, val: u8) {
        self.sweep_enabled = val & 0x80 != 0;
        self.sweep_period = (val >> 4) & 0x07;
        self.sweep_negate = val & 0x08 != 0;
        self.sweep_shift = val & 0x07;
        self.sweep_reload = true;
    }

    /// $4002/$4006: TTTTTTTT
    pub fn write_reg2(&mut self, val: u8) {
        self.timer_period = (self.timer_period & 0x700) | val as u16;
    }

    /// $4003/$4007: LLLLL...TTT
    pub fn write_reg3(&mut self, val: u8) {
        self.timer_period = (self.timer_period & 0x0FF) | ((val as u16 & 0x07) << 8);
        if self.enabled && !self.length_halt {
            self.length_counter = LENGTH_TABLE[((val >> 3) & 0x1F) as usize];
        }
        self.duty_pos = 0;
        self.envelope_start = true;
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

    /// Called every CPU cycle
    pub fn tick(&mut self) {
        if self.timer_counter == 0 {
            self.timer_counter = self.timer_period;
            self.duty_pos = (self.duty_pos + 1) & 7;
        } else {
            self.timer_counter -= 1;
        }
    }

    /// Quarter frame clock: envelope
    pub fn clock_quarter_frame(&mut self) {
        if self.envelope_start {
            self.envelope_start = false;
            self.volume_output = 15;
            self.volume_divider = self.volume_period;
        } else if self.volume_divider == 0 {
            self.volume_divider = self.volume_period;
            if self.volume_output > 0 {
                self.volume_output -= 1;
            } else if self.envelope_loop {
                self.volume_output = 15;
            }
        } else {
            self.volume_divider -= 1;
        }
    }

    /// Half frame clock: length counter + sweep
    pub fn clock_half_frame(&mut self) {
        if !self.length_halt && self.length_counter > 0 {
            self.length_counter -= 1;
        }
        self.clock_sweep();
    }

    fn clock_sweep(&mut self) {
        let delta = self.timer_period >> self.sweep_shift;
        let target = if self.sweep_negate {
            self.timer_period.wrapping_sub(delta)
        } else {
            self.timer_period.wrapping_add(delta)
        };

        self.sweep_mute = self.timer_period < 8 || target > 0x7FF;

        if self.sweep_divider == 0 && self.sweep_enabled && !self.sweep_mute && self.length_counter > 0 {
            self.timer_period = target;
        }

        if self.sweep_divider == 0 || self.sweep_reload {
            self.sweep_divider = self.sweep_period;
            self.sweep_reload = false;
        } else {
            self.sweep_divider -= 1;
        }
    }

    /// Returns output level 0.0 ~ 1.0
    pub fn output(&self) -> f32 {
        if !self.enabled || self.length_counter == 0 || self.sweep_mute {
            return 0.0;
        }
        let sample = self.duty_table[self.duty_pos as usize];
        let volume = if self.constant_volume {
            self.volume_period as f32
        } else {
            self.volume_output as f32
        };
        sample * (volume / 15.0)
    }
}
