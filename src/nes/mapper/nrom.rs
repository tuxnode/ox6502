use crate::nes::cartridge::Mirroring;
use crate::nes::mapper::Mapper;

pub struct Nrom {
    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,
    mirroring: Mirroring,
}

impl Nrom {
    pub fn new(prg_rom: Vec<u8>, chr_rom: Vec<u8>, mirroring: Mirroring) -> Self {
        Self {
            prg_rom,
            chr_rom,
            mirroring,
        }
    }
}

impl Mapper for Nrom {
    fn cpu_read(&mut self, addr: u16) -> u8 {
        match addr {
            // PRG ROM: $8000-$FFFF
            0x8000..=0xFFFF => {
                let offset = (addr - 0x8000) as usize;
                self.prg_rom[offset % self.prg_rom.len()]
            }
            _ => 0,
        }
    }

    fn cpu_write(&mut self, _addr: u16, _val: u8) {
        // PRG ROM is read-only on NROM
    }

    fn ppu_read(&mut self, addr: u16) -> u8 {
        match addr {
            // Pattern tables: $0000-$1FFF from CHR ROM
            0x0000..=0x1FFF if (addr as usize) < self.chr_rom.len() => self.chr_rom[addr as usize],
            _ => 0,
        }
    }

    fn ppu_write(&mut self, _addr: u16, _val: u8) {
        // CHR ROM is read-only on NROM
    }

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}
