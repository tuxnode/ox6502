use crate::nes::{cartridge::Mirroring, mapper::Mapper};

/**
 * AxROM (mapper 7)
 * Reference: https://www.nesdev.org/wiki/AxROM
 *
 * 32KB switchable PRG ROM window at $8000-$FFFF, 8KB CHR RAM and
 * single-screen mirroring selected by the bank register.
 */
pub struct Axrom {
    prg_rom: Vec<u8>,
    chr_ram: Vec<u8>,
    chr_writable: bool,
    mirroring: Mirroring,
    bank_select: u8,
}

impl Axrom {
    pub fn new(prg_rom: Vec<u8>, chr_rom: Vec<u8>, _mirroring: Mirroring) -> Self {
        // AxROM boards use CHR RAM. Treat an 8KB (or smaller) CHR store as
        // writable RAM and anything larger as ROM, matching the other mappers.
        let chr_writable = chr_rom.len() <= 0x2000;
        let mut chr_ram = chr_rom;
        if chr_ram.len() < 0x2000 {
            chr_ram.resize(0x2000, 0);
        }

        Self {
            prg_rom,
            chr_ram,
            chr_writable,
            // The bank register powers up as 0, which selects one-screen A.
            mirroring: Mirroring::OneScreenA,
            bank_select: 0,
        }
    }

    fn prg_bank(&self) -> usize {
        let banks = (self.prg_rom.len() / 0x8000).max(1);
        (self.bank_select as usize & 0x07) % banks
    }
}

impl Mapper for Axrom {
    fn cpu_read(&mut self, addr: u16) -> u8 {
        match addr {
            // 32KB switchable PRG ROM window.
            0x8000..=0xFFFF => {
                let offset = self.prg_bank() * 0x8000 + (addr - 0x8000) as usize;
                self.prg_rom.get(offset).copied().unwrap_or(0)
            }
            _ => 0,
        }
    }

    fn cpu_write(&mut self, addr: u16, val: u8) {
        if addr < 0x8000 {
            return;
        }

        // Bits 0-2 select the 32KB PRG bank; bit 4 selects the nametable.
        self.bank_select = val & 0x07;
        self.mirroring = if val & 0x10 != 0 {
            Mirroring::OneScreenB
        } else {
            Mirroring::OneScreenA
        };
    }

    fn ppu_read(&mut self, addr: u16) -> u8 {
        self.chr_ram.get(addr as usize).copied().unwrap_or(0)
    }

    fn ppu_write(&mut self, addr: u16, val: u8) {
        if self.chr_writable && (addr as usize) < self.chr_ram.len() {
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

    fn chr_ram() -> Vec<u8> {
        vec![0; 0x2000]
    }

    #[test]
    fn cpu_read_maps_bank_zero_at_8000_by_default() {
        let mut mapper = Axrom::new(prg_rom_with_banks(4), chr_ram(), Mirroring::Horizontal);

        assert_eq!(mapper.cpu_read(0x8000), 0);
        assert_eq!(mapper.cpu_read(0x9ABC), 0);
        assert_eq!(mapper.cpu_read(0xFFFF), 0);
    }

    #[test]
    fn cpu_write_selects_32kb_prg_bank() {
        let mut mapper = Axrom::new(prg_rom_with_banks(4), chr_ram(), Mirroring::Horizontal);

        mapper.cpu_write(0x8000, 0x01);
        assert_eq!(mapper.cpu_read(0x8000), 1);
        assert_eq!(mapper.cpu_read(0xFFFF), 1);

        mapper.cpu_write(0xFFFF, 0x03);
        assert_eq!(mapper.cpu_read(0x8000), 3);
    }

    #[test]
    fn prg_bank_wraps_when_bank_number_exceeds_rom_size() {
        let mut mapper = Axrom::new(prg_rom_with_banks(2), chr_ram(), Mirroring::Horizontal);

        mapper.cpu_write(0x8000, 0x06);
        assert_eq!(mapper.cpu_read(0x8000), 0);

        mapper.cpu_write(0x8000, 0x07);
        assert_eq!(mapper.cpu_read(0x8000), 1);
    }

    #[test]
    fn bank_register_ignores_bits_three_and_five_to_seven() {
        let mut mapper = Axrom::new(prg_rom_with_banks(8), chr_ram(), Mirroring::Horizontal);

        // Bit 3 belongs to neither the bank nor the mirroring field.
        mapper.cpu_write(0x8000, 0x0A);
        assert_eq!(mapper.cpu_read(0x8000), 2);

        mapper.cpu_write(0x8000, 0xE1);
        assert_eq!(mapper.cpu_read(0x8000), 1);
    }

    #[test]
    fn bit_four_selects_single_screen_mirroring() {
        let mut mapper = Axrom::new(prg_rom_with_banks(2), chr_ram(), Mirroring::Horizontal);

        assert_eq!(mapper.mirroring(), Mirroring::OneScreenA);

        mapper.cpu_write(0x8000, 0x10);
        assert_eq!(mapper.mirroring(), Mirroring::OneScreenB);

        mapper.cpu_write(0x8000, 0x00);
        assert_eq!(mapper.mirroring(), Mirroring::OneScreenA);
    }

    #[test]
    fn cpu_write_below_8000_is_ignored() {
        let mut mapper = Axrom::new(prg_rom_with_banks(2), chr_ram(), Mirroring::Horizontal);

        mapper.cpu_write(0x6000, 0x01);
        mapper.cpu_write(0x7FFF, 0x11);

        assert_eq!(mapper.cpu_read(0x8000), 0);
        assert_eq!(mapper.mirroring(), Mirroring::OneScreenA);
    }

    #[test]
    fn cpu_read_below_8000_returns_zero() {
        let mut mapper = Axrom::new(prg_rom_with_banks(2), chr_ram(), Mirroring::Horizontal);

        assert_eq!(mapper.cpu_read(0x0000), 0);
        assert_eq!(mapper.cpu_read(0x6000), 0);
        assert_eq!(mapper.cpu_read(0x7FFF), 0);
    }

    #[test]
    fn ppu_read_write_accesses_chr_ram() {
        let mut mapper = Axrom::new(prg_rom_with_banks(2), chr_ram(), Mirroring::Horizontal);

        mapper.ppu_write(0x0000, 0x12);
        mapper.ppu_write(0x1FFF, 0x34);

        assert_eq!(mapper.ppu_read(0x0000), 0x12);
        assert_eq!(mapper.ppu_read(0x1FFF), 0x34);
    }

    #[test]
    fn ppu_access_outside_chr_returns_zero() {
        let mut mapper = Axrom::new(prg_rom_with_banks(2), chr_ram(), Mirroring::Horizontal);

        assert_eq!(mapper.ppu_read(0x2000), 0);
        assert_eq!(mapper.ppu_read(0x4000), 0);
    }

    #[test]
    fn chr_rom_is_not_writable() {
        let mut mapper = Axrom::new(
            prg_rom_with_banks(2),
            vec![0xAA; 0x4000],
            Mirroring::Horizontal,
        );

        mapper.ppu_write(0x0000, 0x12);

        assert_eq!(mapper.ppu_read(0x0000), 0xAA);
    }
}
