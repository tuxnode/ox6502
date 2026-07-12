// MOS Addressing Implementation

use crate::{bus::Bus, cpu::Cpu};

impl<B: Bus> Cpu<B> {
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

    // Zero Page Indexing X, Y: Read 8 bits Address then add X register's vaule
    pub(crate) fn zeropage_x(&mut self) -> u16 {
        (self.fetch().wrapping_add(self.x)) as u16
    }

    pub(crate) fn zeropage_y(&mut self) -> u16 {
        (self.fetch().wrapping_add(self.y)) as u16
    }

    // Absolute Addressing X, Y
    pub(crate) fn absolute_x(&mut self) -> u16 {
        self.fetch_u16().wrapping_add(self.x as u16)
    }

    pub(crate) fn absolute_y(&mut self) -> u16 {
        self.fetch_u16().wrapping_add(self.y as u16)
    }

    // Indirect Addressing (cycles + 2)
    pub(crate) fn indirect(&mut self) -> u16 {
        let ptr = self.fetch_u16();
        let lo = self.read(ptr) as u16;
        let hi = self.read(ptr.wrapping_add(1)) as u16;
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

    pub(crate) fn pre_indexed_y(&mut self) -> u16 {
        let zp = self.fetch();
        let indexed = zp.wrapping_add(self.y);
        let lo = self.read(indexed as u16) as u16;
        let hi = self.read(indexed.wrapping_add(1) as u16) as u16;
        (hi << 8) | lo
    }

    // Post-Indexed Indirect, "(Zero-Page),X,Y"
    pub(crate) fn post_indexed_x(&mut self) -> u16 {
        let zp = self.fetch();
        let lo = self.read(zp as u16) as u16;
        let hi = self.read(zp.wrapping_add(1) as u16) as u16;
        ((hi << 8) | lo).wrapping_add(self.x as u16)
    }

    pub(crate) fn post_indexed_y(&mut self) -> u16 {
        let zp = self.fetch();
        let lo = self.read(zp as u16) as u16;
        let hi = self.read(zp.wrapping_add(1) as u16) as u16;
        ((hi << 8) | lo).wrapping_add(self.y as u16)
    }

    // Relative Addressing (Conditional Branching)
    pub(crate) fn relative(&mut self) -> u16 {
        let offset = self.fetch() as i8;
        self.pc.wrapping_add(offset as u16)
    }
}
