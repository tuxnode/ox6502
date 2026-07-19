use std::fs;

use ox6502::bus::nes::NesBus;
use ox6502::bus::Bus;
use ox6502::cpu::Cpu;
use ox6502::nes::cartridge;

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

    let mut event_pump = sdl.event_pump().expect("Event pump creation failed");

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit { .. }
                | sdl2::event::Event::KeyDown {
                    keycode: Some(sdl2::keyboard::Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }

        // Run CPU until NMI fires (one frame)
        let mut frame_done = false;
        while !frame_done {
            let step_cycles = cpu.step();
            cpu.cycles += step_cycles as u64;

            let tick = cpu.bus_mut().tick(step_cycles);
            cpu.cycles += tick.extra_cycles as u64;

            if tick.nmi {
                cpu.handle_nmi();
                frame_done = true;
            }
        }

        // Copy frame buffer to SDL texture
        let fb = cpu.bus().ppu.frame_buffer();
        texture
            .update(None, fb, 256 * 3)
            .expect("Texture update failed");

        canvas.clear();
        canvas.copy(&texture, None, None).expect("Texture copy failed");
        canvas.present();
    }
}
