use std::fs;
use std::io::Write;

use ox6502::bus::nes::NesBus;
use ox6502::bus::Bus;
use ox6502::cpu::Cpu;
use ox6502::nes::cartridge;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: nes_render <game.nes> [frames]");
        std::process::exit(1);
    }

    let path = &args[1];
    let max_frames: u64 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(3);

    let data = fs::read(path).expect("Failed to read ROM file");
    let cart = cartridge::parse(&data).expect("Failed to parse iNES ROM");
    println!("Mapper: {}", cart.mapper);
    println!("PRG ROM: {} bytes", cart.prg_rom.len());
    println!("CHR ROM: {} bytes", cart.chr_rom.len());
    println!("Mirroring: {:?}", cart.mirroring);

    let bus = NesBus::new(cart);
    let mut cpu = Cpu::new(bus);

    let mut completed_frames = 0;

    loop {
        let step_cycles = cpu.step();
        cpu.cycles += step_cycles as u64;

        let tick = cpu.bus_mut().tick(step_cycles);
        cpu.cycles += tick.extra_cycles as u64;

        if tick.nmi {
            cpu.handle_nmi();
            completed_frames += 1;
            eprintln!("Frame {} completed at cycle {}", completed_frames, cpu.cycles);

            if completed_frames >= max_frames {
                break;
            }
        }
    }

    let fb = cpu.bus().ppu.frame_buffer();
    let ppm_path = "frame.ppm";
    let mut f = fs::File::create(ppm_path).expect("Failed to create PPM file");
    f.write_all(format!("P6\n256 240\n255\n").as_bytes())
        .expect("Failed to write PPM header");
    f.write_all(fb).expect("Failed to write PPM data");
    println!("Wrote {}", ppm_path);
}
