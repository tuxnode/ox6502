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

#[cfg(test)]
mod tests {
    use super::*;

    fn prg_rom_16kb() -> Vec<u8> {
        let mut prg = vec![0; 0x4000];
        for i in 0..0x4000 {
            prg[i] = (i >> 8) as u8;
        }
        prg
    }

    fn prg_rom_32kb() -> Vec<u8> {
        let mut prg = vec![0; 0x8000];
        for i in 0..0x8000 {
            prg[i] = (i >> 8) as u8;
        }
        prg
    }

    fn chr_rom_32kb() -> Vec<u8> {
        let mut chr = vec![0; 0x8000];
        // Fill each 8KB bank with a unique pattern byte
        for bank in 0..4 {
            for i in 0..0x2000 {
                chr[bank * 0x2000 + i] = 0xA0 + bank as u8;
            }
        }
        chr
    }

    #[test]
    fn cpu_read_16kb_prg_maps_8000_to_ffff() {
        let mut prg = vec![0; 0x4000];
        prg[0] = 0xAA;
        prg[0x3FFF] = 0xBB;
        let chr = vec![0; 0x2000];
        let mut mapper = Cnrom::new(prg, chr, Mirroring::Horizontal);

        assert_eq!(mapper.cpu_read(0x8000), 0xAA);
        assert_eq!(mapper.cpu_read(0xBFFF), 0xBB);
        // Mirrored: $C000 wraps to $8000, $FFFF wraps to $BFFF
        assert_eq!(mapper.cpu_read(0xC000), 0xAA);
        assert_eq!(mapper.cpu_read(0xFFFF), 0xBB);
    }

    #[test]
    fn cpu_read_32kb_prg_maps_8000_to_ffff() {
        let prg = prg_rom_32kb();
        let chr = vec![0; 0x2000];
        let mut mapper = Cnrom::new(prg.clone(), chr, Mirroring::Vertical);

        for addr in (0x8000..=0xFFFF).step_by(0x100) {
            let offset = addr - 0x8000;
            assert_eq!(
                mapper.cpu_read(addr),
                prg[offset as usize],
                "addr={:04X}",
                addr
            );
        }
    }

    #[test]
    fn cpu_read_below_8000_returns_zero() {
        let prg = prg_rom_16kb();
        let chr = vec![0; 0x2000];
        let mut mapper = Cnrom::new(prg, chr, Mirroring::Horizontal);

        assert_eq!(mapper.cpu_read(0x0000), 0);
        assert_eq!(mapper.cpu_read(0x6000), 0);
        assert_eq!(mapper.cpu_read(0x7FFF), 0);
    }

    #[test]
    fn cpu_write_selects_chr_bank() {
        let prg = prg_rom_16kb();
        let chr = chr_rom_32kb();
        let mut mapper = Cnrom::new(prg, chr, Mirroring::Horizontal);

        assert_eq!(mapper.chr_bank, 0, "initial bank = 0");

        mapper.cpu_write(0x8000, 0x01);
        assert_eq!(mapper.chr_bank, 1);

        mapper.cpu_write(0xFFFF, 0x03);
        assert_eq!(mapper.chr_bank, 3);
    }

    #[test]
    fn cpu_write_masks_to_low_2_bits() {
        let prg = prg_rom_16kb();
        let chr = chr_rom_32kb();
        let mut mapper = Cnrom::new(prg, chr, Mirroring::Horizontal);

        mapper.cpu_write(0x8000, 0xFF);
        assert_eq!(mapper.chr_bank, 3);

        mapper.cpu_write(0x8000, 0xAA);
        assert_eq!(mapper.chr_bank, 2);
    }

    #[test]
    fn ppu_read_uses_selected_chr_bank() {
        let prg = prg_rom_16kb();
        let chr = chr_rom_32kb();
        let mut mapper = Cnrom::new(prg, chr, Mirroring::Horizontal);

        // Bank 0: pattern byte = 0xA0
        assert_eq!(mapper.ppu_read(0x0000), 0xA0);
        assert_eq!(mapper.ppu_read(0x1FFF), 0xA0);

        // Switch to bank 1: pattern byte = 0xA1
        mapper.cpu_write(0x8000, 0x01);
        assert_eq!(mapper.ppu_read(0x0000), 0xA1);
        assert_eq!(mapper.ppu_read(0x1FFF), 0xA1);

        // Bank 2: 0xA2
        mapper.cpu_write(0x8000, 0x02);
        assert_eq!(mapper.ppu_read(0x0000), 0xA2);

        // Bank 3: 0xA3
        mapper.cpu_write(0x8000, 0x03);
        assert_eq!(mapper.ppu_read(0x0000), 0xA3);
    }

    #[test]
    fn ppu_read_outside_chr_returns_zero() {
        let prg = prg_rom_16kb();
        let chr = vec![0xFF; 0x2000];
        let mut mapper = Cnrom::new(prg, chr, Mirroring::Horizontal);

        // Within CHR ROM range
        assert_eq!(mapper.ppu_read(0x1FFF), 0xFF);

        // Beyond CHR ROM size
        assert_eq!(mapper.ppu_read(0x2000), 0);
        assert_eq!(mapper.ppu_read(0x4000), 0);
    }

    #[test]
    fn ppu_write_is_noop() {
        let prg = prg_rom_16kb();
        let chr = vec![0; 0x2000];
        let mut mapper = Cnrom::new(prg, chr, Mirroring::Horizontal);

        mapper.ppu_write(0x0000, 0x42);
        // Should still be 0 (not written)
        assert_eq!(mapper.ppu_read(0x0000), 0);
    }

    #[test]
    fn mirroring_comes_from_cartridge() {
        let prg = prg_rom_16kb();
        let chr = vec![0; 0x2000];

        let mapper = Cnrom::new(prg.clone(), chr.clone(), Mirroring::Horizontal);
        assert_eq!(mapper.mirroring(), Mirroring::Horizontal);

        let mapper = Cnrom::new(prg.clone(), chr.clone(), Mirroring::Vertical);
        assert_eq!(mapper.mirroring(), Mirroring::Vertical);

        let mapper = Cnrom::new(prg, chr, Mirroring::FourScreen);
        assert_eq!(mapper.mirroring(), Mirroring::FourScreen);
    }

    #[test]
    fn prg_read_wraps_for_16kb_prg_in_32kb_window() {
        let mut prg = vec![0; 0x4000];
        prg[0] = 0xAA;
        prg[0x2000] = 0xBB;
        let chr = vec![0; 0x2000];
        let mut mapper = Cnrom::new(prg, chr, Mirroring::Horizontal);

        // 16KB PRG mirrored: $8000 = prg[0], $C000 = prg[0x4000] wraps to prg[0]
        assert_eq!(mapper.cpu_read(0x8000), 0xAA);
        assert_eq!(mapper.cpu_read(0xC000), 0xAA);
        // $A000 = prg[0x2000], $E000 = prg[0x6000] wraps to prg[0x2000]
        assert_eq!(mapper.cpu_read(0xA000), 0xBB);
        assert_eq!(mapper.cpu_read(0xE000), 0xBB);
    }
}
