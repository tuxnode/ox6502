use crate::nes::{cartridge::Mirroring, mapper::Mapper};

/**
 * Reference: https://www.nesdev.org/wiki/UxROM
 */

pub struct Uxrom {
    prg_rom: Vec<u8>,
    chr_ram: Vec<u8>,
    mirroring: Mirroring,
    bank_select: u8,
}

impl Uxrom {
    pub fn new(prg_rom: Vec<u8>, chr_ram: Vec<u8>, mirroring: Mirroring, bank_select: u8) -> Self {
        Self {
            prg_rom,
            chr_ram,
            mirroring,
            bank_select,
        }
    }
}

impl Mapper for Uxrom {
    fn cpu_read(&mut self, addr: u16) -> u8 {
        match addr {
            // $8000-$BFFF
            0x8000..=0xBFFF => {
                // switch a 16kb bank mapping
                let offset = (self.bank_select as usize * 0x4000) + ((addr - 0x8000) as usize);
                self.prg_rom[offset % self.prg_rom.len()]
            }
            // 16 KB PRG ROM bank, fixed to the last bank
            0xC000..=0xFFFF => {
                let offset = self.prg_rom.len() - 0x4000 + ((addr - 0xC000) as usize);
                self.prg_rom[offset]
            }
            _ => 0,
        }
    }

    fn cpu_write(&mut self, addr: u16, val: u8) {
        if addr >= 0x8000 {
            // low 4 bits is bank_select
            self.bank_select = val & 0x0F;
        }
    }

    fn ppu_read(&mut self, addr: u16) -> u8 {}
    fn ppu_write(&mut self, addr: u16, val: u8) {}
    fn mirroring(&self) -> Mirroring {}
}
