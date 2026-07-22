use std::fs;
use std::time::Instant;

use ox6502::bus::nes::NesBus;
use ox6502::cpu::Cpu;
use ox6502::nes::cartridge;
use sdl2::audio::AudioQueue;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: nes_sdl <game.nes>");
        std::process::exit(1);
    }

    let data = fs::read(&args[1]).expect("Failed to read ROM file");
    let cart = cartridge::parse(&data).expect("Failed to parse iNES ROM");

    let bus = NesBus::new(cart);
    let mut cpu = Cpu::new(bus);

    let sdl = sdl2::init().expect("SDL2 init failed");
    let video = sdl.video().expect("SDL2 video init failed");
    let audio_subsystem = sdl.audio().expect("SDL2 audio init failed");

    let window = video
        .window("ox6502", 256 * 3, 240 * 3)
        .position_centered()
        .build()
        .expect("Window creation failed");
    let mut canvas = window.into_canvas().build().expect("Canvas creation failed");
    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture_streaming(sdl2::pixels::PixelFormatEnum::RGB24, 256, 240)
        .expect("Texture creation failed");

    let desired_spec = sdl2::audio::AudioSpecDesired {
        freq: Some(44100),
        channels: Some(1),
        samples: Some(2048),
    };

    let audio_queue: AudioQueue<f32> = audio_subsystem
        .open_queue(None, &desired_spec)
        .expect("Failed to open audio queue");
    audio_queue.resume();

    let mut event_pump = sdl.event_pump().expect("Event pump creation failed");
    let mut frame_timer = Instant::now();
    let frame_duration = std::time::Duration::from_micros(16_667); // ~60 FPS

    // NES joypad button bit constants
    const BTN_A: u8 = 1 << 0;
    const BTN_B: u8 = 1 << 1;
    const BTN_SELECT: u8 = 1 << 2;
    const BTN_START: u8 = 1 << 3;
    const BTN_UP: u8 = 1 << 4;
    const BTN_DOWN: u8 = 1 << 5;
    const BTN_LEFT: u8 = 1 << 6;
    const BTN_RIGHT: u8 = 1 << 7;

    let update_joypad = |bus: &mut NesBus, keycode: Option<sdl2::keyboard::Keycode>, pressed: bool| {
        let bit = match keycode {
            Some(sdl2::keyboard::Keycode::A) => BTN_A,
            Some(sdl2::keyboard::Keycode::S) => BTN_B,
            Some(sdl2::keyboard::Keycode::Backspace) => BTN_SELECT,
            Some(sdl2::keyboard::Keycode::Return) => BTN_START,
            Some(sdl2::keyboard::Keycode::Up) => BTN_UP,
            Some(sdl2::keyboard::Keycode::Down) => BTN_DOWN,
            Some(sdl2::keyboard::Keycode::Left) => BTN_LEFT,
            Some(sdl2::keyboard::Keycode::Right) => BTN_RIGHT,
            _ => return,
        };
        bus.joypad1.set_button(bit, pressed);
    };

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit { .. }
                | sdl2::event::Event::KeyDown {
                    keycode: Some(sdl2::keyboard::Keycode::Escape),
                    ..
                } => break 'running,
                sdl2::event::Event::KeyDown { keycode, .. } => {
                    update_joypad(cpu.bus_mut(), keycode, true);
                }
                sdl2::event::Event::KeyUp { keycode, .. } => {
                    update_joypad(cpu.bus_mut(), keycode, false);
                }
                _ => {}
            }
        }

        // Run CPU until NMI fires (one frame)
        let mut frame_done = false;
        while !frame_done {
            let tick = cpu.step_nes();

            if tick.nmi {
                cpu.handle_nmi();
                frame_done = true;
            }
        }

        // Queue audio samples
        let samples = cpu.bus_mut().apu.take_samples();
        if !samples.is_empty() {
            audio_queue.queue_audio(&samples).expect("Audio queue failed");
        }

        // Copy frame buffer to SDL texture
        let fb = cpu.bus().ppu.frame_buffer();
        texture
            .update(None, fb, 256 * 3)
            .expect("Texture update failed");

        canvas.clear();
        canvas.copy(&texture, None, None).expect("Texture copy failed");
        canvas.present();

        // Throttle to ~60 FPS
        let elapsed = frame_timer.elapsed();
        if elapsed < frame_duration {
            std::thread::sleep(frame_duration - elapsed);
        }
        frame_timer = Instant::now();
    }
}
