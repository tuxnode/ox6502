use crate::nes::cartridge::Mirroring;
use crate::nes::mapper::Mapper;

/**
 * MMC3 (mapper 4)
 * Reference: https://www.nesdev.org/wiki/MMC3
 *
 * 8KB PRG banks, 1KB CHR banks, configurable mirroring and a scanline IRQ
 * counter clocked by PPU A12 rising edges (approximated at PPU dot 260).
 */
pub struct Mmc3 {
    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,
    chr_ram: bool,
    mirroring: Mirroring,

    bank_select: u8,
    registers: [u8; 8],

    irq_latch: u8,
    irq_counter: u8,
    irq_reload: bool,
    irq_enabled: bool,
    irq_pending: bool,
}

impl Mmc3 {
    pub fn new(prg_rom: Vec<u8>, chr_rom: Vec<u8>, mirroring: Mirroring) -> Self {
        let chr_ram = chr_rom.len() <= 0x2000;
        Self {
            prg_rom,
            chr_rom,
            chr_ram,
            mirroring,
            bank_select: 0,
            registers: [0; 8],
            irq_latch: 0,
            irq_counter: 0,
            irq_reload: false,
            irq_enabled: false,
            irq_pending: false,
        }
    }

    fn prg_bank(&self, slot: usize) -> usize {
        let banks = (self.prg_rom.len() / 0x2000).max(1);
        let r6 = self.registers[6] as usize;
        let r7 = self.registers[7] as usize;
        let mode = (self.bank_select & 0x40) != 0;

        let bank = match (mode, slot) {
            // Mode 0: R6 at $8000, R7 at $A000, fixed -2 at $C000, -1 at $E000.
            (false, 0) => r6,
            (false, 1) => r7,
            (false, 2) => banks.saturating_sub(2),
            (false, 3) => banks.saturating_sub(1),
            // Mode 1: fixed -2 at $8000, R7 at $A000, R6 at $C000, -1 at $E000.
            (true, 0) => banks.saturating_sub(2),
            (true, 1) => r7,
            (true, 2) => r6,
            (true, 3) => banks.saturating_sub(1),
            _ => 0,
        };

        bank % banks
    }

    /// Returns the 1KB CHR bank and the offset inside it for a PPU address.
    fn chr_bank_and_offset(&self, addr: u16) -> (usize, usize) {
        let mode = (self.bank_select & 0x80) != 0;
        let r = &self.registers;

        let (bank, offset) = if !mode {
            match addr {
                0x0000..=0x07FF => ((r[0] as usize) & !1, addr as usize),
                0x0800..=0x0FFF => ((r[1] as usize) & !1, (addr - 0x0800) as usize),
                0x1000..=0x13FF => (r[2] as usize, (addr - 0x1000) as usize),
                0x1400..=0x17FF => (r[3] as usize, (addr - 0x1400) as usize),
                0x1800..=0x1BFF => (r[4] as usize, (addr - 0x1800) as usize),
                0x1C00..=0x1FFF => (r[5] as usize, (addr - 0x1C00) as usize),
                _ => (0, 0),
            }
        } else {
            match addr {
                0x0000..=0x03FF => (r[2] as usize, addr as usize),
                0x0400..=0x07FF => (r[3] as usize, (addr - 0x0400) as usize),
                0x0800..=0x0BFF => (r[4] as usize, (addr - 0x0800) as usize),
                0x0C00..=0x0FFF => (r[5] as usize, (addr - 0x0C00) as usize),
                0x1000..=0x17FF => ((r[0] as usize) & !1, (addr - 0x1000) as usize),
                0x1800..=0x1FFF => ((r[1] as usize) & !1, (addr - 0x1800) as usize),
                _ => (0, 0),
            }
        };

        (bank, offset)
    }

    fn chr_addr(&self, addr: u16) -> usize {
        let (bank, offset) = self.chr_bank_and_offset(addr);
        (bank * 0x400 + offset) % self.chr_rom.len().max(1)
    }
}

impl Mapper for Mmc3 {
    fn cpu_read(&mut self, addr: u16) -> u8 {
        match addr {
            0x8000..=0xFFFF => {
                let slot = ((addr - 0x8000) / 0x2000) as usize;
                let bank = self.prg_bank(slot);
                let offset = bank * 0x2000 + (addr as usize & 0x1FFF);
                self.prg_rom.get(offset).copied().unwrap_or(0)
            }
            _ => 0,
        }
    }

    fn cpu_write(&mut self, addr: u16, val: u8) {
        match addr & 0xE001 {
            // Bank select
            0x8000 => self.bank_select = val,
            // Bank data
            0x8001 => {
                let index = (self.bank_select & 0x07) as usize;
                self.registers[index] = val;
            }
            // Mirroring: bit 0 clear = vertical, set = horizontal
            0xA000 => {
                self.mirroring = if val & 0x01 != 0 {
                    Mirroring::Horizontal
                } else {
                    Mirroring::Vertical
                };
            }
            // PRG RAM protect; the bus always provides 8KB of battery RAM.
            0xA001 => {}
            // IRQ latch
            0xC000 => self.irq_latch = val,
            // IRQ reload
            0xC001 => self.irq_reload = true,
            // IRQ disable + acknowledge
            0xE000 => {
                self.irq_enabled = false;
                self.irq_pending = false;
            }
            // IRQ enable
            0xE001 => self.irq_enabled = true,
            _ => {}
        }
    }

    fn ppu_read(&mut self, addr: u16) -> u8 {
        self.chr_rom.get(self.chr_addr(addr)).copied().unwrap_or(0)
    }

    fn ppu_write(&mut self, addr: u16, val: u8) {
        if self.chr_ram {
            let index = self.chr_addr(addr);
            if index < self.chr_rom.len() {
                self.chr_rom[index] = val;
            }
        }
    }

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }

    fn clock_scanline(&mut self) {
        if self.irq_counter == 0 || self.irq_reload {
            self.irq_counter = self.irq_latch;
            self.irq_reload = false;
        } else {
            self.irq_counter -= 1;
        }

        if self.irq_counter == 0 && self.irq_enabled {
            self.irq_pending = true;
        }
    }

    fn irq_pending(&self) -> bool {
        self.irq_pending
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn prg_rom_with_banks(bank_count: usize) -> Vec<u8> {
        let mut prg = vec![0; bank_count * 0x2000];
        for bank in 0..bank_count {
            for offset in 0..0x2000 {
                prg[bank * 0x2000 + offset] = bank as u8;
            }
        }
        prg
    }

    fn chr_rom_with_banks(bank_count: usize) -> Vec<u8> {
        let mut chr = vec![0; bank_count * 0x400];
        for bank in 0..bank_count {
            for offset in 0..0x400 {
                chr[bank * 0x400 + offset] = 0xA0 + bank as u8;
            }
        }
        chr
    }

    fn set_bank(mapper: &mut Mmc3, register: u8, bank: u8) {
        mapper.cpu_write(0x8000, (mapper.bank_select & 0xC0) | (register & 0x07));
        mapper.cpu_write(0x8001, bank);
    }

    #[test]
    fn prg_mode_zero_maps_switchable_and_fixed_last_banks() {
        let mut mapper = Mmc3::new(
            prg_rom_with_banks(8),
            chr_rom_with_banks(8),
            Mirroring::Horizontal,
        );

        set_bank(&mut mapper, 6, 2);
        set_bank(&mut mapper, 7, 3);

        assert_eq!(mapper.cpu_read(0x8000), 2);
        assert_eq!(mapper.cpu_read(0xA000), 3);
        assert_eq!(mapper.cpu_read(0xC000), 6);
        assert_eq!(mapper.cpu_read(0xE000), 7);
    }

    #[test]
    fn prg_mode_one_swaps_switchable_windows() {
        let mut mapper = Mmc3::new(
            prg_rom_with_banks(8),
            chr_rom_with_banks(8),
            Mirroring::Horizontal,
        );

        mapper.cpu_write(0x8000, 0x40);
        set_bank(&mut mapper, 6, 1);
        set_bank(&mut mapper, 7, 2);

        assert_eq!(mapper.cpu_read(0x8000), 6);
        assert_eq!(mapper.cpu_read(0xA000), 2);
        assert_eq!(mapper.cpu_read(0xC000), 1);
        assert_eq!(mapper.cpu_read(0xE000), 7);
    }

    #[test]
    fn chr_mode_zero_uses_two_kb_pairs_below_1000() {
        let mut mapper = Mmc3::new(
            prg_rom_with_banks(2),
            chr_rom_with_banks(16),
            Mirroring::Horizontal,
        );

        set_bank(&mut mapper, 0, 4);
        set_bank(&mut mapper, 1, 6);
        set_bank(&mut mapper, 2, 8);
        set_bank(&mut mapper, 3, 9);
        set_bank(&mut mapper, 4, 10);
        set_bank(&mut mapper, 5, 11);

        // R0 bit 0 is ignored for the 2KB window.
        assert_eq!(mapper.ppu_read(0x0000), 0xA4);
        assert_eq!(mapper.ppu_read(0x0400), 0xA5);
        assert_eq!(mapper.ppu_read(0x0800), 0xA6);
        assert_eq!(mapper.ppu_read(0x0C00), 0xA7);
        assert_eq!(mapper.ppu_read(0x1000), 0xA8);
        assert_eq!(mapper.ppu_read(0x1400), 0xA9);
        assert_eq!(mapper.ppu_read(0x1800), 0xAA);
        assert_eq!(mapper.ppu_read(0x1C00), 0xAB);
    }

    #[test]
    fn chr_mode_one_swaps_one_kb_and_two_kb_windows() {
        let mut mapper = Mmc3::new(
            prg_rom_with_banks(2),
            chr_rom_with_banks(16),
            Mirroring::Horizontal,
        );

        mapper.cpu_write(0x8000, 0x80);
        set_bank(&mut mapper, 0, 4);
        set_bank(&mut mapper, 1, 6);
        set_bank(&mut mapper, 2, 8);
        set_bank(&mut mapper, 3, 9);
        set_bank(&mut mapper, 4, 10);
        set_bank(&mut mapper, 5, 11);

        assert_eq!(mapper.ppu_read(0x0000), 0xA8);
        assert_eq!(mapper.ppu_read(0x0400), 0xA9);
        assert_eq!(mapper.ppu_read(0x0800), 0xAA);
        assert_eq!(mapper.ppu_read(0x0C00), 0xAB);
        assert_eq!(mapper.ppu_read(0x1000), 0xA4);
        assert_eq!(mapper.ppu_read(0x1400), 0xA5);
        assert_eq!(mapper.ppu_read(0x1800), 0xA6);
        assert_eq!(mapper.ppu_read(0x1C00), 0xA7);
    }

    #[test]
    fn mirroring_register_selects_vertical_or_horizontal() {
        let mut mapper = Mmc3::new(
            prg_rom_with_banks(2),
            chr_rom_with_banks(8),
            Mirroring::Horizontal,
        );

        mapper.cpu_write(0xA000, 0);
        assert_eq!(mapper.mirroring(), Mirroring::Vertical);
        mapper.cpu_write(0xA000, 1);
        assert_eq!(mapper.mirroring(), Mirroring::Horizontal);
    }

    #[test]
    fn irq_counter_reloads_counts_down_and_acknowledges() {
        let mut mapper = Mmc3::new(
            prg_rom_with_banks(2),
            chr_rom_with_banks(8),
            Mirroring::Horizontal,
        );

        mapper.cpu_write(0xC000, 2); // latch
        mapper.cpu_write(0xC001, 0); // reload
        mapper.cpu_write(0xE001, 0); // enable

        mapper.clock_scanline(); // reload -> 2
        assert!(!mapper.irq_pending());
        mapper.clock_scanline(); // 2 -> 1
        assert!(!mapper.irq_pending());
        mapper.clock_scanline(); // 1 -> 0, assert
        assert!(mapper.irq_pending());

        mapper.cpu_write(0xE000, 0); // disable + acknowledge
        assert!(!mapper.irq_pending());
    }

    #[test]
    fn disabled_irq_counter_still_runs_but_does_not_assert() {
        let mut mapper = Mmc3::new(
            prg_rom_with_banks(2),
            chr_rom_with_banks(8),
            Mirroring::Horizontal,
        );

        mapper.cpu_write(0xC000, 1);
        mapper.cpu_write(0xC001, 0);
        mapper.clock_scanline();
        mapper.clock_scanline();
        assert!(!mapper.irq_pending());

        mapper.cpu_write(0xE001, 0);
        mapper.cpu_write(0xC001, 0);
        mapper.clock_scanline();
        mapper.clock_scanline();
        assert!(mapper.irq_pending());
    }

    #[test]
    fn chr_ram_writes_are_visible_through_banks() {
        let mut mapper = Mmc3::new(
            prg_rom_with_banks(2),
            vec![0; 0x2000],
            Mirroring::Horizontal,
        );

        set_bank(&mut mapper, 2, 3);
        mapper.ppu_write(0x1000, 0x5A);

        assert_eq!(mapper.ppu_read(0x1000), 0x5A);
    }
}
