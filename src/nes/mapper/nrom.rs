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

#[cfg(test)]
mod tests {
    use super::*;

    fn prg_rom_32kb() -> Vec<u8> {
        let mut prg = vec![0; 0x8000];
        for (i, byte) in prg.iter_mut().enumerate() {
            *byte = (i >> 8) as u8;
        }
        prg
    }

    #[test]
    fn cpu_read_32kb_prg_maps_8000_to_ffff() {
        let mut prg = prg_rom_32kb();
        prg[0] = 0xAA;
        prg[0x7FFF] = 0xBB;
        let chr = vec![0; 0x2000];
        let mut mapper = Nrom::new(prg.clone(), chr, Mirroring::Horizontal);

        assert_eq!(mapper.cpu_read(0x8000), 0xAA);
        assert_eq!(mapper.cpu_read(0xC000), prg[0x4000]);
        assert_eq!(mapper.cpu_read(0xFFFF), 0xBB);
    }

    #[test]
    fn cpu_read_16kb_prg_is_mirrored_across_window() {
        let mut prg = vec![0; 0x4000];
        prg[0] = 0xAA;
        prg[0x3FFF] = 0xBB;
        let chr = vec![0; 0x2000];
        let mut mapper = Nrom::new(prg, chr, Mirroring::Horizontal);

        assert_eq!(mapper.cpu_read(0x8000), 0xAA);
        assert_eq!(mapper.cpu_read(0xBFFF), 0xBB);
        // 16KB PRG wraps: $C000-$FFFF mirrors $8000-$BFFF.
        assert_eq!(mapper.cpu_read(0xC000), 0xAA);
        assert_eq!(mapper.cpu_read(0xFFFF), 0xBB);
    }

    #[test]
    fn cpu_read_below_8000_returns_zero() {
        let prg = prg_rom_32kb();
        let chr = vec![0; 0x2000];
        let mut mapper = Nrom::new(prg, chr, Mirroring::Horizontal);

        assert_eq!(mapper.cpu_read(0x0000), 0);
        assert_eq!(mapper.cpu_read(0x6000), 0);
        assert_eq!(mapper.cpu_read(0x7FFF), 0);
    }

    #[test]
    fn cpu_write_is_ignored() {
        let mut prg = vec![0; 0x4000];
        prg[0] = 0xAA;
        let chr = vec![0; 0x2000];
        let mut mapper = Nrom::new(prg, chr, Mirroring::Horizontal);

        mapper.cpu_write(0x8000, 0x55);

        assert_eq!(mapper.cpu_read(0x8000), 0xAA);
    }

    #[test]
    fn ppu_read_reads_chr_rom() {
        let prg = vec![0; 0x4000];
        let mut chr = vec![0; 0x2000];
        chr[0] = 0x11;
        chr[0x1FFF] = 0x22;
        let mut mapper = Nrom::new(prg, chr, Mirroring::Horizontal);

        assert_eq!(mapper.ppu_read(0x0000), 0x11);
        assert_eq!(mapper.ppu_read(0x1FFF), 0x22);
    }

    #[test]
    fn ppu_read_outside_chr_returns_zero() {
        let prg = vec![0; 0x4000];
        let mut chr = vec![0; 0x1000];
        chr[0] = 0x11;
        let mut mapper = Nrom::new(prg, chr, Mirroring::Horizontal);

        assert_eq!(mapper.ppu_read(0x0000), 0x11);
        // Inside the pattern-table range but past the end of CHR ROM.
        assert_eq!(mapper.ppu_read(0x1000), 0);
        // Outside the pattern-table range.
        assert_eq!(mapper.ppu_read(0x2000), 0);
    }

    #[test]
    fn ppu_write_is_ignored() {
        let prg = vec![0; 0x4000];
        let mut chr = vec![0; 0x2000];
        chr[0] = 0x11;
        let mut mapper = Nrom::new(prg, chr, Mirroring::Horizontal);

        mapper.ppu_write(0x0000, 0x42);

        assert_eq!(mapper.ppu_read(0x0000), 0x11);
    }

    #[test]
    fn mirroring_comes_from_cartridge() {
        let prg = vec![0; 0x4000];
        let chr = vec![0; 0x2000];

        let mapper = Nrom::new(prg.clone(), chr.clone(), Mirroring::Horizontal);
        assert_eq!(mapper.mirroring(), Mirroring::Horizontal);

        let mapper = Nrom::new(prg.clone(), chr.clone(), Mirroring::Vertical);
        assert_eq!(mapper.mirroring(), Mirroring::Vertical);

        let mapper = Nrom::new(prg, chr, Mirroring::FourScreen);
        assert_eq!(mapper.mirroring(), Mirroring::FourScreen);
    }
}
