/**
 * NES Dev CPU Memory Map: https://www.nesdev.org/wiki/CPU_memory_map
 * NES Dev PPU Memory Map: https://www.nesdev.org/wiki/PPU_memory_map
 */
use crate::{bus::Bus, nes::cartridge::Cartridge, nes::ppu::Ppu};

pub struct NesBus {
    ram: [u8; 2048],       // 2KB internal RAM
    prg_ram: [u8; 0x2000], // 8KB PRG RAM ($6000-$7FFF)
    cartridge: Cartridge,
    pub ppu: Ppu,          // PPU instance
}

impl NesBus {
    pub fn new(cartridge: Cartridge) -> Self {
        Self {
            ram: [0; 2048],
            prg_ram: [0; 0x2000],
            cartridge,
            ppu: Ppu::new(),
        }
    }
}

impl Bus for NesBus {
    fn cpu_read(&mut self, addr: u16) -> u8 {
        match addr {
            // CPU internal RAM: 2KB, mirrored every $800
            0x0000..=0x1FFF => self.ram[(addr & 0x07FF) as usize],

            // PPU registers: 8 bytes, mirrored every 8 bytes up to $3FFF
            0x2000..=0x3FFF => self.ppu.read_register(addr),

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

            // PPU registers
            0x2000..=0x3FFF => {
                self.ppu.write_register(addr, val);
            }

            // OAM DMA: writing page number triggers 256-byte copy
            0x4014 => {
                let page = val as u16;
                let base = page << 8;
                let mut page_data = [0u8; 256];
                for i in 0..256u16 {
                    page_data[i as usize] = self.cpu_read(base + i);
                }
                self.ppu.dma_write_oam(&page_data);
            }

            // APU and I/O (stub)
            0x4000..=0x4017 => {}

            // APU test mode (stub)
            0x4018..=0x401F => {}

            // Cartridge PRG RAM
            0x6000..=0x7FFF => {
                self.prg_ram[(addr - 0x6000) as usize] = val;
            }

            // PRG ROM is read-only, ignore writes
            0x8000..=0xFFFF => {}

            _ => {}
        }
    }

    fn ppu_read(&mut self, addr: u16) -> u8 {
        0
    }

    fn ppu_write(&mut self, _addr: u16, _val: u8) {}
}
