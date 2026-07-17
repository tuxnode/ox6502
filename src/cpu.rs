// mos 6502 CPU mod

// CMOS(65C02) Documents:
// - W65C02S datasheet：https://www.westerndesigncenter.com/wdc/documentation/w65c02s.pdf
// - W65C02S Programming Manual：https://www.westerndesigncenter.com/wdc/documentation/w65c02-programming-manual.pdf

use crate::bus::Bus;
use crate::instructions::{FLAG_C, FLAG_B, FLAG_I, FLAG_N, FLAG_Z};

pub struct Cpu<B: Bus> {
    pub a: u8, // Accumulator Register
    pub x: u8,
    pub y: u8,
    pub sp: u8, // Stack Pointer Register (S)
    pub pc: u16,
    pub status: u8, // Processor Status Register (P)

    pub cycles: u64,
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
            pc: 0xFFFC, // Reset vector address
            cycles: 0,
            bus,
        };
        cpu.pc = cpu.fetch_u16(); // Read reset vector from $FFFC-$FFFD
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

    pub fn get_flag(&self, flag: u8) -> bool {
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

    pub fn read(&mut self, addr: u16) -> u8 {
        self.bus.cpu_read(addr)
    }

    pub(crate) fn write(&mut self, addr: u16, val: u8) {
        self.bus.cpu_write(addr, val);
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

    pub(crate) fn compare(&mut self, reg: u8, val: u8) {
        let result = reg.wrapping_sub(val);
        self.set_flag(FLAG_C, reg >= val);
        self.update_nz(result);
    }

    /// Handle NMI interrupt (similar to BRK but uses $FFFA-$FFFB vector)
    pub fn handle_nmi(&mut self) {
        self.push((self.pc >> 8) as u8);
        self.push(self.pc as u8);
        // NMI pushes status with B=0, unlike BRK which sets B=1
        self.push(self.status & !FLAG_B);
        self.set_flag(FLAG_I, true);
        let lo = self.read(0xFFFA) as u16;
        let hi = self.read(0xFFFB) as u16;
        self.pc = (hi << 8) | lo;
        self.cycles += 7;
    }

    /// NES-specific run loop with DMA and NMI support
    pub fn run_nes(&mut self) {
        loop {
            let step_cycles = self.step() as u64;
            self.cycles += step_cycles;

            // Tick bus: get DMA cycles and check NMI
            let tick = self.bus.tick();
            self.cycles += tick.extra_cycles as u64;

            // Handle NMI (cannot be interrupted if I flag is set)
            if tick.nmi && !self.get_flag(FLAG_I) {
                self.handle_nmi();
            }
        }
    }
}
