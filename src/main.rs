/*
 * MOS 6502 -> CMOS 5C02 Emulator
 * Integration Testing Kit: https://github.com/Klaus2m5/6502_65C02_functional_tests
 */

use ox6502::bus::simple::SimpleBus;
use ox6502::cpu::Cpu;
use std::fs;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let debug = args.iter().any(|a| a == "--debug");
    let test_file = args.get(1).filter(|a| !a.starts_with('-')).or_else(|| {
        args.iter().skip(1).find(|a| !a.starts_with('-'))
    });

    let test_file = match test_file {
        Some(f) => f.clone(),
        None => {
            println!("Usage: ox6502 <test.bin> [--debug]");
            println!("Available tests:");
            println!("  tests/roms/6502_functional_test.bin");
            println!("  tests/roms/65C02_extended_opcodes_test.bin");
            std::process::exit(1);
        }
    };

    let rom = fs::read(&test_file).expect("Failed to read test ROM");
    println!("Loaded {} bytes from {}", rom.len(), test_file);

    let mut bus = SimpleBus::new();
    bus.load(&rom, 0x0000);

    let mut cpu = Cpu::new(bus);
    cpu.pc = 0x0400;

    if debug {
        ox6502::monitor::run(&mut cpu);
        return;
    }

    println!("CPU initialized. Starting execution at $0400...");

    let mut cycles: u64 = 0;
    let max_cycles = 10_000_000;
    let mut trap_count = 0;
    let mut traps: Vec<(u16, u64)> = Vec::new();

    loop {
        let pc_before = cpu.pc;
        let step_cycles = cpu.step();
        cycles += step_cycles as u64;
        cpu.cycles = cycles;

        if cpu.pc == pc_before {
            trap_count += 1;
            traps.push((cpu.pc, cycles));
            if trap_count > 20 {
                println!("\n=== TOO MANY TRAPS, STOPPING ===");
                break;
            }
            cpu.pc = cpu.pc.wrapping_add(2);
            continue;
        }

        if cpu.pc == 0x0400 {
            println!("\n=== TEST PASSED ===");
            println!("Cycles: {}", cycles);
            break;
        }

        if cycles >= max_cycles {
            println!("\n=== TIMEOUT ===");
            println!("Exceeded {} cycles", max_cycles);
            println!("PC: ${:04X}", cpu.pc);
            break;
        }
    }

    println!("\n=== TRAP SUMMARY ===");
    for (i, (addr, cyc)) in traps.iter().enumerate() {
        println!("Trap #{}: ${:04X} at {} cycles", i + 1, addr, cyc);
    }
    println!("Total traps: {}", trap_count);

    println!("\nFinal state:");
    println!("A:  ${:02X}  X: ${:02X}  Y: ${:02X}", cpu.a, cpu.x, cpu.y);
    println!("SP: ${:02X}  PC: ${:04X}", cpu.sp, cpu.pc);
}
