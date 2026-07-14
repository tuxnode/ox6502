// 65C02 Instructions Implementation
//
// References:
// - W65C02S Datasheet: https://www.westerndesigncenter.com/wdc/documentation/w65c02s.pdf
// - W65C02S Programming Manual: https://www.westerndesigncenter.com/wdc/documentation/w65c02-programming-manual.pdf
// - Obelisk 6502 Reference: https://www.obelisk.me.uk/6502/reference.html
// - 6502.org Tutorials: https://www.6502.org/tutorials/

/*
Status Registers Layout:
Bit:  7  6  5  4  3  2  1  0
      N  V  -  B  D  I  Z  C
*/
pub const FLAG_C: u8 = 1 << 0; // 0x01
pub const FLAG_Z: u8 = 1 << 1; // 0x02
pub const FLAG_I: u8 = 1 << 2; // 0x04
pub const FLAG_D: u8 = 1 << 3; // 0x08
pub const FLAG_B: u8 = 1 << 4; // 0x10
pub const FLAG_V: u8 = 1 << 6; // 0x40
pub const FLAG_N: u8 = 1 << 7; // 0x80

use crate::bus::Bus;
use crate::cpu::Cpu;
use crate::opcodes;

impl<B: Bus> Cpu<B> {
    pub fn step(&mut self) -> u8 {
        let opcode = self.fetch();
        match opcode {
            opcodes::NOP => 2,

            // Stack
            opcodes::PHA => { self.push(self.a); 3 }
            opcodes::PHP => { self.push(self.status | FLAG_B | 0x20); 3 }
            opcodes::PHX => { 2 } // NMOS: NOP
            opcodes::PHY => { 2 } // NMOS: NOP
            opcodes::PLA => { self.a = self.pop(); self.update_nz(self.a); 4 }
            opcodes::PLP => { self.status = (self.pop() & 0xEF) | 0x20; 4 }
            opcodes::PLX => { 2 } // NMOS: NOP
            opcodes::PLY => { 2 } // NMOS: NOP

            // Transfer
            opcodes::TAX => { self.x = self.a; self.update_nz(self.x); 2 }
            opcodes::TAY => { self.y = self.a; self.update_nz(self.y); 2 }
            opcodes::TXA => { self.a = self.x; self.update_nz(self.a); 2 }
            opcodes::TYA => { self.a = self.y; self.update_nz(self.a); 2 }
            opcodes::TSX => { self.x = self.sp; self.update_nz(self.x); 2 }
            opcodes::TXS => { self.sp = self.x; 2 }

            // Load
            opcodes::LDA_IMM => { let a = self.immediate(); self.lda(a); 2 }
            opcodes::LDA_ZP => { let a = self.zeropage(); self.lda(a); 3 }
            opcodes::LDA_ZPX => { let a = self.zeropage_x(); self.lda(a); 4 }
            opcodes::LDA_ABS => { let a = self.absolute(); self.lda(a); 4 }
            opcodes::LDA_ABSX => { let a = self.absolute_x(); self.lda(a); 4 }
            opcodes::LDA_ABSY => { let a = self.absolute_y(); self.lda(a); 4 }
            // KIL opcodes (conflict with CMOS ZPI on NMOS 6502)
            opcodes::ORA_ZPI | opcodes::AND_ZPI | opcodes::EOR_ZPI | opcodes::CMP_ZPI |
            opcodes::LDA_ZPI | opcodes::STA_ZPI | opcodes::ADC_ZPI | opcodes::SBC_ZPI => {
                self.pc = self.pc.wrapping_sub(1);
                2
            }
            opcodes::LDA_ZPXI => { let a = self.pre_indexed_x(); self.lda(a); 6 }
            opcodes::LDA_AIY => { let a = self.post_indexed_y(); self.lda(a); 5 }

            opcodes::LDX_IMM => { let a = self.immediate(); self.ldx(a); 2 }
            opcodes::LDX_ZP => { let a = self.zeropage(); self.ldx(a); 3 }
            opcodes::LDX_ZPY => { let a = self.zeropage_y(); self.ldx(a); 4 }
            opcodes::LDX_ABS => { let a = self.absolute(); self.ldx(a); 4 }
            opcodes::LDX_ABSY => { let a = self.absolute_y(); self.ldx(a); 4 }

            opcodes::LDY_IMM => { let a = self.immediate(); self.ldy(a); 2 }
            opcodes::LDY_ZP => { let a = self.zeropage(); self.ldy(a); 3 }
            opcodes::LDY_ZPX => { let a = self.zeropage_x(); self.ldy(a); 4 }
            opcodes::LDY_ABS => { let a = self.absolute(); self.ldy(a); 4 }
            opcodes::LDY_ABSX => { let a = self.absolute_x(); self.ldy(a); 4 }

            // Store
            opcodes::STA_ZP => { let a = self.zeropage(); self.sta(a); 3 }
            opcodes::STA_ZPX => { let a = self.zeropage_x(); self.sta(a); 4 }
            opcodes::STA_ABS => { let a = self.absolute(); self.sta(a); 4 }
            opcodes::STA_ABSX => { let a = self.absolute_x(); self.sta(a); 5 }
            opcodes::STA_ABSY => { let a = self.absolute_y(); self.sta(a); 5 }
            opcodes::STA_ZPXI => { let a = self.pre_indexed_x(); self.sta(a); 6 }
            opcodes::STA_AIY => { let a = self.post_indexed_y(); self.sta(a); 6 }

            opcodes::STX_ZP => { let a = self.zeropage(); self.stx(a); 3 }
            opcodes::STX_ZPY => { let a = self.zeropage_y(); self.stx(a); 4 }
            opcodes::STX_ABS => { let a = self.absolute(); self.stx(a); 4 }

            opcodes::STY_ZP => { let a = self.zeropage(); self.sty(a); 3 }
            opcodes::STY_ZPX => { let a = self.zeropage_x(); self.sty(a); 4 }
            opcodes::STY_ABS => { let a = self.absolute(); self.sty(a); 4 }

            opcodes::STZ_ZP => { self.zeropage(); 3 } // NMOS: DOP zp (NOP)
            opcodes::STZ_ZPX => { self.zeropage_x(); 4 } // NMOS: DOP zp,x (NOP)
            opcodes::STZ_ABS => { let addr = self.absolute_x(); let v = self.y & ((addr >> 8) as u8).wrapping_add(1); self.write(addr, v); 5 } // NMOS: SHY
            opcodes::STZ_ABSX => { let addr = self.absolute_y(); let v = self.x & ((addr >> 8) as u8).wrapping_add(1); self.write(addr, v); 5 } // NMOS: SHX

            // Flags
            opcodes::CLC => { self.set_flag(FLAG_C, false); 2 }
            opcodes::CLD => { self.set_flag(FLAG_D, false); 2 }
            opcodes::CLI => { self.set_flag(FLAG_I, false); 2 }
            opcodes::CLV => { self.set_flag(FLAG_V, false); 2 }
            opcodes::SEC => { self.set_flag(FLAG_C, true); 2 }
            opcodes::SED => { self.set_flag(FLAG_D, true); 2 }
            opcodes::SEI => { self.set_flag(FLAG_I, true); 2 }

            // Jump
            opcodes::JMP_ABS => { self.pc = self.absolute(); 3 }
            opcodes::JMP_IND => { self.pc = self.indirect(); 6 }
            opcodes::JMP_INDX => { self.pc = self.pc.wrapping_add(2); 4 } // NMOS: DOP (NOP)
            opcodes::JSR => { let addr = self.absolute(); let pc = self.pc.wrapping_sub(1); self.push((pc >> 8) as u8); self.push(pc as u8); self.pc = addr; 6 }
            opcodes::RTS => { let lo = self.pop() as u16; let hi = self.pop() as u16; self.pc = ((hi << 8) | lo).wrapping_add(1); 6 }
            opcodes::RTI => { self.status = (self.pop() & 0xEF) | 0x20; let lo = self.pop() as u16; let hi = self.pop() as u16; self.pc = (hi << 8) | lo; 6 }

            // Increment / Decrement
            opcodes::INC_A => { 2 } // NMOS: NOP
            opcodes::INC_ZP => { let a = self.zeropage(); self.inc_mem(a); 5 }
            opcodes::INC_ZPX => { let a = self.zeropage_x(); self.inc_mem(a); 6 }
            opcodes::INC_ABS => { let a = self.absolute(); self.inc_mem(a); 6 }
            opcodes::INC_ABSX => { let a = self.absolute_x(); self.inc_mem(a); 7 }
            opcodes::DEC_A => { 2 } // NMOS: NOP
            opcodes::DEC_ZP => { let a = self.zeropage(); self.dec_mem(a); 5 }
            opcodes::DEC_ZPX => { let a = self.zeropage_x(); self.dec_mem(a); 6 }
            opcodes::DEC_ABS => { let a = self.absolute(); self.dec_mem(a); 6 }
            opcodes::DEC_ABSX => { let a = self.absolute_x(); self.dec_mem(a); 7 }
            opcodes::INX => { self.x = self.x.wrapping_add(1); self.update_nz(self.x); 2 }
            opcodes::INY => { self.y = self.y.wrapping_add(1); self.update_nz(self.y); 2 }
            opcodes::DEX => { self.x = self.x.wrapping_sub(1); self.update_nz(self.x); 2 }
            opcodes::DEY => { self.y = self.y.wrapping_sub(1); self.update_nz(self.y); 2 }

            // Compare
            opcodes::CMP_IMM => { let a = self.immediate(); let v = self.read(a); self.compare(self.a, v); 2 }
            opcodes::CMP_ZP => { let a = self.zeropage(); let v = self.read(a); self.compare(self.a, v); 3 }
            opcodes::CMP_ZPX => { let a = self.zeropage_x(); let v = self.read(a); self.compare(self.a, v); 4 }
            opcodes::CMP_ABS => { let a = self.absolute(); let v = self.read(a); self.compare(self.a, v); 4 }
            opcodes::CMP_ABSX => { let a = self.absolute_x(); let v = self.read(a); self.compare(self.a, v); 4 }
            opcodes::CMP_ABSY => { let a = self.absolute_y(); let v = self.read(a); self.compare(self.a, v); 4 }
            opcodes::CMP_ZPXI => { let a = self.pre_indexed_x(); let v = self.read(a); self.compare(self.a, v); 6 }
            opcodes::CMP_AIY => { let a = self.post_indexed_y(); let v = self.read(a); self.compare(self.a, v); 5 }

            opcodes::CPX_IMM => { let a = self.immediate(); let v = self.read(a); self.compare(self.x, v); 2 }
            opcodes::CPX_ZP => { let a = self.zeropage(); let v = self.read(a); self.compare(self.x, v); 3 }
            opcodes::CPX_ABS => { let a = self.absolute(); let v = self.read(a); self.compare(self.x, v); 4 }

            opcodes::CPY_IMM => { let a = self.immediate(); let v = self.read(a); self.compare(self.y, v); 2 }
            opcodes::CPY_ZP => { let a = self.zeropage(); let v = self.read(a); self.compare(self.y, v); 3 }
            opcodes::CPY_ABS => { let a = self.absolute(); let v = self.read(a); self.compare(self.y, v); 4 }

            // Branch
            opcodes::BCC => { self.branch(!self.get_flag(FLAG_C)) }
            opcodes::BCS => { self.branch(self.get_flag(FLAG_C)) }
            opcodes::BEQ => { self.branch(self.get_flag(FLAG_Z)) }
            opcodes::BNE => { self.branch(!self.get_flag(FLAG_Z)) }
            opcodes::BMI => { self.branch(self.get_flag(FLAG_N)) }
            opcodes::BPL => { self.branch(!self.get_flag(FLAG_N)) }
            opcodes::BVS => { self.branch(self.get_flag(FLAG_V)) }
            opcodes::BVC => { self.branch(!self.get_flag(FLAG_V)) }
            opcodes::BRA => { self.pc = self.pc.wrapping_add(1); 2 } // NMOS: NOP 2-byte

            // System
            opcodes::BRK => {
                self.pc = self.pc.wrapping_add(1); // Skip padding byte
                self.push((self.pc >> 8) as u8);
                self.push(self.pc as u8);
                self.push(self.status | FLAG_B | 0x20);
                self.set_flag(FLAG_I, true);
                let lo = self.read(0xFFFE) as u16;
                let hi = self.read(0xFFFF) as u16;
                self.pc = (hi << 8) | lo;
                7
            }

            // Logic
            opcodes::AND_IMM => { let a = self.immediate(); let v = self.read(a); self.a &= v; self.update_nz(self.a); 2 }
            opcodes::AND_ZP => { let a = self.zeropage(); let v = self.read(a); self.a &= v; self.update_nz(self.a); 3 }
            opcodes::AND_ZPX => { let a = self.zeropage_x(); let v = self.read(a); self.a &= v; self.update_nz(self.a); 4 }
            opcodes::AND_ABS => { let a = self.absolute(); let v = self.read(a); self.a &= v; self.update_nz(self.a); 4 }
            opcodes::AND_ABSX => { let a = self.absolute_x(); let v = self.read(a); self.a &= v; self.update_nz(self.a); 4 }
            opcodes::AND_ABSY => { let a = self.absolute_y(); let v = self.read(a); self.a &= v; self.update_nz(self.a); 4 }
            opcodes::AND_ZPXI => { let a = self.pre_indexed_x(); let v = self.read(a); self.a &= v; self.update_nz(self.a); 6 }
            opcodes::AND_AIY => { let a = self.post_indexed_y(); let v = self.read(a); self.a &= v; self.update_nz(self.a); 5 }

            opcodes::ORA_IMM => { let a = self.immediate(); let v = self.read(a); self.a |= v; self.update_nz(self.a); 2 }
            opcodes::ORA_ZP => { let a = self.zeropage(); let v = self.read(a); self.a |= v; self.update_nz(self.a); 3 }
            opcodes::ORA_ZPX => { let a = self.zeropage_x(); let v = self.read(a); self.a |= v; self.update_nz(self.a); 4 }
            opcodes::ORA_ABS => { let a = self.absolute(); let v = self.read(a); self.a |= v; self.update_nz(self.a); 4 }
            opcodes::ORA_ABSX => { let a = self.absolute_x(); let v = self.read(a); self.a |= v; self.update_nz(self.a); 4 }
            opcodes::ORA_ABSY => { let a = self.absolute_y(); let v = self.read(a); self.a |= v; self.update_nz(self.a); 4 }
            opcodes::ORA_ZPXI => { let a = self.pre_indexed_x(); let v = self.read(a); self.a |= v; self.update_nz(self.a); 6 }
            opcodes::ORA_AIY => { let a = self.post_indexed_y(); let v = self.read(a); self.a |= v; self.update_nz(self.a); 5 }

            opcodes::EOR_IMM => { let a = self.immediate(); let v = self.read(a); self.a ^= v; self.update_nz(self.a); 2 }
            opcodes::EOR_ZP => { let a = self.zeropage(); let v = self.read(a); self.a ^= v; self.update_nz(self.a); 3 }
            opcodes::EOR_ZPX => { let a = self.zeropage_x(); let v = self.read(a); self.a ^= v; self.update_nz(self.a); 4 }
            opcodes::EOR_ABS => { let a = self.absolute(); let v = self.read(a); self.a ^= v; self.update_nz(self.a); 4 }
            opcodes::EOR_ABSX => { let a = self.absolute_x(); let v = self.read(a); self.a ^= v; self.update_nz(self.a); 4 }
            opcodes::EOR_ABSY => { let a = self.absolute_y(); let v = self.read(a); self.a ^= v; self.update_nz(self.a); 4 }
            opcodes::EOR_ZPXI => { let a = self.pre_indexed_x(); let v = self.read(a); self.a ^= v; self.update_nz(self.a); 6 }
            opcodes::EOR_AIY => { let a = self.post_indexed_y(); let v = self.read(a); self.a ^= v; self.update_nz(self.a); 5 }

            opcodes::BIT_IMM => { self.immediate(); 2 } // NMOS: DOP # (NOP)
            opcodes::BIT_ZP => { let a = self.zeropage(); let v = self.read(a); self.set_flag(FLAG_Z, (self.a & v) == 0); self.set_flag(FLAG_N, v & 0x80 != 0); self.set_flag(FLAG_V, v & 0x40 != 0); 3 }
            opcodes::BIT_ZPX => { self.zeropage_x(); 4 } // NMOS: DOP zp,x (NOP)
            opcodes::BIT_ABS => { let a = self.absolute(); let v = self.read(a); self.set_flag(FLAG_Z, (self.a & v) == 0); self.set_flag(FLAG_N, v & 0x80 != 0); self.set_flag(FLAG_V, v & 0x40 != 0); 4 }
            opcodes::BIT_ABSX => { self.absolute_x(); 4 } // NMOS: DOP abs,x (NOP)

            opcodes::TRB_ZP => { self.zeropage(); 3 } // NMOS: DOP zp (NOP)
            opcodes::TRB_ABS => { self.absolute(); 4 } // NMOS: DOP abs (NOP)

            opcodes::TSB_ZP => { self.zeropage(); 3 } // NMOS: DOP zp (NOP)
            opcodes::TSB_ABS => { self.absolute(); 4 } // NMOS: DOP abs (NOP)

            // Shift / Rotate
            opcodes::ASL_A => { let c = self.a & 0x80 != 0; self.a <<= 1; self.set_flag(FLAG_C, c); self.update_nz(self.a); 2 }
            opcodes::ASL_ZP => { let a = self.zeropage(); self.asl_mem(a); 5 }
            opcodes::ASL_ZPX => { let a = self.zeropage_x(); self.asl_mem(a); 6 }
            opcodes::ASL_ABS => { let a = self.absolute(); self.asl_mem(a); 6 }
            opcodes::ASL_ABSX => { let a = self.absolute_x(); self.asl_mem(a); 7 }

            opcodes::LSR_A => { let c = self.a & 0x01 != 0; self.a >>= 1; self.set_flag(FLAG_C, c); self.update_nz(self.a); 2 }
            opcodes::LSR_ZP => { let a = self.zeropage(); self.lsr_mem(a); 5 }
            opcodes::LSR_ZPX => { let a = self.zeropage_x(); self.lsr_mem(a); 6 }
            opcodes::LSR_ABS => { let a = self.absolute(); self.lsr_mem(a); 6 }
            opcodes::LSR_ABSX => { let a = self.absolute_x(); self.lsr_mem(a); 7 }

            opcodes::ROL_A => { let c = self.a & 0x80 != 0; self.a = (self.a << 1) | (self.get_flag(FLAG_C) as u8); self.set_flag(FLAG_C, c); self.update_nz(self.a); 2 }
            opcodes::ROL_ZP => { let a = self.zeropage(); self.rol_mem(a); 5 }
            opcodes::ROL_ZPX => { let a = self.zeropage_x(); self.rol_mem(a); 6 }
            opcodes::ROL_ABS => { let a = self.absolute(); self.rol_mem(a); 6 }
            opcodes::ROL_ABSX => { let a = self.absolute_x(); self.rol_mem(a); 7 }

            opcodes::ROR_A => { let c = self.a & 0x01 != 0; self.a = (self.a >> 1) | (self.get_flag(FLAG_C) as u8) * 0x80; self.set_flag(FLAG_C, c); self.update_nz(self.a); 2 }
            opcodes::ROR_ZP => { let a = self.zeropage(); self.ror_mem(a); 5 }
            opcodes::ROR_ZPX => { let a = self.zeropage_x(); self.ror_mem(a); 6 }
            opcodes::ROR_ABS => { let a = self.absolute(); self.ror_mem(a); 6 }
            opcodes::ROR_ABSX => { let a = self.absolute_x(); self.ror_mem(a); 7 }

            // Arithmetic
            opcodes::ADC_IMM => { let a = self.immediate(); let v = self.read(a); self.adc(v); 2 }
            opcodes::ADC_ZP => { let a = self.zeropage(); let v = self.read(a); self.adc(v); 3 }
            opcodes::ADC_ZPX => { let a = self.zeropage_x(); let v = self.read(a); self.adc(v); 4 }
            opcodes::ADC_ABS => { let a = self.absolute(); let v = self.read(a); self.adc(v); 4 }
            opcodes::ADC_ABSX => { let a = self.absolute_x(); let v = self.read(a); self.adc(v); 4 }
            opcodes::ADC_ABSY => { let a = self.absolute_y(); let v = self.read(a); self.adc(v); 4 }
            opcodes::ADC_ZPI => { let a = self.pre_indexed_y(); let v = self.read(a); self.adc(v); 5 }
            opcodes::ADC_ZPXI => { let a = self.pre_indexed_x(); let v = self.read(a); self.adc(v); 6 }
            opcodes::ADC_AIY => { let a = self.post_indexed_y(); let v = self.read(a); self.adc(v); 5 }

            opcodes::SBC_IMM => { let a = self.immediate(); let v = self.read(a); self.sbc(v); 2 }
            opcodes::SBC_ZP => { let a = self.zeropage(); let v = self.read(a); self.sbc(v); 3 }
            opcodes::SBC_ZPX => { let a = self.zeropage_x(); let v = self.read(a); self.sbc(v); 4 }
            opcodes::SBC_ABS => { let a = self.absolute(); let v = self.read(a); self.sbc(v); 4 }
            opcodes::SBC_ABSX => { let a = self.absolute_x(); let v = self.read(a); self.sbc(v); 4 }
            opcodes::SBC_ABSY => { let a = self.absolute_y(); let v = self.read(a); self.sbc(v); 4 }
            opcodes::SBC_ZPI => { let a = self.pre_indexed_y(); let v = self.read(a); self.sbc(v); 5 }
            opcodes::SBC_ZPXI => { let a = self.pre_indexed_x(); let v = self.read(a); self.sbc(v); 6 }
            opcodes::SBC_AIY => { let a = self.post_indexed_y(); let v = self.read(a); self.sbc(v); 5 }

            // ==================== NMOS 6502 Illegal Opcodes ====================
            // DOP (Double NOP): various addressing modes, read operand but do nothing
            0x04 | 0x44 | 0x64 => { self.zeropage(); 3 },
            0x0C | 0x1C | 0x3C | 0x5C | 0x7C | 0xDC | 0xFC => { self.absolute(); 4 },
            0x14 | 0x34 | 0x54 | 0xD4 | 0xF4 => { self.zeropage_x(); 4 },
            0x80 => { self.branch(true) }, // BRA on NMOS is BPL with bit 7 set (always taken)
            0x82 | 0xC2 | 0xE2 => { self.pc = self.pc.wrapping_add(1); 2 },
            0x89 => { self.pc = self.pc.wrapping_add(1); 2 }, // BIT # as NOP
            0x1A | 0x3A => 2, // NOP (INC A / DEC A on CMOS)
            0x5A | 0x7A | 0xDA | 0xFA => 2, // NOP (PHY/PLY/PHX/PLX on CMOS)
            // WAI: 2-byte NOP on NMOS
            0xCB => {
                let addr = self.immediate();
                let _ = self.read(addr);
                self.set_flag(FLAG_N, false);
                self.set_flag(FLAG_C, true);
                2
            }

            // LAX: Load A and X from memory
            0xA3 => { let a = self.pre_indexed_x(); self.lax(a); 6 }
            0xA7 => { let a = self.zeropage(); self.lax(a); 3 }
            0xAF => { let a = self.absolute(); self.lax(a); 4 }
            0xB3 => { let a = self.post_indexed_y(); self.lax(a); 5 }
            0xB7 => { let a = self.zeropage_y(); self.lax(a); 4 }
            0xBF => { let a = self.absolute_y(); self.lax(a); 4 }
            0xAB => { let a = self.immediate(); self.lax(a); 2 } // ATX (LAX immediate)

            // SAX: Store A AND X to memory
            0x83 => { let a = self.pre_indexed_x(); let v = self.a & self.x; self.write(a, v); 6 }
            0x87 => { let a = self.zeropage(); let v = self.a & self.x; self.write(a, v); 3 }
            0x8F => { let a = self.absolute(); let v = self.a & self.x; self.write(a, v); 4 }
            0x97 => { let a = self.zeropage_y(); let v = self.a & self.x; self.write(a, v); 4 }

            // DCP: Decrement memory then Compare with A
            0xC3 => { let a = self.pre_indexed_x(); self.dcp(a); 8 }
            0xC7 => { let a = self.zeropage(); self.dcp(a); 5 }
            0xCF => { let a = self.absolute(); self.dcp(a); 6 }
            0xD3 => { let a = self.post_indexed_y(); self.dcp(a); 8 }
            0xD7 => { let a = self.zeropage_x(); self.dcp(a); 6 }
            0xDB => { let a = self.absolute_y(); self.dcp(a); 7 }
            0xDF => { let a = self.absolute_x(); self.dcp(a); 7 }

            // ISB/ISC: Increment memory then Subtract with Carry
            0xE3 => { let a = self.pre_indexed_x(); self.isb(a); 8 }
            0xE7 => { let a = self.zeropage(); self.isb(a); 5 }
            0xEF => { let a = self.absolute(); self.isb(a); 6 }
            0xF3 => { let a = self.post_indexed_y(); self.isb(a); 8 }
            0xF7 => { let a = self.zeropage_x(); self.isb(a); 6 }
            0xFB => { let a = self.absolute_y(); self.isb(a); 7 }
            0xFF => { let a = self.absolute_x(); self.isb(a); 7 }

            // SLO: ASL memory then ORA with A
            0x03 => { let a = self.pre_indexed_x(); self.slo(a); 8 }
            0x07 => { let a = self.zeropage(); self.slo(a); 5 }
            0x0F => { let a = self.absolute(); self.slo(a); 6 }
            0x13 => { let a = self.post_indexed_y(); self.slo(a); 8 }
            0x17 => { let a = self.zeropage_x(); self.slo(a); 6 }
            0x1B => { let a = self.absolute_y(); self.slo(a); 7 }
            0x1F => { let a = self.absolute_x(); self.slo(a); 7 }

            // RLA: ROL memory then AND with A
            0x23 => { let a = self.pre_indexed_x(); self.rla(a); 8 }
            0x27 => { let a = self.zeropage(); self.rla(a); 5 }
            0x2F => { let a = self.absolute(); self.rla(a); 6 }
            0x33 => { let a = self.post_indexed_y(); self.rla(a); 8 }
            0x37 => { let a = self.zeropage_x(); self.rla(a); 6 }
            0x3B => { let a = self.absolute_y(); self.rla(a); 7 }
            0x3F => { let a = self.absolute_x(); self.rla(a); 7 }

            // SRE: LSR memory then EOR with A
            0x43 => { let a = self.pre_indexed_x(); self.sre(a); 8 }
            0x47 => { let a = self.zeropage(); self.sre(a); 5 }
            0x4F => { let a = self.absolute(); self.sre(a); 6 }
            0x53 => { let a = self.post_indexed_y(); self.sre(a); 8 }
            0x57 => { let a = self.zeropage_x(); self.sre(a); 6 }
            0x5B => { let a = self.absolute_y(); self.sre(a); 7 }
            0x5F => { let a = self.absolute_x(); self.sre(a); 7 }

            // RRA: ROR memory then ADC
            0x63 => { let a = self.pre_indexed_x(); self.rra(a); 8 }
            0x67 => { let a = self.zeropage(); self.rra(a); 5 }
            0x6F => { let a = self.absolute(); self.rra(a); 6 }
            0x73 => { let a = self.post_indexed_y(); self.rra(a); 8 }
            0x77 => { let a = self.zeropage_x(); self.rra(a); 6 }
            0x7B => { let a = self.absolute_y(); self.rra(a); 7 }
            0x7F => { let a = self.absolute_x(); self.rra(a); 7 }

            // AHX/SHX: Store A AND X AND (high byte of addr + 1)
            0x93 => { let addr = self.post_indexed_y(); let v = self.a & self.x & ((addr >> 8) as u8).wrapping_add(1); self.write(addr, v); 6 }
            0x9F => { let addr = self.absolute_y(); let v = self.a & self.x & ((addr >> 8) as u8).wrapping_add(1); self.write(addr, v); 5 }

            // TAS: Transfer A AND X to SP, then store high byte
            0x9B => {
                let sp_val = self.a & self.x;
                self.sp = sp_val;
                let addr = self.absolute_y();
                self.write(addr, sp_val & ((addr >> 8) as u8).wrapping_add(1));
                5
            }

            // LAS: Load A, X, and SP from memory (A,X,S := mem AND SP)
            0xBB => {
                let addr = self.absolute_y();
                let v = self.read(addr) & self.sp;
                self.a = v;
                self.x = v;
                self.sp = v;
                self.update_nz(v);
                4
            }

            // JAM: CPU lockup - PC does NOT advance
            0x02 | 0x12 | 0x22 | 0x32 | 0x42 | 0x52 | 0x62 | 0x72 | 0x92 | 0xB2 | 0xD2 | 0xF2 => {
                self.pc = self.pc.wrapping_sub(1);
                2
            }

            // ANC: AND immediate, copy N to C
            0x0B | 0x2B => {
                let addr = self.immediate();
                let v = self.read(addr);
                self.a &= v;
                self.update_nz(self.a);
                self.set_flag(FLAG_C, self.get_flag(FLAG_N));
                2
            }

            // XAA: A := X AND #immediate
            0x8B => {
                let addr = self.immediate();
                let v = self.read(addr);
                self.a = self.x & v;
                self.update_nz(self.a);
                2
            }

            // ALR: AND #immediate, then LSR A
            0x4B => {
                let addr = self.immediate();
                let v = self.read(addr);
                self.a &= v;
                let c = self.a & 0x01 != 0;
                self.a >>= 1;
                self.set_flag(FLAG_C, c);
                self.update_nz(self.a);
                2
            }

            // ARR: AND #immediate, then ROR A (special NMOS behavior)
            0x6B => {
                let addr = self.immediate();
                let v = self.read(addr);
                self.a &= v;
                let old_c = self.get_flag(FLAG_C);
                let new_c = (self.a >> 7) & 1 != 0;
                self.a = (self.a >> 1) | ((old_c as u8) << 7);
                self.set_flag(FLAG_C, new_c);
                self.set_flag(FLAG_V, ((self.a >> 6) ^ (self.a >> 5)) & 1 != 0);
                self.update_nz(self.a);
                2
            }

            // Catch-all: treat any remaining unhandled opcode as 1-byte NOP
            _ => 2,
        }
    }

    pub fn run(&mut self) {
        loop {
            let cycles = self.step();
            self.cycles += cycles as u64;
        }
    }

    // Load Accumulator with Memory
    fn lda(&mut self, addr: u16) {
        self.a = self.read(addr);
        self.update_nz(self.a);
    }

    // Load X with Memory
    fn ldx(&mut self, addr: u16) {
        self.x = self.read(addr);
        self.update_nz(self.x);
    }

    // Load Y with Memory
    fn ldy(&mut self, addr: u16) {
        self.y = self.read(addr);
        self.update_nz(self.y);
    }

    // Store Accumulator to Memory
    fn sta(&mut self, addr: u16) {
        self.write(addr, self.a);
    }

    // Store X to Memory
    fn stx(&mut self, addr: u16) {
        self.write(addr, self.x);
    }

    // Store Y to Memory
    fn sty(&mut self, addr: u16) {
        self.write(addr, self.y);
    }

    // Store Zero to Memory (CMOS)
    fn stz(&mut self, addr: u16) {
        self.write(addr, 0);
    }

    // Increment Memory
    fn inc_mem(&mut self, addr: u16) {
        let val = self.read(addr).wrapping_add(1);
        self.write(addr, val);
        self.update_nz(val);
    }

    // Decrement Memory
    fn dec_mem(&mut self, addr: u16) {
        let val = self.read(addr).wrapping_sub(1);
        self.write(addr, val);
        self.update_nz(val);
    }

    // Branch Helper
    fn branch(&mut self, condition: bool) -> u8 {
        let offset = self.fetch() as i8;
        if condition {
            self.pc = self.pc.wrapping_add(offset as u16);
            3
        } else {
            2
        }
    }

    // ASL Memory
    fn asl_mem(&mut self, addr: u16) {
        let val = self.read(addr);
        let c = val & 0x80 != 0;
        let result = val << 1;
        self.write(addr, result);
        self.set_flag(FLAG_C, c);
        self.update_nz(result);
    }

    // LSR Memory
    fn lsr_mem(&mut self, addr: u16) {
        let val = self.read(addr);
        let c = val & 0x01 != 0;
        let result = val >> 1;
        self.write(addr, result);
        self.set_flag(FLAG_C, c);
        self.update_nz(result);
    }

    // ROL Memory
    fn rol_mem(&mut self, addr: u16) {
        let val = self.read(addr);
        let c = val & 0x80 != 0;
        let result = (val << 1) | (self.get_flag(FLAG_C) as u8);
        self.write(addr, result);
        self.set_flag(FLAG_C, c);
        self.update_nz(result);
    }

    // ROR Memory
    fn ror_mem(&mut self, addr: u16) {
        let val = self.read(addr);
        let c = val & 0x01 != 0;
        let result = (val >> 1) | (self.get_flag(FLAG_C) as u8) * 0x80;
        self.write(addr, result);
        self.set_flag(FLAG_C, c);
        self.update_nz(result);
    }

    // ADC - Add with Carry
    // NES 6502 (NMOS): D flag has no effect, always binary
    fn adc(&mut self, val: u8) {
        let carry = self.get_flag(FLAG_C) as u16;
        let result = (self.a as u16) + (val as u16) + carry;
        self.set_flag(FLAG_C, result > 0xFF);
        self.set_flag(FLAG_V, (((self.a ^ val) & 0x80) == 0) && (((self.a ^ result as u8) & 0x80) != 0));
        self.a = result as u8;
        self.update_nz(self.a);
    }

    // SBC - Subtract with Borrow
    // NES 6502 (NMOS): D flag has no effect, always binary
    fn sbc(&mut self, val: u8) {
        let carry = self.get_flag(FLAG_C);
        let borrow = !carry as u16;
        let result = (self.a as u16).wrapping_sub(val as u16).wrapping_sub(borrow);
        self.set_flag(FLAG_C, result <= 0xFF);
        self.set_flag(FLAG_V, (((self.a ^ val) & 0x80) != 0) && (((self.a ^ result as u8) & 0x80) != 0));
        self.a = result as u8;
        self.update_nz(self.a);
    }

    // ==================== Illegal Opcode Helpers ====================

    // LAX: Load A and X from memory
    fn lax(&mut self, addr: u16) {
        let v = self.read(addr);
        self.a = v;
        self.x = v;
        self.update_nz(v);
    }

    // DCP: Decrement memory then Compare with A
    fn dcp(&mut self, addr: u16) {
        let v = self.read(addr).wrapping_sub(1);
        self.write(addr, v);
        self.compare(self.a, v);
    }

    // ISB: Increment memory then Subtract with Carry
    fn isb(&mut self, addr: u16) {
        let v = self.read(addr).wrapping_add(1);
        self.write(addr, v);
        self.sbc(v);
    }

    // SLO: ASL memory then ORA with A
    fn slo(&mut self, addr: u16) {
        let v = self.read(addr);
        let c = v & 0x80 != 0;
        let shifted = v << 1;
        self.write(addr, shifted);
        self.a |= shifted;
        self.set_flag(FLAG_C, c);
        self.update_nz(self.a);
    }

    // RLA: ROL memory then AND with A
    fn rla(&mut self, addr: u16) {
        let v = self.read(addr);
        let c = v & 0x80 != 0;
        let rotated = (v << 1) | (self.get_flag(FLAG_C) as u8);
        self.write(addr, rotated);
        self.a &= rotated;
        self.set_flag(FLAG_C, c);
        self.update_nz(self.a);
    }

    // SRE: LSR memory then EOR with A
    fn sre(&mut self, addr: u16) {
        let v = self.read(addr);
        let c = v & 0x01 != 0;
        let shifted = v >> 1;
        self.write(addr, shifted);
        self.a ^= shifted;
        self.set_flag(FLAG_C, c);
        self.update_nz(self.a);
    }

    // RRA: ROR memory then ADC (all flags from ADC)
    fn rra(&mut self, addr: u16) {
        let v = self.read(addr);
        let c = v & 0x01 != 0;
        let rotated = (v >> 1) | (self.get_flag(FLAG_C) as u8) * 0x80;
        self.write(addr, rotated);
        self.set_flag(FLAG_C, c); // Set carry from ROR BEFORE ADC
        self.adc(rotated);         // ADC sets final N, V, Z, C
    }
}
