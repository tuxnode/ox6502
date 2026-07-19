use crate::nes::cartridge::Mirroring;
use crate::nes::mapper::Mapper;

pub struct Mmc1 {
    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,
    mirroring: Mirroring,
    /// Whether the cartridge has CHR RAM instead of ROM
    chr_ram: bool,

    // Serial load register
    shift: u8,
    counter: u8,

    // Internal registers
    control: u8,
    chr_bank_0: u8,
    chr_bank_1: u8,
    prg_bank: u8,
}

impl Mmc1 {
    pub fn new(prg_rom: Vec<u8>, chr_rom: Vec<u8>, mirroring: Mirroring) -> Self {
        let chr_ram = chr_rom.len() <= 0x2000 && (chr_rom.len() == 0x2000 || chr_rom.is_empty());
        Self {
            prg_rom,
            chr_rom,
            mirroring,
            chr_ram,
            shift: 0,
            counter: 0,
            control: 0x0C,
            chr_bank_0: 0,
            chr_bank_1: 0,
            prg_bank: 0,
        }
    }

    fn write_register(&mut self, addr: u16, val: u8) {
        if val & 0x80 != 0 {
            // Bit 7 set: reset shift register
            self.shift = 0;
            self.counter = 0;
            self.control |= 0x0C;
            return;
        }

        // Shift in the bit
        self.shift = (self.shift >> 1) | ((val & 1) << 4);
        self.counter += 1;

        if self.counter < 5 {
            return;
        }

        // Execute the command
        match addr & 0xE000 {
            0x8000 => {
                // Control register
                self.control = self.shift;
                self.mirroring = match self.shift & 0x03 {
                    0 => Mirroring::OneScreenA,
                    1 => Mirroring::OneScreenB,
                    2 => Mirroring::Vertical,
                    3 => Mirroring::Horizontal,
                    _ => unreachable!(),
                };
            }
            0xA000 => self.chr_bank_0 = self.shift,
            0xC000 => self.chr_bank_1 = self.shift,
            0xE000 => self.prg_bank = self.shift & 0x0F,
            _ => unreachable!(),
        }

        self.shift = 0;
        self.counter = 0;
    }

    fn prg_addr(&self, addr: u16) -> usize {
        let prg_mode = (self.control >> 2) & 1;
        let prg_size = self.prg_rom.len();

        match addr {
            0x8000..=0xBFFF => {
                let bank = if prg_mode == 0 {
                    // 32KB mode: ignore low bit
                    (self.prg_bank >> 1) & 0x0F
                } else {
                    // 16KB mode: fixed at $8000 or switchable
                    if (self.control >> 3) & 1 == 0 {
                        // Fixed $8000 = first bank
                        0
                    } else {
                        // Switchable $8000
                        self.prg_bank
                    }
                };
                let bank = (bank as usize) % (prg_size / 0x4000).max(1);
                bank * 0x4000 + (addr - 0x8000) as usize
            }
            0xC000..=0xFFFF => {
                let bank = if prg_mode == 0 {
                    // 32KB mode: low bit OR'd in
                    ((self.prg_bank >> 1) | 0x01) & 0x0F
                } else {
                    if (self.control >> 3) & 1 == 0 {
                        // Switchable at $C000
                        self.prg_bank
                    } else {
                        // Fixed $C000 = last bank
                        (prg_size / 0x4000).wrapping_sub(1) as u8
                    }
                };
                let bank = (bank as usize) % (prg_size / 0x4000).max(1);
                bank * 0x4000 + (addr - 0xC000) as usize
            }
            _ => 0,
        }
    }

    fn chr_addr(&self, addr: u16) -> usize {
        let chr_mode = (self.control >> 4) & 1;
        if chr_mode == 0 {
            // 8KB mode
            let bank = (self.chr_bank_0 >> 1) & 0xFE;
            let bank = bank as usize;
            bank * 0x1000 + addr as usize
        } else {
            // 4KB mode
            match addr {
                0x0000..=0x0FFF => {
                    self.chr_bank_0 as usize * 0x1000 + addr as usize
                }
                0x1000..=0x1FFF => {
                    self.chr_bank_1 as usize * 0x1000 + (addr - 0x1000) as usize
                }
                _ => 0,
            }
        }
    }
}

impl Mapper for Mmc1 {
    fn cpu_read(&mut self, addr: u16) -> u8 {
        match addr {
            0x6000..=0x7FFF => 0,
            0x8000..=0xFFFF => {
                let index = self.prg_addr(addr);
                if index < self.prg_rom.len() {
                    self.prg_rom[index]
                } else {
                    0
                }
            }
            _ => 0,
        }
    }

    fn cpu_write(&mut self, addr: u16, val: u8) {
        match addr {
            0x6000..=0x7FFF => {}
            0x8000..=0xFFFF => self.write_register(addr, val),
            _ => {}
        }
    }

    fn ppu_read(&mut self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1FFF => {
                let index = self.chr_addr(addr);
                if index < self.chr_rom.len() {
                    self.chr_rom[index]
                } else {
                    0
                }
            }
            _ => 0,
        }
    }

    fn ppu_write(&mut self, addr: u16, val: u8) {
        if self.chr_ram && matches!(addr, 0x0000..=0x1FFF) {
            let index = self.chr_addr(addr);
            if index < self.chr_rom.len() {
                self.chr_rom[index] = val;
            }
        }
    }

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}
