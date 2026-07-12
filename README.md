# ox6502
A MOS 6502 / CMOS W65C02 CPU emulator written in Rust.

![MOS 6502](https://www.masswerk.at/6502/assets/MOS_6502AD_4585.jpg)

## Features
- Full 6502/65C02 instruction set support
- All 13 addressing modes
- Bus abstraction for flexible memory mapping
- Integration testing with [Klaus2m5 functional tests](https://github.com/Klaus2m5/6502_65C02_functional_tests)
## Project Structure
```
ox6502/
├── src/
│   ├── lib.rs          # Library entry point
│   ├── main.rs         # Binary entry point
│   ├── cpu.rs          # CPU registers and core operations
│   ├── instructions.rs # Instruction implementations
│   ├── addressing.rs   # Addressing modes
│   ├── opcodes.rs      # Opcode constants
│   └── bus/
│       ├── mod.rs      # Bus trait definition
│       └── simple.rs   # SimpleBus (64KB memory)
└── tests/
    └── cpu_tests.rs    # Integration tests
```
## Build
```bash
cargo build
```
## Test
```bash
cargo test
```
## Run with Test ROM
```bash
cargo run -- tests/roms/6502_functional_test.bin
cargo run -- tests/roms/65C02_extended_opcodes_test.bin
```
## References
- [W65C02S Datasheet](https://www.westerndesigncenter.com/wdc/documentation/w65c02s.pdf)
- [W65C02S Programming Manual](https://www.westerndesigncenter.com/wdc/documentation/w65c02-programming-manual.pdf)
- [6502 Functional Tests](https://github.com/Klaus2m5/6502_65C02_functional_tests)
- [6502 Instructions Set](https://www.masswerk.at/6502/6502_instruction_set.html)
