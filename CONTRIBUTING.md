# Contributing to ox6502

MOS 6502 / CMOS W65C02 CPU emulator with NES system emulation, written in Rust.

## Getting Started

### Prerequisites

- Rust (edition 2024)

The real-time NES display uses Notan (pure-Rust windowing/rendering) and cpal
for audio streaming. On Linux, cpal needs the ALSA development headers:

```bash
sudo apt-get install libasound2-dev
```

No SDL2 installation is required.

### Build & Test

```bash
cargo build          # build
cargo test           # run all unit + integration tests
cargo run -- tests/roms/6502_functional_test.bin   # run Klaus Dormann 6502 test ROM
cargo run -- tests/roms/65C02_extended_opcodes_test.bin   # run 65C02 extended test
cargo run -- tests/roms/6502_functional_test.bin --debug   # interactive monitor
cargo run --features frontend --bin nes_notan -- <game.nes>   # NES real-time display (Notan)
cargo run --bin nes_render -- <game.nes>         # NES offline frame → PPM
```

No dev-dependencies, no formatter/linter config. CI (`.github/workflows/ci.yml`) runs on every push and pull request: `cargo fmt --check`, `cargo check --all-targets`, and the project's own tests. To reproduce locally:

```bash
cargo fmt --all -- --check
cargo check --all-targets
cargo test --lib --test cpu_tests   # excludes the SST tests, which need the ProcessorTests submodule
```

## Architecture

- `src/cpu.rs` — `Cpu<B: Bus>` struct, registers, fetch/read/write, stack ops
- `src/instructions.rs` — `step()` (giant match on opcode), `run()` (infinite loop), all instruction implementations including `adc`/`sbc`
- `src/opcodes.rs` — opcode constant definitions (244 lines of `pub const`)
- `src/addressing.rs` — all 13 addressing modes as methods on `Cpu<B>`, including NMOS JMP (ind) page boundary bug
- `src/bus/mod.rs` — `Bus` trait (cpu_read/write, ppu_read/write, tick)
- `src/bus/simple.rs` — `SimpleBus` (64KB flat array, for CPU test ROMs)
- `src/bus/nes.rs` — `NesBus`: NES address routing, PPU/APU I/O, OAM DMA
- `src/nes/cartridge.rs` — iNES ROM header parser (magic, mapper, mirroring, PRG/CHR banks)
- `src/nes/ppu.rs` — PPU registers ($2000-$2007), loopy address system, tick timing, background+sprite rendering, frame buffer
- `src/nes/palette.rs` — NTSC 64-color system palette table
- `src/nes/mapper/nrom.rs` — NROM (mapper 0) stub
- `src/monitor/mod.rs` — interactive debugger REPL (Monitor struct, run loop)
- `src/monitor/commands.rs` — command parsing, execution, all monitor commands
- `src/monitor/disass.rs` — disassembler (lookup table + disassemble_at)
- `tests/cpu_tests.rs` — integration tests with `TestBus` (also 64KB flat array)
- `src/main.rs` — CLI binary: loads ROM, runs CPU with trap detection, `--debug` for monitor
- `src/bin/nes_render.rs` — offline NES frame renderer (loads .nes, runs N frames, writes PPM)
- `src/bin/nes_notan.rs` — real-time NES display window (Notan + cpal, `frontend` feature)

## Key Quirks

- `Cpu::new(bus)` reads reset vector from `$FFFC-$FFFD` and sets the I flag. SP initialized to `$FD`.
- Tests set reset vector to `$0400` (Klaus Dormann test convention). `create_test_cpu()` in `tests/cpu_tests.rs` does this.
- `step()` returns cycle count as `u8`. `run()` is an **infinite loop** with no stopping — use the CLI `main.rs` loop or write your own.
- JSR pushes `PC - 1`; RTS adds 1 to popped address. This is correct 6502 behavior — do not "fix" it.
- Branch offsets are signed `i8` relative to PC **after** the branch byte fetch.
- **NES 6502 (NMOS) specific**: D flag has no effect on ADC/SBC (BCD disabled). All CMOS opcodes (STZ, BRA, PHX/PHY/PLX/PLY, INC A, DEC A) are NOPs on NMOS.
- **KIL opcodes** (0x02, 0x12, etc.) lock the CPU — PC does not advance.
- **JMP (ind) page boundary bug**: On NMOS, if the pointer crosses a page boundary, the high byte is read from the same page, not ptr+1.
- `Cpu` is generic over `Bus` — use `TestBus` in tests, `SimpleBus` for CLI. Don't couple CPU to a concrete bus.
- **PPU timing**: 1 CPU cycle = 3 PPU dots. `Bus::tick(cpu_cycles)` advances PPU by `cpu_cycles * 3` dots.
- **NMI is non-maskable**: I flag does not block NMI (unlike IRQ). `tick()` signals NMI via `TickResult.nmi`.
- **Background rendering**: Decodes nametable → attribute table → pattern table, writes 256×240 RGB frame buffer.
- **Sprite rendering**: OAM decode, horizontal/vertical flip, priority-behind-background (attr bit 5), sprite palette at `palette[0x11..0x1F]`.

## What's Implemented vs Missing

Implemented: full load/store, transfers, flags, jumps, inc/dec, compare, branches, logic, shifts/rotates, ADC/SBC (binary only, NMOS), BRK, JMP (ind) page bug, 247/256 NMOS illegal opcodes.

Missing: 9 unstable illegal opcodes (XAA, AHX, TAS, SHY, SHX, LAX#, AXS, SBC#), page-crossing cycle penalties, branch-taken +1 cycle, precise cycle timing.

## SST Test Results

**247/256 opcodes pass** (96.5%). Remaining 9 are unstable opcodes with behavior that varies by CPU revision.

SST tests are in `tests/sst_tests.rs` and use JSON fixtures from `tests/sst_tests/nes6502/v1/`. Each opcode has 10,000 randomized test cases.

## Monitor Usage

```bash
cargo run -- tests/roms/6502_functional_test.bin --debug
```

Commands: `s`/step, `c`/continue, `r`/regs, `d`/disassemble, `m`/memory, `b`/breakpoint, `bc`/breakpoint clear, `bl`/breakpoint list, `t`/trace, `h`/help, `q`/quit. Press Enter to repeat last command.

## Adding Instructions

1. Add opcode constant(s) in `src/opcodes.rs`
2. Add match arm(s) in `step()` in `src/instructions.rs` — each arm calls an addressing mode, then an instruction helper, and returns a cycle count `u8`
3. Implement any new helper methods on `Cpu<B>` in `instructions.rs`
4. Add tests in `tests/cpu_tests.rs` using `create_test_cpu()` or `create_cpu_with_reset(addr)`

## Test Pattern

Tests use a `TestBus` struct (64KB memory) with `load_program(code, start_addr)`. The helper `create_test_cpu()` sets reset vector to `$0400`. Write test ROM bytes inline, call `cpu.step()`, assert registers/flags.

To run a single SST test: `cargo test test_sst_00` (or any hex opcode like `test_sst_ff`).

## Code Style

- Follow Rust conventions (snake_case for functions/variables, CamelCase for types)
- Keep functions focused and small
- Use `pub` sparingly — prefer module-internal visibility
- Avoid unnecessary comments — code should be self-documenting
- Use `unwrap()` freely in tests, but handle errors properly in library code

## Submitting Changes

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Commit Messages

- Use present tense ("Add feature" not "Added feature")
- Use imperative mood ("Move cursor" not "Moves cursor")
- Keep the first line under 72 characters
- Reference issues and pull requests where appropriate

## Reporting Issues

- Use the GitHub issue tracker
- Include steps to reproduce
- Include your Rust version (`rustc --version`)
- Include relevant test ROMs or ROM files if applicable

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
