// MOS Addressing Implementation

use crate::{bus::Bus, cpu::Cpu};

impl<B: Bus> Cpu<B> {
    /// Check whether an indexed effective address crosses a page boundary.
    pub(crate) fn crossed_page(&self, base: u16, effective: u16) -> bool {
        (base & 0xFF00) != (effective & 0xFF00)
    }

    // Immediately Addressing
    pub(crate) fn immediate(&mut self) -> u16 {
        let addr = self.pc;
        self.pc = self.pc.wrapping_add(1);
        addr
    }

    // Absolute Addressing
    pub(crate) fn absolute(&mut self) -> u16 {
        self.fetch_u16()
    }

    // Zero Page Addressing
    pub(crate) fn zeropage(&mut self) -> u16 {
        self.fetch() as u16
    }

    // Zero Page Indexing X, Y: Read 8 bits Address then add X register's value
    pub(crate) fn zeropage_x(&mut self) -> u16 {
        (self.fetch().wrapping_add(self.x)) as u16
    }

    pub(crate) fn zeropage_y(&mut self) -> u16 {
        (self.fetch().wrapping_add(self.y)) as u16
    }

    // Absolute Addressing X, Y — returns (base_addr, effective_addr)
    pub(crate) fn absolute_x(&mut self) -> (u16, u16) {
        let base = self.fetch_u16();
        let addr = base.wrapping_add(self.x as u16);
        (base, addr)
    }

    pub(crate) fn absolute_y(&mut self) -> (u16, u16) {
        let base = self.fetch_u16();
        let addr = base.wrapping_add(self.y as u16);
        (base, addr)
    }

    // Indirect Addressing (NMOS 6502 page boundary bug)
    pub(crate) fn indirect(&mut self) -> u16 {
        let ptr = self.fetch_u16();
        let lo = self.read(ptr) as u16;
        // NMOS bug: high byte read from same page, not ptr+1
        let hi_addr = (ptr & 0xFF00) | ((ptr + 1) & 0xFF);
        let hi = self.read(hi_addr) as u16;
        (hi << 8) | lo
    }

    // Pre-Indexed Indirect, "(Zero-Page,X, Y)"
    pub(crate) fn pre_indexed_x(&mut self) -> u16 {
        let zp = self.fetch();
        let indexed = zp.wrapping_add(self.x);
        let lo = self.read(indexed as u16) as u16;
        let hi = self.read(indexed.wrapping_add(1) as u16) as u16;
        (hi << 8) | lo
    }

    pub(crate) fn post_indexed_y(&mut self) -> (u16, u16) {
        let zp = self.fetch();
        let lo = self.read(zp as u16) as u16;
        let hi = self.read(zp.wrapping_add(1) as u16) as u16;
        let base = (hi << 8) | lo;
        let addr = base.wrapping_add(self.y as u16);
        (base, addr)
    }

}
