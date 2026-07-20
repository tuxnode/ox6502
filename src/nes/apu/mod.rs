/**
 * APU References: https://www.nesdev.org/wiki/APU_basics
 */
mod noise;
mod pulse;
mod triangle;

use noise::Noise;
use pulse::Pulse;
use triangle::Triangle;

pub struct Apu {
    pulse1: Pulse,
    pulse2: Pulse,
    triangle: Triangle,
    noise: Noise,
    // dmc: Dmc,
    // frame_counter_mode: u8,
    // frame_step: u8,
    // frame_counter: u32,
    // sample_divider: u32,
    // sample_buffer: Vec<f32>,
}
