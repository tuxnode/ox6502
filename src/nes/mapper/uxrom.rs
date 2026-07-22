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

    fn ppu_read(&mut self, addr: u16) -> u8 {
        if (addr as usize) < self.chr_ram.len() {
            self.chr_ram[addr as usize]
        } else {
            0
        }
    }

    fn ppu_write(&mut self, addr: u16, val: u8) {
        if (addr as usize) < self.chr_ram.len() {
            self.chr_ram[addr as usize] = val;
        }
    }
    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn prg_rom_with_banks(bank_count: usize) -> Vec<u8> {
        let mut prg_rom = vec![0; bank_count * 0x4000];
        for bank in 0..bank_count {
            for offset in 0..0x4000 {
                prg_rom[bank * 0x4000 + offset] = bank as u8;
            }
        }
        prg_rom
    }

    #[test]
    fn cpu_read_maps_selected_bank_at_8000() {
        let prg_rom = prg_rom_with_banks(4);
        let chr_ram = vec![0; 0x2000];
        let mut mapper = Uxrom::new(prg_rom, chr_ram, Mirroring::Vertical, 2);

        assert_eq!(mapper.cpu_read(0x8000), 2);
        assert_eq!(mapper.cpu_read(0x9ABC), 2);
        assert_eq!(mapper.cpu_read(0xBFFF), 2);
    }

    #[test]
    fn cpu_read_maps_fixed_last_bank_at_c000() {
        let prg_rom = prg_rom_with_banks(4);
        let chr_ram = vec![0; 0x2000];
        let mut mapper = Uxrom::new(prg_rom, chr_ram, Mirroring::Vertical, 0);

        assert_eq!(mapper.cpu_read(0xC000), 3);
        assert_eq!(mapper.cpu_read(0xD234), 3);
        assert_eq!(mapper.cpu_read(0xFFFF), 3);

        mapper.cpu_write(0x8000, 1);

        assert_eq!(mapper.cpu_read(0xC000), 3);
        assert_eq!(mapper.cpu_read(0xFFFF), 3);
    }

    #[test]
    fn cpu_write_selects_low_four_bits_for_switchable_bank() {
        let prg_rom = prg_rom_with_banks(4);
        let chr_ram = vec![0; 0x2000];
        let mut mapper = Uxrom::new(prg_rom, chr_ram, Mirroring::Vertical, 0);

        mapper.cpu_write(0x8000, 0x03);
        assert_eq!(mapper.cpu_read(0x8000), 3);

        mapper.cpu_write(0xFFFF, 0x12);
        assert_eq!(mapper.cpu_read(0x8000), 2);
    }

    #[test]
    fn selected_bank_wraps_when_bank_number_exceeds_prg_size() {
        let prg_rom = prg_rom_with_banks(4);
        let chr_ram = vec![0; 0x2000];
        let mut mapper = Uxrom::new(prg_rom, chr_ram, Mirroring::Vertical, 0);

        mapper.cpu_write(0x8000, 0x06);

        assert_eq!(mapper.cpu_read(0x8000), 2);
    }

    #[test]
    fn ppu_read_write_accesses_chr_ram() {
        let prg_rom = prg_rom_with_banks(2);
        let chr_ram = vec![0; 0x2000];
        let mut mapper = Uxrom::new(prg_rom, chr_ram, Mirroring::Vertical, 0);

        mapper.ppu_write(0x0000, 0x12);
        mapper.ppu_write(0x1FFF, 0x34);

        assert_eq!(mapper.ppu_read(0x0000), 0x12);
        assert_eq!(mapper.ppu_read(0x1FFF), 0x34);
    }

    #[test]
    fn ppu_access_outside_chr_ram_is_ignored() {
        let prg_rom = prg_rom_with_banks(2);
        let chr_ram = vec![0; 0x2000];
        let mut mapper = Uxrom::new(prg_rom, chr_ram, Mirroring::Vertical, 0);

        mapper.ppu_write(0x2000, 0x56);

        assert_eq!(mapper.ppu_read(0x2000), 0);
    }

    #[test]
    fn mirroring_comes_from_cartridge() {
        let prg_rom = prg_rom_with_banks(2);
        let chr_ram = vec![0; 0x2000];
        let mapper = Uxrom::new(prg_rom, chr_ram, Mirroring::Horizontal, 0);

        assert_eq!(mapper.mirroring(), Mirroring::Horizontal);
    }
}
