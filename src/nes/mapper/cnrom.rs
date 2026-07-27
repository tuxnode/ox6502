use crate::nes::{cartridge::Mirroring, mapper::Mapper};

/**
 * Reference: https://www.nesdev.org/wiki/CNROM
 */

pub struct Cnrom {
    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,
    mirroring: Mirroring,
    chr_bank: u8,
}

impl Cnrom {
    pub fn new(prg_rom: Vec<u8>, chr_rom: Vec<u8>, mirroring: Mirroring) -> Self {
        Self {
            prg_rom,
            chr_rom,
            mirroring,
            chr_bank: 0,
        }
    }
}

impl Mapper for Cnrom {
    fn cpu_read(&mut self, addr: u16) -> u8 {
        match addr {
            0x8000..=0xFFFF => {
                let offset = (addr - 0x8000) as usize;
                self.prg_rom[offset % self.prg_rom.len()]
            }
            _ => 0,
        }
    }

    fn cpu_write(&mut self, _addr: u16, val: u8) {
        self.chr_bank = val & 0x03;
    }

    fn ppu_read(&mut self, addr: u16) -> u8 {
        let bank_start = (self.chr_bank as usize) * 0x2000;
        self.chr_rom
            .get(bank_start + addr as usize)
            .copied()
            .unwrap_or(0)
    }

    fn ppu_write(&mut self, _addr: u16, _val: u8) {}

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}
