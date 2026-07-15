# AGENTS.md — ox6502

MOS 6502 / CMOS W65C02 CPU emulator. Single Rust crate, edition 2024.

## Build & Test

```bash
cargo build          # build
cargo test           # run all unit + integration tests
cargo run -- tests/roms/6502_functional_test.bin   # run Klaus Dormann 6502 test ROM
cargo run -- tests/roms/65C02_extended_opcodes_test.bin   # run 65C02 extended test
cargo run -- tests/roms/6502_functional_test.bin --debug   # interactive monitor
```

No dev-dependencies, no formatter/linter config, no CI. Just `cargo build` and `cargo test`.

Dependencies: `serde` + `serde_json` (for SST JSON test deserialization in `tests/sst_tests.rs`).

## Architecture

- `src/cpu.rs` — `Cpu<B: Bus>` struct, registers, fetch/read/write, stack ops
- `src/instructions.rs` — `step()` (giant match on opcode), `run()` (infinite loop), all instruction implementations including `adc`/`sbc`
- `src/opcodes.rs` — opcode constant definitions (244 lines of `pub const`)
- `src/addressing.rs` — all 13 addressing modes as methods on `Cpu<B>`, including NMOS JMP (ind) page boundary bug
- `src/bus/mod.rs` — `Bus` trait (read/write)
- `src/bus/simple.rs` — `SimpleBus` (64KB flat array)
- `src/monitor/mod.rs` — interactive debugger REPL (Monitor struct, run loop)
- `src/monitor/commands.rs` — command parsing, execution, all monitor commands
- `src/monitor/disass.rs` — disassembler (lookup table + disassemble_at)
- `tests/cpu_tests.rs` — integration tests with `TestBus` (also 64KB flat array)
- `src/main.rs` — CLI binary: loads ROM, runs CPU with trap detection, `--debug` for monitor

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
