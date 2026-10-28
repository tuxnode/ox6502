use std::cell::RefCell;

use capstone::prelude::*;

thread_local! {
    /// Capstone handles are not cheap to create, and the monitor disassembles
    /// on every step, so keep one per thread.
    static CAPSTONE: RefCell<Capstone> = RefCell::new(
        Capstone::new()
            .mos65xx()
            .mode(capstone::arch::mos65xx::ArchMode::Mos65xx65c02)
            .build()
            .expect("failed to initialize Capstone")
    );
}

/// Disassemble the single instruction at `addr`.
///
/// Returns the formatted instruction and its length in bytes. Capstone does
/// not know the NMOS undocumented opcodes; unknown bytes are shown as
/// `??? (XX)` and advance by one byte.
pub fn disassemble_at(addr: u16, mut read: impl FnMut(u16) -> u8) -> (String, u8) {
    // The longest 6502 instruction is three bytes.
    let bytes = [
        read(addr),
        read(addr.wrapping_add(1)),
        read(addr.wrapping_add(2)),
    ];

    CAPSTONE.with(|capstone| {
        let capstone = capstone.borrow();

        if let Ok(instructions) = capstone.disasm_count(&bytes, addr as u64, 1)
            && let Some(instruction) = instructions.iter().next()
        {
            let mnemonic = instruction.mnemonic().unwrap_or("???").to_ascii_uppercase();
            let operands = instruction
                .op_str()
                .unwrap_or("")
                .to_ascii_uppercase()
                .replace("0X", "$")
                .replace(", ", ",");
            let text = if operands.is_empty() {
                mnemonic
            } else {
                format!("{mnemonic} {operands}")
            };
            let len = instruction.bytes().len().clamp(1, 3) as u8;

            return (text, len);
        }

        (format!("??? ({:02X})", bytes[0]), 1)
    })
}

#[cfg(test)]
mod tests {
    use super::disassemble_at;

    fn disassemble(code: &[u8]) -> (String, u8) {
        disassemble_at(0x8000, |addr| {
            code.get(addr.wrapping_sub(0x8000) as usize)
                .copied()
                .unwrap_or(0)
        })
    }

    #[test]
    fn disassembles_immediate() {
        assert_eq!(disassemble(&[0xA9, 0x10]), ("LDA #$10".into(), 2));
    }

    #[test]
    fn disassembles_absolute_indexed() {
        assert_eq!(disassemble(&[0xBD, 0x34, 0x12]), ("LDA $1234,X".into(), 3));
    }

    #[test]
    fn disassembles_indirect_indexed() {
        assert_eq!(disassemble(&[0xB1, 0xA2]), ("LDA ($A2),Y".into(), 2));
    }

    #[test]
    fn disassembles_indirect_jump() {
        assert_eq!(disassemble(&[0x6C, 0x34, 0x12]), ("JMP ($1234)".into(), 3));
    }

    #[test]
    fn disassembles_accumulator_shift() {
        assert_eq!(disassemble(&[0x0A]), ("ASL A".into(), 1));
    }

    #[test]
    fn disassembles_relative_branch_target() {
        assert_eq!(disassemble(&[0xF0, 0xFE]), ("BEQ $8000".into(), 2));
    }

    #[test]
    fn disassembles_65c02_stz() {
        assert_eq!(disassemble(&[0x9C, 0x34, 0x12]), ("STZ $1234".into(), 3));
    }
}
