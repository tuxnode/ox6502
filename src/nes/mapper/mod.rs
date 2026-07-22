use crate::nes::cartridge::{Cartridge, Mirroring};
use crate::nes::mapper::mmc1::Mmc1;
use crate::nes::mapper::nrom::Nrom;
use crate::nes::mapper::uxrom::Uxrom;

pub trait Mapper {
    fn cpu_read(&mut self, addr: u16) -> u8;
    fn cpu_write(&mut self, addr: u16, val: u8);
    fn ppu_read(&mut self, addr: u16) -> u8;
    fn ppu_write(&mut self, addr: u16, val: u8);
    fn mirroring(&self) -> Mirroring;
}

pub fn from_cartridge(cart: Cartridge) -> Box<dyn Mapper> {
    match cart.mapper {
        0 => Box::new(Nrom::new(cart.prg_rom, cart.chr_rom, cart.mirroring)),
        1 => Box::new(Mmc1::new(cart.prg_rom, cart.chr_rom, cart.mirroring)),
        2 => Box::new(Uxrom::new(cart.prg_rom, cart.chr_rom, cart.mirroring, 0)),
        n => panic!("Unsupported mapper: {}", n),
    }
}

pub mod mmc1;
pub mod nrom;
pub mod uxrom;
