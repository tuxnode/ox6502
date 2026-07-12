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

            //Stack
            opcodes::PHA => {
                self.push(self.a);
                3
            }
            opcodes::PHP => {
                self.push(self.status);
                3
            }
            opcodes::PHX => {
                self.push(self.x);
                3
            }
            opcodes::PHY => {
                self.push(self.y);
                3
            }

            opcodes::PLA => {
                self.a = self.pop();
                self.update_nz(self.a);
                4
            }
            opcodes::PLP => {
                self.status = self.pop();
                4
            }
            opcodes::PLX => {
                self.x = self.pop();
                self.update_nz(self.x);
                4
            }
            opcodes::PLY => {
                self.y = self.pop();
                self.update_nz(self.y);
                4
            }

            // Transfer
            opcodes::TAX => {
                self.x = self.a;
                self.update_nz(self.x);
                2
            }
            opcodes::TAY => {
                self.y = self.a;
                self.update_nz(self.y);
                2
            }
            opcodes::TXA => {
                self.a = self.x;
                self.update_nz(self.a);
                2
            }
            opcodes::TYA => {
                self.a = self.y;
                self.update_nz(self.a);
                2
            }
            opcodes::TSX => {
                self.x = self.sp;
                self.update_nz(self.x);
                2
            }
            opcodes::TXS => {
                self.sp = self.x;
                2
            }

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
}
