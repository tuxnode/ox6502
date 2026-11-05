use crate::nes::cartridge::{Cartridge, Mirroring};
use crate::nes::mapper::axrom::Axrom;
use crate::nes::mapper::cnrom::Cnrom;
use crate::nes::mapper::colordreams::ColorDreams;
use crate::nes::mapper::gxrom::Gxrom;
use crate::nes::mapper::mmc1::Mmc1;
use crate::nes::mapper::mmc3::Mmc3;
use crate::nes::mapper::nrom::Nrom;
use crate::nes::mapper::uxrom::Uxrom;

pub trait Mapper {
    fn cpu_read(&mut self, addr: u16) -> u8;
    fn cpu_write(&mut self, addr: u16, val: u8);
    fn ppu_read(&mut self, addr: u16) -> u8;
    fn ppu_write(&mut self, addr: u16, val: u8);
    fn mirroring(&self) -> Mirroring;

    /// Clock the mapper's scanline IRQ counter (MMC3). Called once per PPU
    /// scanline when rendering is active.
    fn clock_scanline(&mut self) {}

    /// Whether the mapper is currently asserting its IRQ line (MMC3).
    fn irq_pending(&self) -> bool {
        false
    }
}

pub fn from_cartridge(cart: Cartridge) -> Box<dyn Mapper> {
    match cart.mapper {
        0 => Box::new(Nrom::new(cart.prg_rom, cart.chr_rom, cart.mirroring)),
        1 => Box::new(Mmc1::new(cart.prg_rom, cart.chr_rom, cart.mirroring)),
        2 => Box::new(Uxrom::new(cart.prg_rom, cart.chr_rom, cart.mirroring, 0)),
        3 => Box::new(Cnrom::new(cart.prg_rom, cart.chr_rom, cart.mirroring)),
        4 => Box::new(Mmc3::new(cart.prg_rom, cart.chr_rom, cart.mirroring)),
        7 => Box::new(Axrom::new(cart.prg_rom, cart.chr_rom, cart.mirroring)),
        11 => Box::new(ColorDreams::new(cart.prg_rom, cart.chr_rom, cart.mirroring)),
        66 => Box::new(Gxrom::new(cart.prg_rom, cart.chr_rom, cart.mirroring)),
        n => panic!("Unsupported mapper: {}", n),
    }
}

pub mod axrom;
pub mod cnrom;
pub mod colordreams;
pub mod gxrom;
pub mod mmc1;
pub mod mmc3;
pub mod nrom;
pub mod uxrom;

#[cfg(test)]
mod tests {
    use super::*;

    fn cartridge(mapper: u8) -> Cartridge {
        Cartridge {
            mapper,
            mirroring: Mirroring::Horizontal,
            battery: false,
            trainer: false,
            prg_rom: vec![0; 0x10000],
            chr_rom: vec![0; 0x4000],
        }
    }

    #[test]
    fn from_cartridge_dispatches_axrom() {
        let mut cart = cartridge(7);
        for bank in 0..2 {
            for offset in 0..0x8000 {
                cart.prg_rom[bank * 0x8000 + offset] = bank as u8;
            }
        }

        let mut mapper = from_cartridge(cart);

        assert_eq!(mapper.mirroring(), Mirroring::OneScreenA);
        assert_eq!(mapper.cpu_read(0x8000), 0);

        mapper.cpu_write(0x8000, 0x11);
        assert_eq!(mapper.mirroring(), Mirroring::OneScreenB);
        assert_eq!(mapper.cpu_read(0x8000), 1);
    }

    #[test]
    fn from_cartridge_dispatches_color_dreams() {
        let mut cart = cartridge(11);
        for bank in 0..2 {
            for offset in 0..0x8000 {
                cart.prg_rom[bank * 0x8000 + offset] = bank as u8;
            }
            for offset in 0..0x2000 {
                cart.chr_rom[bank * 0x2000 + offset] = 0xA0 + bank as u8;
            }
        }

        let mut mapper = from_cartridge(cart);

        mapper.cpu_write(0x8000, 0x21);

        assert_eq!(mapper.cpu_read(0x8000), 1);
        // CHR bank 2 wraps modulo the two banks present.
        assert_eq!(mapper.ppu_read(0x0000), 0xA0);
    }
}
