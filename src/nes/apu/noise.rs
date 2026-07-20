//! NES APU Noise Channel
//! Reference: https://www.nesdev.org/wiki/APU_Noise

const NOISE_TABLE: [u16; 16] = [
    4, 8, 16, 32, 64, 96, 128, 160, 202, 254, 380, 508, 762, 1016, 2034, 4068,
];

const LENGTH_TABLE: [u8; 32] = [
    10, 254, 20, 2, 40, 4, 80, 6, 160, 8, 60, 10, 14, 12, 26, 14,
    12, 16, 24, 18, 48, 20, 96, 22, 192, 24, 72, 26, 16, 28, 32, 30,
];

pub struct Noise {
    enabled: bool,

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

    // LFSR
    shift_register: u16,
    mode: bool,
}

impl Default for Noise {
    fn default() -> Self {
        Self::new()
    }
}

impl Noise {
    pub fn new() -> Self {
        Self {
            enabled: false,
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
            shift_register: 1,
            mode: false,
        }
    }

    /// $400C: --LC.NNNN
    pub fn write_reg0(&mut self, val: u8) {
        self.length_halt = val & 0x20 != 0;
        self.envelope_loop = val & 0x20 != 0;
        self.constant_volume = val & 0x10 != 0;
        self.volume_period = val & 0x0F;
        self.envelope_start = true;
    }

    /// $400E: M---.PPPP
    pub fn write_reg1(&mut self, val: u8) {
        self.mode = val & 0x80 != 0;
        self.timer_period = NOISE_TABLE[(val & 0x0F) as usize];
    }

    /// $400F: LLLL.L---
    pub fn write_reg2(&mut self, val: u8) {
        if self.enabled && !self.length_halt {
            self.length_counter = LENGTH_TABLE[((val >> 3) & 0x1F) as usize];
        }
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

            let bit0 = self.shift_register & 1;
            self.shift_register >>= 1;
            let xor_bit = if self.mode { 6 } else { 1 };
            if bit0 ^ ((self.shift_register >> (xor_bit - 1)) & 1) != 0 {
                self.shift_register |= 0x4000;
            }
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

    /// Half frame clock: length counter
    pub fn clock_half_frame(&mut self) {
        if !self.length_halt && self.length_counter > 0 {
            self.length_counter -= 1;
        }
    }

    /// Returns output level 0.0 ~ 1.0
    pub fn output(&self) -> f32 {
        if !self.enabled || self.length_counter == 0 || self.shift_register & 1 != 0 {
            return 0.0;
        }
        let volume = if self.constant_volume {
            self.volume_period as f32
        } else {
            self.volume_output as f32
        };
        volume / 15.0
    }
}
