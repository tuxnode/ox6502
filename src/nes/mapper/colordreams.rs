use crate::nes::{cartridge::Mirroring, mapper::Mapper};

/**
 * Color Dreams (mapper 11)
 * Reference: https://www.nesdev.org/wiki/Color_Dreams
 *
 * 32KB switchable PRG ROM window and 8KB switchable CHR ROM window. Both
 * bank numbers come from a single register at $8000-$FFFF.
 */
pub struct ColorDreams {
    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,
    mirroring: Mirroring,
    prg_bank: u8,
    chr_bank: u8,
}

impl ColorDreams {
    pub fn new(prg_rom: Vec<u8>, chr_rom: Vec<u8>, mirroring: Mirroring) -> Self {
        Self {
            prg_rom,
            chr_rom,
            mirroring,
            prg_bank: 0,
            chr_bank: 0,
        }
    }
}

impl Mapper for ColorDreams {
    fn cpu_read(&mut self, addr: u16) -> u8 {
        match addr {
            // 32KB switchable PRG ROM window.
            0x8000..=0xFFFF => {
                let offset = self.prg_bank as usize * 0x8000 + (addr - 0x8000) as usize;
                self.prg_rom.get(offset).copied().unwrap_or(0)
            }
            _ => 0,
        }
    }

    fn cpu_write(&mut self, addr: u16, val: u8) {
        if addr < 0x8000 {
            return;
        }

        // Bits 0-1 select the 32KB PRG bank; bits 4-7 select the 8KB CHR bank.
        // Mask against the actual ROM size so smaller carts wrap instead of
        // reading past the end of the data.
        let prg_banks = (self.prg_rom.len() / 0x8000).max(1);
        let chr_banks = (self.chr_rom.len() / 0x2000).max(1);
        self.prg_bank = ((val & 0x03) as usize % prg_banks) as u8;
        self.chr_bank = (((val >> 4) & 0x0F) as usize % chr_banks) as u8;
    }

    fn ppu_read(&mut self, addr: u16) -> u8 {
        let offset = self.chr_bank as usize * 0x2000 + addr as usize;
        self.chr_rom.get(offset).copied().unwrap_or(0)
    }

    fn ppu_write(&mut self, _addr: u16, _val: u8) {
        // Color Dreams boards use CHR ROM.
    }

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// PRG ROM with `bank_count` 32KB banks, each filled with its bank index.
    fn prg_rom_with_banks(bank_count: usize) -> Vec<u8> {
        let mut prg_rom = vec![0; bank_count * 0x8000];
        for bank in 0..bank_count {
            for offset in 0..0x8000 {
                prg_rom[bank * 0x8000 + offset] = bank as u8;
            }
        }
        prg_rom
    }

    /// CHR ROM with `bank_count` 8KB banks, each filled with `0xA0 + bank`.
    fn chr_rom_with_banks(bank_count: usize) -> Vec<u8> {
        let mut chr_rom = vec![0; bank_count * 0x2000];
        for bank in 0..bank_count {
            for offset in 0..0x2000 {
                chr_rom[bank * 0x2000 + offset] = 0xA0 + bank as u8;
            }
        }
        chr_rom
    }

    #[test]
    fn cpu_read_maps_prg_bank_zero_at_8000_by_default() {
        let mut mapper = ColorDreams::new(
            prg_rom_with_banks(4),
            chr_rom_with_banks(4),
            Mirroring::Vertical,
        );

        assert_eq!(mapper.cpu_read(0x8000), 0);
        assert_eq!(mapper.cpu_read(0x9ABC), 0);
        assert_eq!(mapper.cpu_read(0xFFFF), 0);
    }

    #[test]
    fn cpu_write_selects_32kb_prg_bank() {
        let mut mapper = ColorDreams::new(
            prg_rom_with_banks(4),
            chr_rom_with_banks(4),
            Mirroring::Vertical,
        );

        mapper.cpu_write(0x8000, 0x01);
        assert_eq!(mapper.cpu_read(0x8000), 1);
        assert_eq!(mapper.cpu_read(0xFFFF), 1);

        mapper.cpu_write(0xFFFF, 0x03);
        assert_eq!(mapper.cpu_read(0x8000), 3);
    }

    #[test]
    fn cpu_write_selects_8kb_chr_bank() {
        let mut mapper = ColorDreams::new(
            prg_rom_with_banks(4),
            chr_rom_with_banks(4),
            Mirroring::Vertical,
        );

        assert_eq!(mapper.ppu_read(0x0000), 0xA0);

        mapper.cpu_write(0x8000, 0x10);
        assert_eq!(mapper.ppu_read(0x0000), 0xA1);
        assert_eq!(mapper.ppu_read(0x1FFF), 0xA1);

        mapper.cpu_write(0x8000, 0xF0);
        assert_eq!(mapper.ppu_read(0x0000), 0xA3);
    }

    #[test]
    fn single_register_sets_prg_and_chr_banks() {
        let mut mapper = ColorDreams::new(
            prg_rom_with_banks(4),
            chr_rom_with_banks(4),
            Mirroring::Vertical,
        );

        // Bits 0-1 = PRG bank 1, bits 4-7 = CHR bank 2.
        mapper.cpu_write(0x8000, 0x21);

        assert_eq!(mapper.cpu_read(0x8000), 1);
        assert_eq!(mapper.ppu_read(0x0000), 0xA2);
    }

    #[test]
    fn bank_numbers_wrap_to_rom_size() {
        let mut mapper = ColorDreams::new(
            prg_rom_with_banks(2),
            chr_rom_with_banks(3),
            Mirroring::Vertical,
        );

        // Only two PRG banks and three CHR banks exist.
        mapper.cpu_write(0x8000, 0x33);

        assert_eq!(mapper.prg_bank, 1);
        assert_eq!(mapper.chr_bank, 0);
        assert_eq!(mapper.cpu_read(0x8000), 1);
        assert_eq!(mapper.ppu_read(0x0000), 0xA0);
    }

    #[test]
    fn cpu_write_below_8000_is_ignored() {
        let mut mapper = ColorDreams::new(
            prg_rom_with_banks(4),
            chr_rom_with_banks(4),
            Mirroring::Vertical,
        );

        mapper.cpu_write(0x6000, 0x23);

        assert_eq!(mapper.cpu_read(0x8000), 0);
        assert_eq!(mapper.ppu_read(0x0000), 0xA0);
    }

    #[test]
    fn cpu_read_below_8000_returns_zero() {
        let mut mapper = ColorDreams::new(
            prg_rom_with_banks(2),
            chr_rom_with_banks(2),
            Mirroring::Vertical,
        );

        assert_eq!(mapper.cpu_read(0x0000), 0);
        assert_eq!(mapper.cpu_read(0x6000), 0);
        assert_eq!(mapper.cpu_read(0x7FFF), 0);
    }

    #[test]
    fn ppu_read_outside_chr_returns_zero() {
        let mut mapper = ColorDreams::new(
            prg_rom_with_banks(2),
            chr_rom_with_banks(1),
            Mirroring::Vertical,
        );

        assert_eq!(mapper.ppu_read(0x1FFF), 0xA0);
        assert_eq!(mapper.ppu_read(0x2000), 0);
        assert_eq!(mapper.ppu_read(0x4000), 0);
    }

    #[test]
    fn ppu_write_is_ignored() {
        let mut mapper = ColorDreams::new(
            prg_rom_with_banks(2),
            chr_rom_with_banks(2),
            Mirroring::Vertical,
        );

        mapper.ppu_write(0x0000, 0x42);

        assert_eq!(mapper.ppu_read(0x0000), 0xA0);
    }

    #[test]
    fn mirroring_comes_from_cartridge() {
        for mirroring in [
            Mirroring::Horizontal,
            Mirroring::Vertical,
            Mirroring::FourScreen,
            Mirroring::OneScreenA,
        ] {
            let mapper = ColorDreams::new(prg_rom_with_banks(2), chr_rom_with_banks(2), mirroring);

            assert_eq!(mapper.mirroring(), mirroring);
        }
    }
}
