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
}
