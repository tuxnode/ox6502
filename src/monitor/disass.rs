use crate::opcodes;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Implied,
    Accumulator,
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    Indirect,
    ZpIndirect,
    ZpIndirectX,
    ZpIndirectY,
    Relative,
}

pub fn lookup(opcode: u8) -> Option<(&'static str, Mode, u8)> {
    match opcode {
        // NOP
        opcodes::NOP => Some(("NOP", Mode::Implied, 2)),

        // Stack
        opcodes::PHA => Some(("PHA", Mode::Implied, 3)),
        opcodes::PHP => Some(("PHP", Mode::Implied, 3)),
        opcodes::PHX => Some(("PHX", Mode::Implied, 3)),
        opcodes::PHY => Some(("PHY", Mode::Implied, 3)),
        opcodes::PLA => Some(("PLA", Mode::Implied, 4)),
        opcodes::PLP => Some(("PLP", Mode::Implied, 4)),
        opcodes::PLX => Some(("PLX", Mode::Implied, 4)),
        opcodes::PLY => Some(("PLY", Mode::Implied, 4)),

        // Transfer
        opcodes::TAX => Some(("TAX", Mode::Implied, 2)),
        opcodes::TAY => Some(("TAY", Mode::Implied, 2)),
        opcodes::TXA => Some(("TXA", Mode::Implied, 2)),
        opcodes::TYA => Some(("TYA", Mode::Implied, 2)),
        opcodes::TSX => Some(("TSX", Mode::Implied, 2)),
        opcodes::TXS => Some(("TXS", Mode::Implied, 2)),

        // Flags
        opcodes::CLC => Some(("CLC", Mode::Implied, 2)),
        opcodes::CLD => Some(("CLD", Mode::Implied, 2)),
        opcodes::CLI => Some(("CLI", Mode::Implied, 2)),
        opcodes::CLV => Some(("CLV", Mode::Implied, 2)),
        opcodes::SEC => Some(("SEC", Mode::Implied, 2)),
        opcodes::SED => Some(("SED", Mode::Implied, 2)),
        opcodes::SEI => Some(("SEI", Mode::Implied, 2)),

        // System
        opcodes::BRK => Some(("BRK", Mode::Implied, 7)),
        opcodes::WAI => Some(("WAI", Mode::Implied, 2)),
        opcodes::STP => Some(("STP", Mode::Implied, 2)),

        // Accumulator
        opcodes::ASL_A => Some(("ASL", Mode::Accumulator, 2)),
        opcodes::LSR_A => Some(("LSR", Mode::Accumulator, 2)),
        opcodes::ROL_A => Some(("ROL", Mode::Accumulator, 2)),
        opcodes::ROR_A => Some(("ROR", Mode::Accumulator, 2)),
        opcodes::INC_A => Some(("INC", Mode::Accumulator, 2)),
        opcodes::DEC_A => Some(("DEC", Mode::Accumulator, 2)),

        // LDA
        opcodes::LDA_IMM => Some(("LDA", Mode::Immediate, 2)),
        opcodes::LDA_ZP => Some(("LDA", Mode::ZeroPage, 3)),
        opcodes::LDA_ZPX => Some(("LDA", Mode::ZeroPageX, 4)),
        opcodes::LDA_ABS => Some(("LDA", Mode::Absolute, 4)),
        opcodes::LDA_ABSX => Some(("LDA", Mode::AbsoluteX, 4)),
        opcodes::LDA_ABSY => Some(("LDA", Mode::AbsoluteY, 4)),
        opcodes::LDA_ZPI => Some(("LDA", Mode::ZpIndirect, 5)),
        opcodes::LDA_ZPXI => Some(("LDA", Mode::ZpIndirectX, 6)),
        opcodes::LDA_AIY => Some(("LDA", Mode::ZpIndirectY, 5)),

        // LDX
        opcodes::LDX_IMM => Some(("LDX", Mode::Immediate, 2)),
        opcodes::LDX_ZP => Some(("LDX", Mode::ZeroPage, 3)),
        opcodes::LDX_ZPY => Some(("LDX", Mode::ZeroPageY, 4)),
        opcodes::LDX_ABS => Some(("LDX", Mode::Absolute, 4)),
        opcodes::LDX_ABSY => Some(("LDX", Mode::AbsoluteY, 4)),

        // LDY
        opcodes::LDY_IMM => Some(("LDY", Mode::Immediate, 2)),
        opcodes::LDY_ZP => Some(("LDY", Mode::ZeroPage, 3)),
        opcodes::LDY_ZPX => Some(("LDY", Mode::ZeroPageX, 4)),
        opcodes::LDY_ABS => Some(("LDY", Mode::Absolute, 4)),
        opcodes::LDY_ABSX => Some(("LDY", Mode::AbsoluteX, 4)),

        // STA
        opcodes::STA_ZP => Some(("STA", Mode::ZeroPage, 3)),
        opcodes::STA_ZPX => Some(("STA", Mode::ZeroPageX, 4)),
        opcodes::STA_ABS => Some(("STA", Mode::Absolute, 4)),
        opcodes::STA_ABSX => Some(("STA", Mode::AbsoluteX, 5)),
        opcodes::STA_ABSY => Some(("STA", Mode::AbsoluteY, 5)),
        opcodes::STA_ZPI => Some(("STA", Mode::ZpIndirect, 5)),
        opcodes::STA_ZPXI => Some(("STA", Mode::ZpIndirectX, 6)),
        opcodes::STA_AIY => Some(("STA", Mode::ZpIndirectY, 6)),

        // STX
        opcodes::STX_ZP => Some(("STX", Mode::ZeroPage, 3)),
        opcodes::STX_ZPY => Some(("STX", Mode::ZeroPageY, 4)),
        opcodes::STX_ABS => Some(("STX", Mode::Absolute, 4)),

        // STY
        opcodes::STY_ZP => Some(("STY", Mode::ZeroPage, 3)),
        opcodes::STY_ZPX => Some(("STY", Mode::ZeroPageX, 4)),
        opcodes::STY_ABS => Some(("STY", Mode::Absolute, 4)),

        // STZ
        opcodes::STZ_ZP => Some(("STZ", Mode::ZeroPage, 3)),
        opcodes::STZ_ZPX => Some(("STZ", Mode::ZeroPageX, 4)),
        opcodes::STZ_ABS => Some(("STZ", Mode::Absolute, 4)),
        opcodes::STZ_ABSX => Some(("STZ", Mode::AbsoluteX, 5)),

        // ADC
        opcodes::ADC_IMM => Some(("ADC", Mode::Immediate, 2)),
        opcodes::ADC_ZP => Some(("ADC", Mode::ZeroPage, 3)),
        opcodes::ADC_ZPX => Some(("ADC", Mode::ZeroPageX, 4)),
        opcodes::ADC_ABS => Some(("ADC", Mode::Absolute, 4)),
        opcodes::ADC_ABSX => Some(("ADC", Mode::AbsoluteX, 4)),
        opcodes::ADC_ABSY => Some(("ADC", Mode::AbsoluteY, 4)),
        opcodes::ADC_ZPI => Some(("ADC", Mode::ZpIndirect, 5)),
        opcodes::ADC_ZPXI => Some(("ADC", Mode::ZpIndirectX, 6)),
        opcodes::ADC_AIY => Some(("ADC", Mode::ZpIndirectY, 5)),

        // SBC
        opcodes::SBC_IMM => Some(("SBC", Mode::Immediate, 2)),
        opcodes::SBC_ZP => Some(("SBC", Mode::ZeroPage, 3)),
        opcodes::SBC_ZPX => Some(("SBC", Mode::ZeroPageX, 4)),
        opcodes::SBC_ABS => Some(("SBC", Mode::Absolute, 4)),
        opcodes::SBC_ABSX => Some(("SBC", Mode::AbsoluteX, 4)),
        opcodes::SBC_ABSY => Some(("SBC", Mode::AbsoluteY, 4)),
        opcodes::SBC_ZPI => Some(("SBC", Mode::ZpIndirect, 5)),
        opcodes::SBC_ZPXI => Some(("SBC", Mode::ZpIndirectX, 6)),
        opcodes::SBC_AIY => Some(("SBC", Mode::ZpIndirectY, 5)),

        // AND
        opcodes::AND_IMM => Some(("AND", Mode::Immediate, 2)),
        opcodes::AND_ZP => Some(("AND", Mode::ZeroPage, 3)),
        opcodes::AND_ZPX => Some(("AND", Mode::ZeroPageX, 4)),
        opcodes::AND_ABS => Some(("AND", Mode::Absolute, 4)),
        opcodes::AND_ABSX => Some(("AND", Mode::AbsoluteX, 4)),
        opcodes::AND_ABSY => Some(("AND", Mode::AbsoluteY, 4)),
        opcodes::AND_ZPI => Some(("AND", Mode::ZpIndirect, 5)),
        opcodes::AND_ZPXI => Some(("AND", Mode::ZpIndirectX, 6)),
        opcodes::AND_AIY => Some(("AND", Mode::ZpIndirectY, 5)),

        // ORA
        opcodes::ORA_IMM => Some(("ORA", Mode::Immediate, 2)),
        opcodes::ORA_ZP => Some(("ORA", Mode::ZeroPage, 3)),
        opcodes::ORA_ZPX => Some(("ORA", Mode::ZeroPageX, 4)),
        opcodes::ORA_ABS => Some(("ORA", Mode::Absolute, 4)),
        opcodes::ORA_ABSX => Some(("ORA", Mode::AbsoluteX, 4)),
        opcodes::ORA_ABSY => Some(("ORA", Mode::AbsoluteY, 4)),
        opcodes::ORA_ZPI => Some(("ORA", Mode::ZpIndirect, 5)),
        opcodes::ORA_ZPXI => Some(("ORA", Mode::ZpIndirectX, 6)),
        opcodes::ORA_AIY => Some(("ORA", Mode::ZpIndirectY, 5)),

        // EOR
        opcodes::EOR_IMM => Some(("EOR", Mode::Immediate, 2)),
        opcodes::EOR_ZP => Some(("EOR", Mode::ZeroPage, 3)),
        opcodes::EOR_ZPX => Some(("EOR", Mode::ZeroPageX, 4)),
        opcodes::EOR_ABS => Some(("EOR", Mode::Absolute, 4)),
        opcodes::EOR_ABSX => Some(("EOR", Mode::AbsoluteX, 4)),
        opcodes::EOR_ABSY => Some(("EOR", Mode::AbsoluteY, 4)),
        opcodes::EOR_ZPI => Some(("EOR", Mode::ZpIndirect, 5)),
        opcodes::EOR_ZPXI => Some(("EOR", Mode::ZpIndirectX, 6)),
        opcodes::EOR_AIY => Some(("EOR", Mode::ZpIndirectY, 5)),

        // CMP
        opcodes::CMP_IMM => Some(("CMP", Mode::Immediate, 2)),
        opcodes::CMP_ZP => Some(("CMP", Mode::ZeroPage, 3)),
        opcodes::CMP_ZPX => Some(("CMP", Mode::ZeroPageX, 4)),
        opcodes::CMP_ABS => Some(("CMP", Mode::Absolute, 4)),
        opcodes::CMP_ABSX => Some(("CMP", Mode::AbsoluteX, 4)),
        opcodes::CMP_ABSY => Some(("CMP", Mode::AbsoluteY, 4)),
        opcodes::CMP_ZPI => Some(("CMP", Mode::ZpIndirect, 5)),
        opcodes::CMP_ZPXI => Some(("CMP", Mode::ZpIndirectX, 6)),
        opcodes::CMP_AIY => Some(("CMP", Mode::ZpIndirectY, 5)),

        // CPX
        opcodes::CPX_IMM => Some(("CPX", Mode::Immediate, 2)),
        opcodes::CPX_ZP => Some(("CPX", Mode::ZeroPage, 3)),
        opcodes::CPX_ABS => Some(("CPX", Mode::Absolute, 4)),

        // CPY
        opcodes::CPY_IMM => Some(("CPY", Mode::Immediate, 2)),
        opcodes::CPY_ZP => Some(("CPY", Mode::ZeroPage, 3)),
        opcodes::CPY_ABS => Some(("CPY", Mode::Absolute, 4)),

        // INC
        opcodes::INC_ZP => Some(("INC", Mode::ZeroPage, 5)),
        opcodes::INC_ZPX => Some(("INC", Mode::ZeroPageX, 6)),
        opcodes::INC_ABS => Some(("INC", Mode::Absolute, 6)),
        opcodes::INC_ABSX => Some(("INC", Mode::AbsoluteX, 7)),

        // DEC
        opcodes::DEC_ZP => Some(("DEC", Mode::ZeroPage, 5)),
        opcodes::DEC_ZPX => Some(("DEC", Mode::ZeroPageX, 6)),
        opcodes::DEC_ABS => Some(("DEC", Mode::Absolute, 6)),
        opcodes::DEC_ABSX => Some(("DEC", Mode::AbsoluteX, 7)),

        // INX, INY, DEX, DEY
        opcodes::INX => Some(("INX", Mode::Implied, 2)),
        opcodes::INY => Some(("INY", Mode::Implied, 2)),
        opcodes::DEX => Some(("DEX", Mode::Implied, 2)),
        opcodes::DEY => Some(("DEY", Mode::Implied, 2)),

        // ASL
        opcodes::ASL_ZP => Some(("ASL", Mode::ZeroPage, 5)),
        opcodes::ASL_ZPX => Some(("ASL", Mode::ZeroPageX, 6)),
        opcodes::ASL_ABS => Some(("ASL", Mode::Absolute, 6)),
        opcodes::ASL_ABSX => Some(("ASL", Mode::AbsoluteX, 7)),

        // LSR
        opcodes::LSR_ZP => Some(("LSR", Mode::ZeroPage, 5)),
        opcodes::LSR_ZPX => Some(("LSR", Mode::ZeroPageX, 6)),
        opcodes::LSR_ABS => Some(("LSR", Mode::Absolute, 6)),
        opcodes::LSR_ABSX => Some(("LSR", Mode::AbsoluteX, 7)),

        // ROL
        opcodes::ROL_ZP => Some(("ROL", Mode::ZeroPage, 5)),
        opcodes::ROL_ZPX => Some(("ROL", Mode::ZeroPageX, 6)),
        opcodes::ROL_ABS => Some(("ROL", Mode::Absolute, 6)),
        opcodes::ROL_ABSX => Some(("ROL", Mode::AbsoluteX, 7)),

        // ROR
        opcodes::ROR_ZP => Some(("ROR", Mode::ZeroPage, 5)),
        opcodes::ROR_ZPX => Some(("ROR", Mode::ZeroPageX, 6)),
        opcodes::ROR_ABS => Some(("ROR", Mode::Absolute, 6)),
        opcodes::ROR_ABSX => Some(("ROR", Mode::AbsoluteX, 7)),

        // BIT
        opcodes::BIT_IMM => Some(("BIT", Mode::Immediate, 2)),
        opcodes::BIT_ZP => Some(("BIT", Mode::ZeroPage, 3)),
        opcodes::BIT_ZPX => Some(("BIT", Mode::ZeroPageX, 4)),
        opcodes::BIT_ABS => Some(("BIT", Mode::Absolute, 4)),
        opcodes::BIT_ABSX => Some(("BIT", Mode::AbsoluteX, 4)),

        // TRB
        opcodes::TRB_ZP => Some(("TRB", Mode::ZeroPage, 5)),
        opcodes::TRB_ABS => Some(("TRB", Mode::Absolute, 6)),

        // TSB
        opcodes::TSB_ZP => Some(("TSB", Mode::ZeroPage, 5)),
        opcodes::TSB_ABS => Some(("TSB", Mode::Absolute, 6)),

        // Branch
        opcodes::BCC => Some(("BCC", Mode::Relative, 2)),
        opcodes::BCS => Some(("BCS", Mode::Relative, 2)),
        opcodes::BEQ => Some(("BEQ", Mode::Relative, 2)),
        opcodes::BNE => Some(("BNE", Mode::Relative, 2)),
        opcodes::BMI => Some(("BMI", Mode::Relative, 2)),
        opcodes::BPL => Some(("BPL", Mode::Relative, 2)),
        opcodes::BVC => Some(("BVC", Mode::Relative, 2)),
        opcodes::BVS => Some(("BVS", Mode::Relative, 2)),
        opcodes::BRA => Some(("BRA", Mode::Relative, 2)),

        // Jump
        opcodes::JMP_ABS => Some(("JMP", Mode::Absolute, 3)),
        opcodes::JMP_IND => Some(("JMP", Mode::Indirect, 6)),
        opcodes::JMP_INDX => Some(("JMP", Mode::ZpIndirectX, 6)),
        opcodes::JSR => Some(("JSR", Mode::Absolute, 6)),
        opcodes::RTS => Some(("RTS", Mode::Implied, 6)),
        opcodes::RTI => Some(("RTI", Mode::Implied, 6)),

        _ => None,
    }
}

pub fn disassemble_at(addr: u16, mut read: impl FnMut(u16) -> u8) -> (String, u8) {
    let opcode = read(addr);

    let (mnem, mode, _) = match lookup(opcode) {
        Some(info) => info,
        None => return (format!("??? ({:02X})", opcode), 1),
    };

    match mode {
        Mode::Implied => (mnem.to_string(), 1),
        Mode::Accumulator => (format!("{} A", mnem), 1),
        Mode::Immediate => {
            let b = read(addr.wrapping_add(1));
            (format!("{} #{:02X}", mnem, b), 2)
        }
        Mode::ZeroPage => {
            let b = read(addr.wrapping_add(1));
            (format!("{} ${:02X}", mnem, b), 2)
        }
        Mode::ZeroPageX => {
            let b = read(addr.wrapping_add(1));
            (format!("{} ${:02X},X", mnem, b), 2)
        }
        Mode::ZeroPageY => {
            let b = read(addr.wrapping_add(1));
            (format!("{} ${:02X},Y", mnem, b), 2)
        }
        Mode::Absolute => {
            let lo = read(addr.wrapping_add(1));
            let hi = read(addr.wrapping_add(2));
            let addr16 = (hi as u16) << 8 | lo as u16;
            (format!("{} ${:04X}", mnem, addr16), 3)
        }
        Mode::AbsoluteX => {
            let lo = read(addr.wrapping_add(1));
            let hi = read(addr.wrapping_add(2));
            let addr16 = (hi as u16) << 8 | lo as u16;
            (format!("{} ${:04X},X", mnem, addr16), 3)
        }
        Mode::AbsoluteY => {
            let lo = read(addr.wrapping_add(1));
            let hi = read(addr.wrapping_add(2));
            let addr16 = (hi as u16) << 8 | lo as u16;
            (format!("{} ${:04X},Y", mnem, addr16), 3)
        }
        Mode::Indirect => {
            let lo = read(addr.wrapping_add(1));
            let hi = read(addr.wrapping_add(2));
            let addr16 = (hi as u16) << 8 | lo as u16;
            (format!("{} (${:04X})", mnem, addr16), 3)
        }
        Mode::ZpIndirect => {
            let b = read(addr.wrapping_add(1));
            (format!("{} (${:02X})", mnem, b), 2)
        }
        Mode::ZpIndirectX => {
            let b = read(addr.wrapping_add(1));
            (format!("{} (${:02X},X)", mnem, b), 2)
        }
        Mode::ZpIndirectY => {
            let b = read(addr.wrapping_add(1));
            (format!("{} (${:02X}),Y", mnem, b), 2)
        }
        Mode::Relative => {
            let offset = read(addr.wrapping_add(1)) as i8;
            let target = (addr as i16 + 2 + offset as i16) as u16;
            (format!("{} ${:04X}", mnem, target), 2)
        }
    }
}
