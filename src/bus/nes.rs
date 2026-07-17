/**
 * NES Dev CPU Memory Map: https://www.nesdev.org/wiki/CPU_memory_map
 * NES Dev PPU Memory Map: https://www.nesdev.org/wiki/PPU_memory_map
 */
use crate::{bus::Bus, nes::cartridge::Cartridge};

pub struct NesBus {
    ram: [u8; 2048],       // 2KB internal RAM
    prg_ram: [u8; 0x2000], // 8KB PRG RAM ($6000-$7FFF)
    cartridge: Cartridge,

    // PPU (stub for now)
    vram: [u8; 2048],
    palette: [u8; 32],
    oam: [u8; 256],
}

impl NesBus {
    pub fn new(cartridge: Cartridge) -> Self {
        Self {
            ram: [0; 2048],
            prg_ram: [0; 0x2000],
            cartridge,
            vram: [0; 2048],
            palette: [0; 32],
            oam: [0; 256],
        }
    }
}

impl Bus for NesBus {
    fn cpu_read(&mut self, addr: u16) -> u8 {
        match addr {
            // CPU internal RAM: 2KB, mirrored every $800
            0x0000..=0x1FFF => self.ram[(addr & 0x07FF) as usize],

            // PPU registers: 8 bytes, mirrored every 8 bytes up to $3FFF
            // PPU not implemented yet, return 0
            0x2000..=0x3FFF => 0,

            // APU and I/O registers
            0x4000..=0x4017 => 0,

            // APU test mode (normally disabled)
            0x4018..=0x401F => 0,

            // Cartridge PRG RAM ($6000-$7FFF)
            0x6000..=0x7FFF => self.prg_ram[(addr - 0x6000) as usize],

            // Cartridge PRG ROM ($8000-$FFFF)
            // NROM: 16KB mirrored to $C000-$FFFF, or 32KB full range
            0x8000..=0xFFFF => {
                let offset = (addr - 0x8000) as usize;
                self.cartridge.prg_rom[offset % self.cartridge.prg_rom.len()]
            }

            _ => 0,
        }
    }

    fn cpu_write(&mut self, addr: u16, val: u8) {
        match addr {
            // CPU internal RAM
            0x0000..=0x1FFF => {
                self.ram[(addr & 0x07FF) as usize] = val;
            }

            // TODO: PPU registers (stub)
            0x2000..=0x3FFF => {}

            // TODO: APU and I/O (stub)
            0x4000..=0x4017 => {}

            // TODO: APU test mode (stub)
            0x4018..=0x401F => {}

            // Cartridge PRG RAM
            0x6000..=0x7FFF => {
                self.prg_ram[(addr - 0x6000) as usize] = val;
            }

            // TODO: PRG ROM is read-only, ignore writes
            0x8000..=0xFFFF => {}

            _ => {}
        }
    }

    fn ppu_read(&mut self, addr: u16) -> u8 {
        0
    }

    fn ppu_write(&mut self, _addr: u16, _val: u8) {
        0
    }
}
