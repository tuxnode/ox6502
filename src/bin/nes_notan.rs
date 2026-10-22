use std::collections::VecDeque;
use std::fs;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use notan::draw::*;
use notan::prelude::*;

use ox6502::bus::nes::NesBus;
use ox6502::cpu::Cpu;
use ox6502::nes::cartridge;

const NES_WIDTH: u32 = 256;
const NES_HEIGHT: u32 = 240;
const SCALE: u32 = 3;
const APU_SAMPLE_RATE: u32 = 44100;
/// NTSC NES frame rate (89342 PPU dots per frame ≈ 60.0988 Hz).
const NES_FPS: f64 = 60.098_8;
/// Keep roughly 50 ms of audio queued. This absorbs scheduler/device jitter
/// while keeping latency low enough for games.
const AUDIO_TARGET_SAMPLES: usize = 2205;
/// Hard limit; beyond this latency would be noticeable, so drop old audio.
const AUDIO_MAX_SAMPLES: usize = AUDIO_TARGET_SAMPLES * 3;

/// Linux: stream to PulseAudio/PipeWire-pulse using a native-Rust
/// implementation of the PulseAudio protocol. This avoids linking
/// libasound2 (and therefore libasound2-dev) entirely.
#[cfg(target_os = "linux")]
struct AudioSink {
    _client: pulseaudio::Client,
    _stream: pulseaudio::PlaybackStream,
    queue: Arc<Mutex<VecDeque<i16>>>,
}

#[cfg(target_os = "linux")]
impl AudioSink {
    fn new() -> Option<Self> {
        use pulseaudio::AsPlaybackSource;
        use pulseaudio::protocol::{
            ChannelMap, ChannelVolume, DEFAULT_SINK, PlaybackStreamParams, SampleFormat, SampleSpec,
        };

        let client = pulseaudio::Client::from_env(c"ox6502").ok()?;
        let mut params = PlaybackStreamParams {
            sample_spec: SampleSpec {
                format: SampleFormat::S16Le,
                channels: 1,
                sample_rate: APU_SAMPLE_RATE,
            },
            channel_map: ChannelMap::mono(),
            sink_name: Some(DEFAULT_SINK.to_owned()),
            cvolume: Some(ChannelVolume::norm(1)),
            ..Default::default()
        };
        // Ask for small (~23 ms) server-side requests instead of PulseAudio's
        // ~2 second default. This keeps the client queue short and stable.
        params.buffer_attr.max_length = 8192;
        params.buffer_attr.target_length = 2048;
        params.buffer_attr.pre_buffering = 2048;
        params.buffer_attr.minimum_request_length = 512;

        let queue = Arc::new(Mutex::new(VecDeque::with_capacity(8192)));
        let callback_queue = Arc::clone(&queue);
        let callback = move |buf: &mut [u8]| {
            let mut queue = callback_queue.lock().unwrap();
            for chunk in buf.chunks_mut(2) {
                let sample = queue.pop_front().unwrap_or(0i16).to_le_bytes();
                chunk.copy_from_slice(&sample[..chunk.len()]);
            }
            buf.len()
        };

        let stream = futures::executor::block_on(
            client.create_playback_stream(params, callback.as_playback_source()),
        )
        .ok()?;

        Some(Self {
            _client: client,
            _stream: stream,
            queue,
        })
    }

    fn push(&self, samples: &[f32]) {
        let mut queue = self.queue.lock().unwrap();
        queue.extend(
            samples
                .iter()
                .map(|s| (s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16),
        );
        while queue.len() > AUDIO_MAX_SAMPLES {
            queue.pop_front();
        }
    }

    fn buffered_samples(&self) -> usize {
        self.queue.lock().unwrap().len()
    }
}

/// macOS/Windows (and other non-Linux Unixes): use cpal, which talks to
/// CoreAudio/WASAPI without any external SDK. Resample to the device rate.
#[cfg(not(target_os = "linux"))]
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

#[cfg(not(target_os = "linux"))]
struct AudioSink {
    _stream: cpal::Stream,
    queue: Arc<Mutex<VecDeque<f32>>>,
}

#[cfg(not(target_os = "linux"))]
impl AudioSink {
    fn new() -> Option<Self> {
        let host = cpal::default_host();
        let device = host.default_output_device()?;
        let supported = device.default_output_config().ok()?;
        let sample_format = supported.sample_format();
        let config: cpal::StreamConfig = supported.into();
        let channels = config.channels as usize;
        let step = APU_SAMPLE_RATE as f64 / config.sample_rate as f64;
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
        queue.extend(samples.iter().copied());
        while queue.len() > AUDIO_MAX_SAMPLES {
            queue.pop_front();
        }
    }

    fn buffered_samples(&self) -> usize {
        self.queue.lock().unwrap().len()
    }
}

#[cfg(not(target_os = "linux"))]
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
    last_update: Instant,
    frame_accumulator: f64,
    smoothed_buffer: f64,
    primed: bool,
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
            last_update: Instant::now(),
            frame_accumulator: 0.0,
            smoothed_buffer: AUDIO_TARGET_SAMPLES as f64,
            primed: false,
        }
    }

    /// Fill the audio queue before regular playback starts, otherwise the
    /// first few seconds would run with an empty buffer and crackle.
    fn prime_audio(&mut self) {
        if self.audio.is_some() {
            while self
                .audio
                .as_ref()
                .is_some_and(|a| a.buffered_samples() < AUDIO_TARGET_SAMPLES)
            {
                self.run_frame();
            }
            if let Some(audio) = &self.audio {
                self.smoothed_buffer = audio.buffered_samples() as f64;
            }
        } else {
            self.run_frame();
        }

        self.primed = true;
        self.frame_accumulator = 0.0;
        self.last_update = Instant::now();
    }

    /// Advance emulation based on wall-clock time, not display refresh rate.
    /// The audio queue level trims the emulation rate slightly so the sound
    /// device and the emulated console cannot drift apart over time.
    fn advance(&mut self) {
        if !self.primed {
            self.prime_audio();
            return;
        }

        let now = Instant::now();
        let since_last = now.duration_since(self.last_update).as_secs_f64();
        self.last_update = now;

        // After the window was stalled/minimized the audio queue has drained.
        // Refill it instead of trying to catch up on several frames at once.
        if since_last > 0.25 {
            self.prime_audio();
            return;
        }

        let elapsed = since_last.min(0.1);

        let mut rate = NES_FPS;
        if let Some(buffered) = self.audio.as_ref().map(AudioSink::buffered_samples) {
            self.smoothed_buffer = self.smoothed_buffer * 0.9 + buffered as f64 * 0.1;
            let error =
                (self.smoothed_buffer - AUDIO_TARGET_SAMPLES as f64) / AUDIO_TARGET_SAMPLES as f64;
            // ±0.5% is inaudible, but enough to correct clock drift.
            rate *= 1.0 - (error * 0.02).clamp(-0.005, 0.005);
        }

        self.frame_accumulator += elapsed * rate;
        // Avoid a burst of catch-up frames after the window was stalled.
        self.frame_accumulator = self.frame_accumulator.min(3.0);

        let mut ran = 0;
        while self.frame_accumulator >= 1.0 && ran < 3 {
            self.run_frame();
            self.frame_accumulator -= 1.0;
            ran += 1;
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

    state.advance();
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
