/**
 * APU References: https://www.nesdev.org/wiki/APU_basics
 * Frame Counter: https://www.nesdev.org/wiki/APU_Frame_Counter
 */
pub mod noise;
pub mod pulse;
pub mod triangle;

use noise::Noise;
use pulse::Pulse;
use triangle::Triangle;

pub struct Apu {
    pub pulse1: Pulse,
    pub pulse2: Pulse,
    pub triangle: Triangle,
    pub noise: Noise,
    // dmc: Dmc,

    // Frame counter
    frame_counter: u32,
    frame_mode: u8,        // 0 = 4-step, 1 = 5-step
    frame_interrupt: bool,

    // Sample output
    sample_counter: u32,
    sample_buffer: Vec<f32>,
}

const SAMPLE_RATE: u32 = 44100;
const CPU_FREQ: u32 = 1_789_773;

impl Apu {
    pub fn new() -> Self {
        Self {
            pulse1: Pulse::new(),
            pulse2: Pulse::new(),
            triangle: Triangle::new(),
            noise: Noise::new(),

            frame_counter: 0,
            frame_mode: 0,
            frame_interrupt: false,

            sample_counter: 0,
            sample_buffer: Vec::new(),
        }
    }

    /// $4015 read: status register
    pub fn read_status(&self) -> u8 {
        let mut status = 0;
        if self.pulse1.enabled() { status |= 0x01; }
        if self.pulse2.enabled() { status |= 0x02; }
        if self.triangle.enabled() { status |= 0x04; }
        if self.noise.enabled() { status |= 0x08; }
        // if self.dmc.enabled() { status |= 0x10; }
        if self.frame_interrupt { status |= 0x40; }
        status
    }

    /// $4015 write: enable flags
    pub fn write_status(&mut self, val: u8) {
        self.pulse1.set_enabled(val & 0x01 != 0);
        self.pulse2.set_enabled(val & 0x02 != 0);
        self.triangle.set_enabled(val & 0x04 != 0);
        self.noise.set_enabled(val & 0x08 != 0);
        // self.dmc.set_enabled(val & 0x10 != 0);
        self.frame_interrupt = false;
    }

    /// $4017 write: frame counter mode
    pub fn write_frame_counter(&mut self, val: u8) {
        self.frame_mode = (val >> 7) & 1;
        self.frame_counter = 0;
        if self.frame_mode == 1 {
            self.clock_quarter_frame();
            self.clock_half_frame();
        }
    }

    /// Called every CPU cycle
    pub fn tick(&mut self) {
        // Clock all channel timers
        self.pulse1.tick();
        self.pulse2.tick();
        self.triangle.tick();
        self.noise.tick();

        // Frame counter (CPU cycles)
        self.frame_counter += 1;
        match self.frame_mode {
            0 => self.tick_frame_4step(),
            1 => self.tick_frame_5step(),
            _ => {}
        }

        // Collect samples
        self.sample_counter += 1;
        if self.sample_counter >= CPU_FREQ / SAMPLE_RATE {
            self.sample_counter = 0;
            self.sample_buffer.push(self.mix());
        }
    }

    fn tick_frame_4step(&mut self) {
        match self.frame_counter {
            7457 => {
                self.clock_quarter_frame();
            }
            14913 => {
                self.clock_quarter_frame();
                self.clock_half_frame();
            }
            22371 => {
                self.clock_quarter_frame();
            }
            29829 => {
                self.clock_quarter_frame();
                self.clock_half_frame();
                self.frame_interrupt = true;
            }
            29830 => {
                self.frame_counter = 0;
            }
            _ => {}
        }
    }

    fn tick_frame_5step(&mut self) {
        match self.frame_counter {
            7457 => {
                self.clock_quarter_frame();
            }
            14913 => {
                self.clock_quarter_frame();
                self.clock_half_frame();
            }
            22371 => {
                self.clock_quarter_frame();
            }
            29829 => {
                self.clock_quarter_frame();
                self.clock_half_frame();
            }
            37281 => {
                self.clock_quarter_frame();
                self.clock_half_frame();
            }
            37282 => {
                self.frame_counter = 0;
            }
            _ => {}
        }
    }

    fn clock_quarter_frame(&mut self) {
        self.pulse1.clock_quarter_frame();
        self.pulse2.clock_quarter_frame();
        self.triangle.clock_quarter_frame();
        self.noise.clock_quarter_frame();
    }

    fn clock_half_frame(&mut self) {
        self.pulse1.clock_half_frame();
        self.pulse2.clock_half_frame();
        self.triangle.clock_half_frame();
        self.noise.clock_half_frame();
    }

    /// Mix channels to single sample (0.0 ~ 1.0)
    fn mix(&self) -> f32 {
        let pulse1 = self.pulse1.output();
        let pulse2 = self.pulse2.output();
        let triangle = self.triangle.output();
        let noise = self.noise.output();

        let pulse_out = 0.00752 * (pulse1 + pulse2);
        let tnd_out = 0.00851 * triangle + 0.00494 * noise;

        pulse_out + tnd_out
    }

    /// Take buffered samples
    pub fn take_samples(&mut self) -> Vec<f32> {
        std::mem::take(&mut self.sample_buffer)
    }
}
