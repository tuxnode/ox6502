use crate::nes::cartridge::{Cartridge, Mirroring};
use crate::nes::mapper::cnrom::Cnrom;
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
        66 => Box::new(Gxrom::new(cart.prg_rom, cart.chr_rom, cart.mirroring)),
        n => panic!("Unsupported mapper: {}", n),
    }
}

pub mod cnrom;
pub mod gxrom;
pub mod mmc1;
pub mod mmc3;
pub mod nrom;
pub mod uxrom;
