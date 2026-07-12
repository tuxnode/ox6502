// mos 6502 CPU mod

// CMOS(65C02) Documents:
// - W65C02S datasheet：https://www.westerndesigncenter.com/wdc/documentation/w65c02s.pdf
// - W65C02S Programming Manual：https://www.westerndesigncenter.com/wdc/documentation/w65c02-programming-manual.pdf

use crate::bus::Bus;
use crate::instructions::{FLAG_I, FLAG_N, FLAG_Z};

pub struct Cpu<B: Bus> {
    pub a: u8, // Accumulator Register
    pub x: u8,
    pub y: u8,
    pub sp: u8, // Stack Pointer Register (S)
    pub pc: u16,
    pub status: u8, // Processor Status Register (P)

    pub(crate) cycles: u64,
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
            cycles: 0,
            bus,
        };
        cpu.pc = cpu.fetch_u16();
        cpu.set_flag(FLAG_I, true);
        cpu
    }

    pub(crate) fn set_flag(&mut self, flags: u8, val: bool) {
        if val {
            self.status |= flags;
        } else {
            self.status &= !flags;
        }
    }

    pub(crate) fn get_flag(&self, flag: u8) -> bool {
        self.status & flag != 0
    }

    pub(crate) fn update_nz(&mut self, val: u8) {
        self.set_flag(FLAG_N, val & 0x80 != 0);
        self.set_flag(FLAG_Z, val == 0);
    }

    pub(crate) fn fetch(&mut self) -> u8 {
        let val = self.read(self.pc);
        self.pc = self.pc.wrapping_add(1);
        val
    }
    pub(crate) fn fetch_u16(&mut self) -> u16 {
        let lo = self.fetch() as u16;
        let hi = self.fetch() as u16;
        (hi << 8) | lo
    }

    pub(crate) fn read(&mut self, addr: u16) -> u8 {
        self.bus.read(addr)
    }

    pub(crate) fn write(&mut self, addr: u16, val: u8) {
        self.bus.write(addr, val);
    }

    // Basic Instructions Dependences
    pub(crate) fn push(&mut self, val: u8) {
        let addr = 0x0100 | self.sp as u16;
        self.write(addr, val);
        self.sp = self.sp.wrapping_sub(1);
    }

    pub(crate) fn pop(&mut self) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        let addr = 0x0100 | self.sp as u16;
        self.read(addr)
    }
}
