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
            opcodes::PHX => { self.push(self.x); 3 }
            opcodes::PHY => { self.push(self.y); 3 }
            opcodes::PLA => { self.a = self.pop(); self.update_nz(self.a); 4 }
            opcodes::PLP => { self.status = (self.pop() & 0xEF) | 0x20; 4 }
            opcodes::PLX => { self.x = self.pop(); self.update_nz(self.x); 4 }
            opcodes::PLY => { self.y = self.pop(); self.update_nz(self.y); 4 }

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
            opcodes::LDA_ZPI => { let a = self.pre_indexed_y(); self.lda(a); 5 }
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
            opcodes::STA_ZPI => { let a = self.pre_indexed_y(); self.sta(a); 5 }
            opcodes::STA_ZPXI => { let a = self.pre_indexed_x(); self.sta(a); 6 }
            opcodes::STA_AIY => { let a = self.post_indexed_y(); self.sta(a); 6 }

            opcodes::STX_ZP => { let a = self.zeropage(); self.stx(a); 3 }
            opcodes::STX_ZPY => { let a = self.zeropage_y(); self.stx(a); 4 }
            opcodes::STX_ABS => { let a = self.absolute(); self.stx(a); 4 }

            opcodes::STY_ZP => { let a = self.zeropage(); self.sty(a); 3 }
            opcodes::STY_ZPX => { let a = self.zeropage_x(); self.sty(a); 4 }
            opcodes::STY_ABS => { let a = self.absolute(); self.sty(a); 4 }

            opcodes::STZ_ZP => { let a = self.zeropage(); self.stz(a); 3 }
            opcodes::STZ_ZPX => { let a = self.zeropage_x(); self.stz(a); 4 }
            opcodes::STZ_ABS => { let a = self.absolute(); self.stz(a); 4 }
            opcodes::STZ_ABSX => { let a = self.absolute_x(); self.stz(a); 5 }

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
            opcodes::JMP_INDX => { self.pc = self.pre_indexed_x(); 6 }
            opcodes::JSR => { let addr = self.absolute(); let pc = self.pc.wrapping_sub(1); self.push((pc >> 8) as u8); self.push(pc as u8); self.pc = addr; 6 }
            opcodes::RTS => { let lo = self.pop() as u16; let hi = self.pop() as u16; self.pc = ((hi << 8) | lo).wrapping_add(1); 6 }
            opcodes::RTI => { self.status = (self.pop() & 0xEF) | 0x20; let lo = self.pop() as u16; let hi = self.pop() as u16; self.pc = (hi << 8) | lo; 6 }

            // Increment / Decrement
            opcodes::INC_A => { self.a = self.a.wrapping_add(1); self.update_nz(self.a); 2 }
            opcodes::INC_ZP => { let a = self.zeropage(); self.inc_mem(a); 5 }
            opcodes::INC_ZPX => { let a = self.zeropage_x(); self.inc_mem(a); 6 }
            opcodes::INC_ABS => { let a = self.absolute(); self.inc_mem(a); 6 }
            opcodes::INC_ABSX => { let a = self.absolute_x(); self.inc_mem(a); 7 }
            opcodes::DEC_A => { self.a = self.a.wrapping_sub(1); self.update_nz(self.a); 2 }
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
            opcodes::CMP_ZPI => { let a = self.pre_indexed_y(); let v = self.read(a); self.compare(self.a, v); 5 }
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
            opcodes::BRA => { self.branch(true) }

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
            opcodes::AND_ZPI => { let a = self.pre_indexed_y(); let v = self.read(a); self.a &= v; self.update_nz(self.a); 5 }
            opcodes::AND_ZPXI => { let a = self.pre_indexed_x(); let v = self.read(a); self.a &= v; self.update_nz(self.a); 6 }
            opcodes::AND_AIY => { let a = self.post_indexed_y(); let v = self.read(a); self.a &= v; self.update_nz(self.a); 5 }

            opcodes::ORA_IMM => { let a = self.immediate(); let v = self.read(a); self.a |= v; self.update_nz(self.a); 2 }
            opcodes::ORA_ZP => { let a = self.zeropage(); let v = self.read(a); self.a |= v; self.update_nz(self.a); 3 }
            opcodes::ORA_ZPX => { let a = self.zeropage_x(); let v = self.read(a); self.a |= v; self.update_nz(self.a); 4 }
            opcodes::ORA_ABS => { let a = self.absolute(); let v = self.read(a); self.a |= v; self.update_nz(self.a); 4 }
            opcodes::ORA_ABSX => { let a = self.absolute_x(); let v = self.read(a); self.a |= v; self.update_nz(self.a); 4 }
            opcodes::ORA_ABSY => { let a = self.absolute_y(); let v = self.read(a); self.a |= v; self.update_nz(self.a); 4 }
            opcodes::ORA_ZPI => { let a = self.pre_indexed_y(); let v = self.read(a); self.a |= v; self.update_nz(self.a); 5 }
            opcodes::ORA_ZPXI => { let a = self.pre_indexed_x(); let v = self.read(a); self.a |= v; self.update_nz(self.a); 6 }
            opcodes::ORA_AIY => { let a = self.post_indexed_y(); let v = self.read(a); self.a |= v; self.update_nz(self.a); 5 }

            opcodes::EOR_IMM => { let a = self.immediate(); let v = self.read(a); self.a ^= v; self.update_nz(self.a); 2 }
            opcodes::EOR_ZP => { let a = self.zeropage(); let v = self.read(a); self.a ^= v; self.update_nz(self.a); 3 }
            opcodes::EOR_ZPX => { let a = self.zeropage_x(); let v = self.read(a); self.a ^= v; self.update_nz(self.a); 4 }
            opcodes::EOR_ABS => { let a = self.absolute(); let v = self.read(a); self.a ^= v; self.update_nz(self.a); 4 }
            opcodes::EOR_ABSX => { let a = self.absolute_x(); let v = self.read(a); self.a ^= v; self.update_nz(self.a); 4 }
            opcodes::EOR_ABSY => { let a = self.absolute_y(); let v = self.read(a); self.a ^= v; self.update_nz(self.a); 4 }
            opcodes::EOR_ZPI => { let a = self.pre_indexed_y(); let v = self.read(a); self.a ^= v; self.update_nz(self.a); 5 }
            opcodes::EOR_ZPXI => { let a = self.pre_indexed_x(); let v = self.read(a); self.a ^= v; self.update_nz(self.a); 6 }
            opcodes::EOR_AIY => { let a = self.post_indexed_y(); let v = self.read(a); self.a ^= v; self.update_nz(self.a); 5 }

            opcodes::BIT_IMM => { let a = self.immediate(); let v = self.read(a); self.set_flag(FLAG_Z, (self.a & v) == 0); self.set_flag(FLAG_N, v & 0x80 != 0); self.set_flag(FLAG_V, v & 0x40 != 0); 2 }
            opcodes::BIT_ZP => { let a = self.zeropage(); let v = self.read(a); self.set_flag(FLAG_Z, (self.a & v) == 0); self.set_flag(FLAG_N, v & 0x80 != 0); self.set_flag(FLAG_V, v & 0x40 != 0); 3 }
            opcodes::BIT_ZPX => { let a = self.zeropage_x(); let v = self.read(a); self.set_flag(FLAG_Z, (self.a & v) == 0); self.set_flag(FLAG_N, v & 0x80 != 0); self.set_flag(FLAG_V, v & 0x40 != 0); 4 }
            opcodes::BIT_ABS => { let a = self.absolute(); let v = self.read(a); self.set_flag(FLAG_Z, (self.a & v) == 0); self.set_flag(FLAG_N, v & 0x80 != 0); self.set_flag(FLAG_V, v & 0x40 != 0); 4 }
            opcodes::BIT_ABSX => { let a = self.absolute_x(); let v = self.read(a); self.set_flag(FLAG_Z, (self.a & v) == 0); self.set_flag(FLAG_N, v & 0x80 != 0); self.set_flag(FLAG_V, v & 0x40 != 0); 4 }

            opcodes::TRB_ZP => { let a = self.zeropage(); let v = self.read(a); self.set_flag(FLAG_Z, (self.a & v) == 0); self.write(a, v & !self.a); 5 }
            opcodes::TRB_ABS => { let a = self.absolute(); let v = self.read(a); self.set_flag(FLAG_Z, (self.a & v) == 0); self.write(a, v & !self.a); 6 }

            opcodes::TSB_ZP => { let a = self.zeropage(); let v = self.read(a); self.set_flag(FLAG_Z, (self.a & v) == 0); self.write(a, v | self.a); 5 }
            opcodes::TSB_ABS => { let a = self.absolute(); let v = self.read(a); self.set_flag(FLAG_Z, (self.a & v) == 0); self.write(a, v | self.a); 6 }

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

            _ => panic!("Unknown opcode: {:#04X}", opcode),
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
}
