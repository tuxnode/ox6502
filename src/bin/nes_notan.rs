use std::collections::VecDeque;
use std::fs;
use std::sync::{Arc, Mutex};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use notan::draw::*;
use notan::prelude::*;

use ox6502::bus::nes::NesBus;
use ox6502::cpu::Cpu;
use ox6502::nes::cartridge;

const NES_WIDTH: u32 = 256;
const NES_HEIGHT: u32 = 240;
const SCALE: u32 = 3;

/// Push the APU's 44.1 kHz mono output to the default audio device through
/// cpal (a pure-Rust audio API), resampling if the device runs at another rate.
struct AudioSink {
    _stream: cpal::Stream,
    queue: Arc<Mutex<VecDeque<f32>>>,
}

impl AudioSink {
    fn new() -> Option<Self> {
        let host = cpal::default_host();
        let device = host.default_output_device()?;
        let supported = device.default_output_config().ok()?;
        let sample_format = supported.sample_format();
        let config: cpal::StreamConfig = supported.into();
        let channels = config.channels as usize;
        let step = 44100.0 / config.sample_rate as f64;
        let queue = Arc::new(Mutex::new(VecDeque::with_capacity(8192)));

        let stream = match sample_format {
            cpal::SampleFormat::F32 => {
                build_stream::<f32>(&device, &config, &queue, channels, step)
            }
            cpal::SampleFormat::I16 => {
                build_stream::<i16>(&device, &config, &queue, channels, step)
            }
            cpal::SampleFormat::U16 => {
                build_stream::<u16>(&device, &config, &queue, channels, step)
            }
            cpal::SampleFormat::I8 => build_stream::<i8>(&device, &config, &queue, channels, step),
            cpal::SampleFormat::U8 => build_stream::<u8>(&device, &config, &queue, channels, step),
            cpal::SampleFormat::I32 => {
                build_stream::<i32>(&device, &config, &queue, channels, step)
            }
            cpal::SampleFormat::U32 => {
                build_stream::<u32>(&device, &config, &queue, channels, step)
            }
            cpal::SampleFormat::F64 => {
                build_stream::<f64>(&device, &config, &queue, channels, step)
            }
            format => {
                eprintln!("Unsupported audio sample format: {format:?}");
                None
            }
        }?;

        stream.play().ok()?;
        Some(Self {
            _stream: stream,
            queue,
        })
    }

    fn push(&self, samples: &[f32]) {
        let mut queue = self.queue.lock().unwrap();
        // Never let a stalled device make the queue grow without bound.
        if queue.len() < 44_100 {
            queue.extend(samples.iter().copied());
        }
    }
}

fn build_stream<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    queue: &Arc<Mutex<VecDeque<f32>>>,
    channels: usize,
    step: f64,
) -> Option<cpal::Stream>
where
    T: cpal::SizedSample + cpal::FromSample<f32>,
{
    let queue = Arc::clone(queue);
    let mut fraction = 0.0f64;
    let mut current = 0.0f32;
    let err_fn = |err| eprintln!("Audio stream error: {err}");

    device
        .build_output_stream(
            config.clone(),
            move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                let mut queue = queue.lock().unwrap();
                for frame in data.chunks_mut(channels) {
                    // Linear resampling from 44100 Hz to the device rate.
                    while fraction <= 0.0 {
                        current = queue.pop_front().unwrap_or(0.0);
                        fraction += step;
                    }
                    fraction -= 1.0;

                    let value = T::from_sample(current);
                    for sample in frame.iter_mut() {
                        *sample = value;
                    }
                }
            },
            err_fn,
            None,
        )
        .ok()
}

#[derive(AppState)]
struct NesApp {
    cpu: Cpu<NesBus>,
    texture: Texture,
    rgba: Vec<u8>,
    audio: Option<AudioSink>,
}

impl NesApp {
    fn new(gfx: &mut Graphics, cart: cartridge::Cartridge) -> Self {
        let bus = NesBus::new(cart);
        let cpu = Cpu::new(bus);
        let rgba = vec![0u8; (NES_WIDTH * NES_HEIGHT * 4) as usize];
        let texture = gfx
            .create_texture()
            .from_bytes(&rgba, NES_WIDTH, NES_HEIGHT)
            .with_filter(TextureFilter::Nearest, TextureFilter::Nearest)
            .build()
            .expect("Failed to create NES texture");

        Self {
            cpu,
            texture,
            rgba,
            audio: AudioSink::new(),
        }
    }

    /// Run the CPU/PPU until the PPU finishes the current frame.
    fn run_frame(&mut self) {
        let frame_before = self.cpu.bus().ppu.frame();
        while self.cpu.bus().ppu.frame() == frame_before {
            let tick = self.cpu.step_nes();
            if tick.nmi {
                self.cpu.handle_nmi();
            }
        }

        // Always drain the APU buffer so it does not grow without bound,
        // then hand the samples to the audio device if one is available.
        let samples = self.cpu.bus_mut().apu.take_samples();
        if let Some(audio) = &self.audio {
            audio.push(&samples);
        }

        // Expand the PPU's RGB frame buffer to RGBA for the GPU texture.
        let frame = self.cpu.bus().ppu.frame_buffer();
        for (dst, src) in self.rgba.chunks_exact_mut(4).zip(frame.chunks_exact(3)) {
            dst[..3].copy_from_slice(src);
            dst[3] = 0xFF;
        }
    }
}

#[notan_main]
fn main() -> Result<(), String> {
    let path = std::env::args()
        .nth(1)
        .ok_or_else(|| "Usage: nes_notan <game.nes>".to_string())?;
    let data = fs::read(&path).map_err(|e| format!("Failed to read ROM file: {e}"))?;
    let cart = cartridge::parse(&data).map_err(|e| format!("Failed to parse iNES ROM: {e}"))?;

    let window = WindowConfig::new()
        .set_title("ox6502")
        .set_size(NES_WIDTH * SCALE, NES_HEIGHT * SCALE)
        .set_vsync(true);

    notan::init_with(move |gfx: &mut Graphics| NesApp::new(gfx, cart))
        .add_config(window)
        .add_config(DrawConfig)
        .update(update)
        .draw(draw)
        .build()
}

fn update(app: &mut App, state: &mut NesApp) {
    if app.keyboard.was_pressed(KeyCode::Escape) {
        app.exit();
    }

    // NES joypad bit order: A, B, Select, Start, Up, Down, Left, Right.
    let start = app.keyboard.is_down(KeyCode::Enter) || app.keyboard.is_down(KeyCode::NumpadEnter);
    state.cpu.bus_mut().joypad1.set_button(1 << 3, start);

    let keys = [
        (KeyCode::KeyA, 1u8 << 0),
        (KeyCode::KeyS, 1 << 1),
        (KeyCode::Backspace, 1 << 2),
        (KeyCode::ArrowUp, 1 << 4),
        (KeyCode::ArrowDown, 1 << 5),
        (KeyCode::ArrowLeft, 1 << 6),
        (KeyCode::ArrowRight, 1 << 7),
    ];
    for (key, bit) in keys {
        state
            .cpu
            .bus_mut()
            .joypad1
            .set_button(bit, app.keyboard.is_down(key));
    }

    state.run_frame();
}

fn draw(gfx: &mut Graphics, state: &mut NesApp) {
    gfx.update_texture(&mut state.texture)
        .with_data(&state.rgba)
        .update()
        .expect("Failed to update NES texture");

    let mut draw = gfx.create_draw();
    draw.clear(Color::BLACK);
    draw.image(&state.texture)
        .position(0.0, 0.0)
        .size((NES_WIDTH * SCALE) as f32, (NES_HEIGHT * SCALE) as f32);
    gfx.render(&draw);
}
