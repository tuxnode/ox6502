# ox6502

MOS 6502 / CMOS W65C02 CPU emulator with NES system emulation, written in Rust.

![MOS 6502](https://www.masswerk.at/6502/assets/MOS_6502AD_4585.jpg)

## Features

### CPU
- Full 6502/65C02 instruction set support
- 247/256 NMOS illegal opcodes (SST test pass rate: 96.5%)
- All 13 addressing modes
- Bus abstraction for flexible memory mapping
- Interactive debugger/monitor

### NES System
- iNES cartridge parser (header, PRG ROM, CHR ROM, mirroring)
- NES CPU memory map with address routing
- PPU register interface ($2000-$2007) with loopy address system
- PPU memory map (pattern tables, nametables, palette)
- OAM DMA ($4014) with cycle penalty
- NMI interrupt handling
- CPU main loop with DMA/NMI support

## Project Structure
```
ox6502/
├── src/
│   ├── lib.rs
│   ├── main.rs
│   ├── cpu.rs              # CPU registers and core operations
│   ├── instructions.rs     # Instruction implementations
│   ├── addressing.rs       # Addressing modes
│   ├── opcodes.rs          # Opcode constants
│   ├── bus/
│   │   ├── mod.rs          # Bus trait (cpu_read/write, ppu_read/write, tick)
│   │   ├── simple.rs       # SimpleBus (64KB flat memory for CPU tests)
│   │   └── nes.rs          # NesBus (NES address routing + PPU/APU)
│   ├── nes/
│   │   ├── mod.rs
│   │   ├── cartridge.rs    # iNES parser
│   │   ├── ppu.rs          # PPU registers and memory
│   │   └── mapper/
│   │       ├── mod.rs
│   │       └── nrom.rs     # NROM mapper (stub)
│   └── monitor/
│       ├── mod.rs
│       ├── commands.rs
│       └── disass.rs
└── tests/
    ├── cpu_tests.rs        # CPU integration tests
    └── sst_tests.rs        # SST opcode tests
```

## Build & Test
```bash
cargo build
cargo test
```

## CPU Test ROM
```bash
cargo run -- tests/roms/6502_functional_test.bin
cargo run -- tests/roms/6502_functional_test.bin --debug
```

## Monitor Commands

| Command | Description |
|---------|-------------|
| `s`, `step` | Execute one instruction |
| `c`, `continue` | Run until breakpoint or trap |
| `r`, `regs` | Show registers and flags |
| `d [addr] [count]` | Disassemble instructions |
| `m [addr] [len]` | Hex dump memory |
| `b <addr>` | Set breakpoint |
| `bc <id>` | Clear breakpoint by id |
| `bl` | List all breakpoints |
| `t [count]` | Show trace history |
| `h`, `help` | Show help |
| `q`, `quit` | Exit monitor |

Press Enter to repeat the last command.

## NES CPU Memory Map

| Address | Size | Device |
|---------|------|--------|
| $0000-$1FFF | 2KB | Internal RAM (mirrored every $800) |
| $2000-$3FFF | 8B | PPU registers (mirrored every 8 bytes) |
| $4000-$4017 | 24B | APU and I/O |
| $4014 | 1B | OAM DMA |
| $6000-$7FFF | 8KB | Cartridge PRG RAM |
| $8000-$FFFF | 32KB | Cartridge PRG ROM (NROM) |

## PPU Registers

| Address | Name | Description |
|---------|------|-------------|
| $2000 | PPUCTRL | NMI enable, pattern table select, VRAM increment |
| $2001 | PPUMASK | Greyscale, left 8px clip, BG/Sprite enable |
| $2002 | PPUSTATUS | Vblank, sprite 0 hit, overflow |
| $2003 | OAMADDR | OAM address |
| $2004 | OAMDATA | OAM data read/write |
| $2005 | PPUSCROLL | Scroll position (2 writes) |
| $2006 | PPUADDR | VRAM address (2 writes) |
| $2007 | PPUDATA | VRAM data read/write |

## References
- [W65C02S Datasheet](https://www.westerndesigncenter.com/wdc/documentation/w65c02s.pdf)
- [6502 Functional Tests](https://github.com/Klaus2m5/6502_65C02_functional_tests)
- [NES Dev Wiki](https://www.nesdev.org/wiki/Nesdev_Wiki)
- [NES CPU Memory Map](https://www.nesdev.org/wiki/CPU_memory_map)
- [NES PPU Memory Map](https://www.nesdev.org/wiki/PPU_memory_map)
