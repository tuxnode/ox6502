/**
 * NES Dev CPU Memory Map: https://www.nesdev.org/wiki/CPU_memory_map
 * NES Dev PPU Memory Map: https://www.nesdev.org/wiki/PPU_memory_map
 */
use crate::{
    bus::{Bus, TickResult},
    nes::apu::Apu,
    nes::cartridge::Cartridge,
    nes::joypad::Joypad,
    nes::mapper,
    nes::mapper::Mapper,
    nes::ppu::Ppu,
};

pub struct NesBus {
    ram: [u8; 2048],       // 2KB internal RAM
    prg_ram: [u8; 0x2000], // 8KB PRG RAM ($6000-$7FFF)
    pub ppu: Ppu,          // PPU instance
    pub apu: Apu,          // APU instance
    pub joypad1: Joypad,   // Player 1 controller
    joypad2: Joypad,       // Player 2 controller
    mapper: Box<dyn Mapper>,
    dma_cycles: u32,       // DMA pending cycles (0 = no DMA)
}

impl NesBus {
    pub fn new(cart: Cartridge) -> Self {
        let chr_rom = cart.chr_rom.clone();
        Self {
            ram: [0; 2048],
            prg_ram: [0; 0x2000],
            ppu: Ppu::new(chr_rom),
            apu: Apu::new(),
            joypad1: Joypad::new(),
            joypad2: Joypad::new(),
            mapper: mapper::from_cartridge(cart),
            dma_cycles: 0,
        }
    }

    /// Take pending DMA cycles (resets to 0)
    pub fn take_dma_cycles(&mut self) -> u32 {
        let c = self.dma_cycles;
        self.dma_cycles = 0;
        c
    }

    /// Check if PPU has a pending NMI
    pub fn check_nmi(&mut self) -> bool {
        self.ppu.take_nmi()
    }

    /// Advance PPU by the given number of CPU cycles (1 CPU cycle = 3 PPU dots)
    pub fn tick_ppu(&mut self, cpu_cycles: u8) {
        for _ in 0..(cpu_cycles * 3) {
            self.ppu.tick();
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

            // Joypad 1 ($4016)
            0x4016 => self.joypad1.read(),
            // Joypad 2 ($4017)
            0x4017 => self.joypad2.read(),
            // APU status ($4015)
            0x4015 => self.apu.read_status(),
            // Other I/O registers (stub)
            0x4000..=0x4014 | 0x4018..=0x401F => 0,

            // Cartridge PRG RAM ($6000-$7FFF)
            0x6000..=0x7FFF => self.prg_ram[(addr - 0x6000) as usize],

            // Cartridge PRG ROM via mapper ($8000-$FFFF)
            0x8000..=0xFFFF => self.mapper.cpu_read(addr),

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
                // DMA takes 513 cycles if OAMADDR even, 514 if odd
                self.dma_cycles = if self.ppu.oam_addr.is_multiple_of(2) { 513 } else { 514 };
            }

            // Joypad strobe ($4016 write)
            0x4016 => self.joypad1.write(val),

            // APU registers ($4000-$4013)
            0x4000 => self.apu.pulse1.write_reg0(val),
            0x4001 => self.apu.pulse1.write_reg1(val),
            0x4002 => self.apu.pulse1.write_reg2(val),
            0x4003 => self.apu.pulse1.write_reg3(val),
            0x4004 => self.apu.pulse2.write_reg0(val),
            0x4005 => self.apu.pulse2.write_reg1(val),
            0x4006 => self.apu.pulse2.write_reg2(val),
            0x4007 => self.apu.pulse2.write_reg3(val),
            0x4008 => self.apu.triangle.write_reg0(val),
            // 0x4009 is unused
            0x400A => self.apu.triangle.write_reg2(val),
            0x400B => self.apu.triangle.write_reg3(val),
            0x400C => self.apu.noise.write_reg0(val),
            // 0x400D is unused
            0x400E => self.apu.noise.write_reg1(val),
            0x400F => self.apu.noise.write_reg2(val),
            // 0x4010-$4013: DMC (stub)
            0x4010..=0x4013 => {}

            // APU status ($4015)
            0x4015 => self.apu.write_status(val),
            // APU frame counter ($4017)
            0x4017 => self.apu.write_frame_counter(val),

            // Other I/O registers (stub)
            0x4018..=0x401F => {}

            // Cartridge PRG RAM
            0x6000..=0x7FFF => {
                self.prg_ram[(addr - 0x6000) as usize] = val;
            }

            // Cartridge PRG ROM via mapper
            0x8000..=0xFFFF => self.mapper.cpu_write(addr, val),

            _ => {}
        }
    }

    fn ppu_read(&mut self, addr: u16) -> u8 {
        // Pattern tables come from mapper; everything else from PPU internal
        match addr {
            0x0000..=0x1FFF => self.mapper.ppu_read(addr),
            _ => self.ppu.ppu_read(addr),
        }
    }

    fn ppu_write(&mut self, addr: u16, val: u8) {
        match addr {
            0x0000..=0x1FFF => self.mapper.ppu_write(addr, val),
            _ => self.ppu.ppu_write(addr, val),
        }
    }

    fn tick(&mut self, cpu_cycles: u8) -> TickResult {
        // Sync mirroring from mapper to PPU
        self.ppu.set_mirroring(self.mapper.mirroring());
        // Advance PPU first (may set nmi_pending when entering VBlank)
        self.tick_ppu(cpu_cycles);
        // Clock APU
        for _ in 0..cpu_cycles {
            self.apu.tick();
        }
        // Then check for NMI
        let nmi = self.check_nmi();
        TickResult {
            extra_cycles: self.take_dma_cycles(),
            nmi,
        }
    }
}
