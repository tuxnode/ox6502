use crate::nes::{cartridge::Mirroring, mapper::Mapper};
pub struct Gxrom {
    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,
    mirroring: Mirroring,
    prg_bank: u8,
    chr_bank: u8,
}
impl Gxrom {
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
impl Mapper for Gxrom {
    fn cpu_read(&mut self, addr: u16) -> u8 {
        match addr {
            // 32 KB PRG ROM window
            0x8000..=0xFFFF => {
                let offset = (self.prg_bank as usize * 0x8000) + ((addr - 0x8000) as usize);
                self.prg_rom[offset % self.prg_rom.len()]
            }
            _ => 0,
        }
    }
    fn cpu_write(&mut self, _addr: u16, val: u8) {
        // GxROM uses two bits each for the PRG (32KB) and CHR (8KB) banks.
        // Mask them against the actual ROM size so smaller carts wrap
        // instead of reading past the end of the data.
        let prg_banks = (self.prg_rom.len() / 0x8000).max(1);
        let chr_banks = (self.chr_rom.len() / 0x2000).max(1);
        self.prg_bank = (((val >> 4) & 0x03) as usize % prg_banks) as u8;
        self.chr_bank = ((val & 0x03) as usize % chr_banks) as u8;
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
#[cfg(test)]
mod tests {
    use super::*;
    /// PRG ROM with `bank_count` 32 KB banks, each filled with its bank index.
    fn prg_rom_with_banks(bank_count: usize) -> Vec<u8> {
        let mut prg_rom = vec![0; bank_count * 0x8000];
        for bank in 0..bank_count {
            for offset in 0..0x8000 {
                prg_rom[bank * 0x8000 + offset] = bank as u8;
            }
        }
        prg_rom
    }
    /// CHR ROM with `bank_count` 8 KB banks, each filled with `0xA0 + bank`.
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
        let prg_rom = prg_rom_with_banks(4);
        let chr_rom = chr_rom_with_banks(4);
        let mut mapper = Gxrom::new(prg_rom, chr_rom, Mirroring::Vertical);
        assert_eq!(mapper.cpu_read(0x8000), 0);
        assert_eq!(mapper.cpu_read(0x9ABC), 0);
        assert_eq!(mapper.cpu_read(0xFFFF), 0);
    }
    #[test]
    fn cpu_write_selects_32kb_prg_bank() {
        let prg_rom = prg_rom_with_banks(4);
        let chr_rom = chr_rom_with_banks(4);
        let mut mapper = Gxrom::new(prg_rom, chr_rom, Mirroring::Vertical);
        mapper.cpu_write(0x8000, 0x10);
        assert_eq!(mapper.cpu_read(0x8000), 1);
        assert_eq!(mapper.cpu_read(0xFFFF), 1);
        mapper.cpu_write(0xFFFF, 0x20);
        assert_eq!(mapper.cpu_read(0x8000), 2);
        mapper.cpu_write(0x8000, 0x30);
        assert_eq!(mapper.cpu_read(0x8000), 3);
    }
    #[test]
    fn cpu_write_selects_8kb_chr_bank() {
        let prg_rom = prg_rom_with_banks(4);
        let chr_rom = chr_rom_with_banks(4);
        let mut mapper = Gxrom::new(prg_rom, chr_rom, Mirroring::Vertical);
        assert_eq!(mapper.ppu_read(0x0000), 0xA0);
        mapper.cpu_write(0x8000, 0x01);
        assert_eq!(mapper.ppu_read(0x0000), 0xA1);
        assert_eq!(mapper.ppu_read(0x1FFF), 0xA1);
        mapper.cpu_write(0x8000, 0x03);
        assert_eq!(mapper.ppu_read(0x0000), 0xA3);
    }
    #[test]
    fn chr_bank_wraps_to_rom_size() {
        let prg_rom = prg_rom_with_banks(2);
        let chr_rom = chr_rom_with_banks(2);
        let mut mapper = Gxrom::new(prg_rom, chr_rom, Mirroring::Vertical);

        // Only two 8KB CHR banks exist; register value 3 must wrap to bank 1.
        mapper.cpu_write(0x8000, 0x03);

        assert_eq!(mapper.chr_bank, 1);
        assert_eq!(mapper.ppu_read(0x0000), 0xA1);
    }

    #[test]
    fn cpu_write_sets_prg_and_chr_from_one_register() {
        let prg_rom = prg_rom_with_banks(4);
        let chr_rom = chr_rom_with_banks(4);
        let mut mapper = Gxrom::new(prg_rom, chr_rom, Mirroring::Vertical);
        // bits 4-5 = PRG bank 2, bits 0-1 = CHR bank 1
        mapper.cpu_write(0x8000, 0x21);
        assert_eq!(mapper.cpu_read(0x8000), 2);
        assert_eq!(mapper.ppu_read(0x0000), 0xA1);
    }
    #[test]
    fn cpu_write_ignores_bits_outside_the_two_bank_fields() {
        let prg_rom = prg_rom_with_banks(4);
        let chr_rom = chr_rom_with_banks(4);
        let mut mapper = Gxrom::new(prg_rom, chr_rom, Mirroring::Vertical);
        // 0x44 has bits 2 and 6 set, which belong to neither bank field.
        mapper.cpu_write(0x8000, 0x44);
        assert_eq!(mapper.cpu_read(0x8000), 0);
        assert_eq!(mapper.ppu_read(0x0000), 0xA0);
    }
    #[test]
    fn selected_prg_bank_wraps_when_bank_number_exceeds_prg_size() {
        let prg_rom = prg_rom_with_banks(2);
        let chr_rom = chr_rom_with_banks(2);
        let mut mapper = Gxrom::new(prg_rom, chr_rom, Mirroring::Vertical);
        // Only two 32 KB banks, but the register asks for bank 2.
        mapper.cpu_write(0x8000, 0x20);
        assert_eq!(mapper.cpu_read(0x8000), 0);
    }
    #[test]
    fn cpu_read_below_8000_returns_zero() {
        let prg_rom = prg_rom_with_banks(4);
        let chr_rom = chr_rom_with_banks(4);
        let mut mapper = Gxrom::new(prg_rom, chr_rom, Mirroring::Vertical);
        assert_eq!(mapper.cpu_read(0x0000), 0);
        assert_eq!(mapper.cpu_read(0x6000), 0);
        assert_eq!(mapper.cpu_read(0x7FFF), 0);
    }
    #[test]
    fn ppu_read_outside_chr_returns_zero() {
        let prg_rom = prg_rom_with_banks(4);
        let chr_rom = chr_rom_with_banks(1);
        let mut mapper = Gxrom::new(prg_rom, chr_rom, Mirroring::Vertical);
        assert_eq!(mapper.ppu_read(0x1FFF), 0xA0);
        assert_eq!(mapper.ppu_read(0x2000), 0);
        assert_eq!(mapper.ppu_read(0x4000), 0);
    }
    #[test]
    fn ppu_write_is_noop() {
        let prg_rom = prg_rom_with_banks(4);
        let chr_rom = chr_rom_with_banks(2);
        let mut mapper = Gxrom::new(prg_rom, chr_rom, Mirroring::Vertical);
        mapper.ppu_write(0x0000, 0x42);
        assert_eq!(mapper.ppu_read(0x0000), 0xA0);
    }
    #[test]
    fn mirroring_comes_from_cartridge() {
        let prg_rom = prg_rom_with_banks(4);
        let chr_rom = chr_rom_with_banks(4);
        let mapper = Gxrom::new(prg_rom.clone(), chr_rom.clone(), Mirroring::Horizontal);
        assert_eq!(mapper.mirroring(), Mirroring::Horizontal);
        let mapper = Gxrom::new(prg_rom.clone(), chr_rom.clone(), Mirroring::Vertical);
        assert_eq!(mapper.mirroring(), Mirroring::Vertical);
        let mapper = Gxrom::new(prg_rom, chr_rom, Mirroring::FourScreen);
        assert_eq!(mapper.mirroring(), Mirroring::FourScreen);
    }
}
