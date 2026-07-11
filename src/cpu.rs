// mos 6502 CPU mod

// CMOS(65C02) Documents:
// - W65C02S datasheet：https://www.westerndesigncenter.com/wdc/documentation/w65c02s.pdf
// - W65C02S Programming Manual：https://www.westerndesigncenter.com/wdc/documentation/w65c02-programming-manual.pdf

use crate::bus::Bus;

/*
Status Registers Layout:
Bit:  7  6  5  4  3  2  1  0
      N  V  -  B  D  I  Z  C
*/
const FLAG_C: u8 = 1 << 0; // 0x01
const FLAG_Z: u8 = 1 << 1; // 0x02
const FLAG_I: u8 = 1 << 2; // 0x04
const FLAG_D: u8 = 1 << 3; // 0x08
const FLAG_B: u8 = 1 << 4; // 0x10
const FLAG_V: u8 = 1 << 6; // 0x40
const FLAG_N: u8 = 1 << 7; // 0x80

struct Cpu<B: Bus> {
    pub a: u8, // Accumulator Register
    pub x: u8,
    pub y: u8,
    pub sp: u8, // Stack Pointer Register (S)
    pub pc: u16,
    pub status: u8, // Processor Status Register (P)

    bus: B,
}

impl<B: Bus> Cpu<B> {
    pub fn new(bus: B) -> Self {
        let mut cpu = Self {
            a: 0,
            x: 0,
            y: 0,
            sp: 0xFD,
            status: 0,
            pc: 0,
            bus: bus,
        };
        cpu.pc = cpu.fetch_u16();
        cpu.set_flag(FLAG_I, true);
        cpu
    }

    fn set_flag(&mut self, flags: u8, val: bool) {
        if val {
            self.status |= flags;
        } else {
            self.status &= !flags;
        }
    }

    fn get_flag(&self, flag: u8) -> bool {
        self.status & flag != 0
    }

    // update N and Z flag bit
    fn update_nz(&mut self, val: u8) {
        self.set_flag(FLAG_N, val & 0x80 != 0);
        self.set_flag(FLAG_Z, val == 0);
    }

    fn fetch(&mut self) -> u8 {
        let val = self.read(self.pc);
        self.pc = self.pc.wrapping_add(1);
        val
    }
    fn fetch_u16(&mut self) -> u16 {
        let lo = self.fetch() as u16;
        let hi = self.fetch() as u16;
        (hi << 8) | lo
    }

    fn read(&mut self, addr: u16) -> u8 {
        self.bus.read(addr)
    }

    fn write(&mut self, addr: u16, val: u8) {
        self.bus.write(addr, val);
    }
}
