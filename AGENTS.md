# AGENTS.md — ox6502

MOS 6502 / CMOS W65C02 CPU emulator. Single Rust crate, zero dependencies, edition 2024.

## Build & Test

```bash
cargo build          # build
cargo test           # run all unit + integration tests
cargo run -- tests/roms/6502_functional_test.bin   # run Klaus Dormann 6502 test ROM
```

No dev-dependencies, no formatter/linter config, no CI. Just `cargo build` and `cargo test`.

## Architecture

- `src/cpu.rs` — `Cpu<B: Bus>` struct, registers, fetch/read/write, stack ops
- `src/instructions.rs` — `step()` (giant match on opcode), `run()` (infinite loop), all instruction implementations including `adc`/`sbc`
- `src/opcodes.rs` — opcode constant definitions (244 lines of `pub const`)
- `src/addressing.rs` — all 13 addressing modes as methods on `Cpu<B>`
- `src/bus/mod.rs` — `Bus` trait (read/write)
- `src/bus/simple.rs` — `SimpleBus` (64KB flat array)
- `tests/cpu_tests.rs` — integration tests with `TestBus` (also 64KB flat array)
- `src/main.rs` — CLI binary: loads ROM, runs CPU with trap detection

## Key Quirks

- `Cpu::new(bus)` reads reset vector from `$FFFC-$FFFD` and sets the I flag. SP initialized to `$FD`.
- Tests set reset vector to `$0400` (Klaus Dormann test convention). `create_test_cpu()` in `tests/cpu_tests.rs` does this.
- `step()` returns cycle count as `u8`. `run()` is an **infinite loop** with no stopping — use the CLI `main.rs` loop or write your own.
- JSR pushes `PC - 1`; RTS adds 1 to popped address. This is correct 6502 behavior — do not "fix" it.
- Branch offsets are signed `i8` relative to PC **after** the branch byte fetch.
- Unknown opcodes `panic!` — the emulator will crash on unimplemented instructions.
- `Cpu` is generic over `Bus` — use `TestBus` in tests, `SimpleBus` for CLI. Don't couple CPU to a concrete bus.

## What's Implemented vs Missing

Implemented: full load/store, transfers, flags, jumps, inc/dec, compare, branches, logic, shifts/rotates, ADC/SBC (including BCD), BRK.

Missing (per TODO.md): WAI/STP (CMOS halt), page-crossing cycle penalties, branch-taken +1 cycle, precise cycle timing for all instructions.

## Adding Instructions

1. Add opcode constant(s) in `src/opcodes.rs`
2. Add match arm(s) in `step()` in `src/instructions.rs` — each arm calls an addressing mode, then an instruction helper, and returns a cycle count `u8`
3. Implement any new helper methods on `Cpu<B>` in `instructions.rs`
4. Add tests in `tests/cpu_tests.rs` using `create_test_cpu()` or `create_cpu_with_reset(addr)`

## Test Pattern

Tests use a `TestBus` struct (64KB memory) with `load_program(code, start_addr)`. The helper `create_test_cpu()` sets reset vector to `$0400`. Write test ROM bytes inline, call `cpu.step()`, assert registers/flags.
