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
            // 8KB mode: the bank number is bits 1-4 (bit 0 is ignored)
            let bank = (self.chr_bank_0 >> 1) as usize;
            bank * 0x2000 + addr as usize
        } else {
            // 4KB mode
            match addr {
                0x0000..=0x0FFF => self.chr_bank_0 as usize * 0x1000 + addr as usize,
                0x1000..=0x1FFF => self.chr_bank_1 as usize * 0x1000 + (addr - 0x1000) as usize,
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Loads a 5-bit value into an MMC1 register using the serial protocol
    /// (least-significant bit first, five writes to the same register address).
    fn load_register(mapper: &mut Mmc1, addr: u16, value: u8) {
        for bit in 0..5 {
            mapper.cpu_write(addr, (value >> bit) & 1);
        }
    }

    fn prg_rom_with_banks(bank_count: usize) -> Vec<u8> {
        let mut prg_rom = vec![0; bank_count * 0x4000];
        for bank in 0..bank_count {
            for offset in 0..0x4000 {
                prg_rom[bank * 0x4000 + offset] = bank as u8;
            }
        }
        prg_rom
    }

    fn chr_rom_8kb() -> Vec<u8> {
        vec![0; 0x2000]
    }

    fn chr_rom_32kb() -> Vec<u8> {
        let mut chr = vec![0; 0x8000];
        for bank in 0..8 {
            for offset in 0..0x1000 {
                chr[bank * 0x1000 + offset] = 0xA0 + bank as u8;
            }
        }
        chr
    }

    #[test]
    fn mirroring_defaults_to_cartridge_mirroring() {
        let mapper = Mmc1::new(prg_rom_with_banks(2), chr_rom_8kb(), Mirroring::Vertical);

        assert_eq!(mapper.mirroring(), Mirroring::Vertical);
    }

    #[test]
    fn control_register_selects_mirroring() {
        let mut mapper = Mmc1::new(prg_rom_with_banks(2), chr_rom_8kb(), Mirroring::Horizontal);

        for (value, expected) in [
            (0, Mirroring::OneScreenA),
            (1, Mirroring::OneScreenB),
            (2, Mirroring::Vertical),
            (3, Mirroring::Horizontal),
        ] {
            load_register(&mut mapper, 0x8000, value);
            assert_eq!(mapper.mirroring(), expected, "control value {}", value);
        }
    }

    #[test]
    fn register_selected_by_address_high_bits() {
        let mut mapper = Mmc1::new(prg_rom_with_banks(2), chr_rom_8kb(), Mirroring::Vertical);

        // Any address in $8000-$9FFF targets the control register.
        load_register(&mut mapper, 0x9FFF, 0x03);

        assert_eq!(mapper.mirroring(), Mirroring::Horizontal);
    }

    #[test]
    fn bit_seven_resets_the_shift_register() {
        let mut mapper = Mmc1::new(prg_rom_with_banks(4), chr_rom_8kb(), Mirroring::Horizontal);

        // Start a partial load ...
        mapper.cpu_write(0xE000, 1);
        mapper.cpu_write(0xE000, 1);
        // ... discard it by writing bit 7 ...
        mapper.cpu_write(0xE000, 0x80);
        // ... then load a full value selecting bank 2.
        load_register(&mut mapper, 0xE000, 2);

        // Default control selects a switchable bank at $8000.
        assert_eq!(mapper.cpu_read(0x8000), 2);
    }

    #[test]
    fn prg_bank_masks_to_low_four_bits() {
        let mut mapper = Mmc1::new(prg_rom_with_banks(4), chr_rom_8kb(), Mirroring::Horizontal);

        load_register(&mut mapper, 0xE000, 0x1F); // only the low 4 bits are kept -> 0x0F

        // Default control: switchable $8000; 15 % 4 banks == 3.
        assert_eq!(mapper.cpu_read(0x8000), 3);
    }

    #[test]
    fn switchable_bank_wraps_modulo_prg_size() {
        let mut mapper = Mmc1::new(prg_rom_with_banks(4), chr_rom_8kb(), Mirroring::Horizontal);

        load_register(&mut mapper, 0xE000, 5); // 5 % 4 banks == 1

        assert_eq!(mapper.cpu_read(0x8000), 1);
    }

    #[test]
    fn prg_16kb_mode_fixes_last_bank_at_c000() {
        // Default control: 16KB mode, switchable at $8000, $C000 fixed to last bank.
        let mut mapper = Mmc1::new(prg_rom_with_banks(4), chr_rom_8kb(), Mirroring::Horizontal);

        load_register(&mut mapper, 0xE000, 2);

        assert_eq!(mapper.cpu_read(0x8000), 2);
        assert_eq!(mapper.cpu_read(0xBFFF), 2);
        assert_eq!(mapper.cpu_read(0xC000), 3);
        assert_eq!(mapper.cpu_read(0xFFFF), 3);
    }

    #[test]
    fn prg_16kb_mode_fixes_first_bank_at_8000() {
        let mut mapper = Mmc1::new(prg_rom_with_banks(4), chr_rom_8kb(), Mirroring::Horizontal);

        // 16KB mode with the fixed bank at $8000 and a switchable bank at $C000.
        load_register(&mut mapper, 0x8000, 0x04);
        load_register(&mut mapper, 0xE000, 2);

        assert_eq!(mapper.cpu_read(0x8000), 0);
        assert_eq!(mapper.cpu_read(0xC000), 2);
    }

    #[test]
    fn prg_32kb_mode_selects_consecutive_banks() {
        let mut mapper = Mmc1::new(prg_rom_with_banks(4), chr_rom_8kb(), Mirroring::Horizontal);

        load_register(&mut mapper, 0x8000, 0x00); // 32KB mode
        load_register(&mut mapper, 0xE000, 4);

        // The low bit of the bank number is OR'd in for the $C000 half.
        assert_eq!(mapper.cpu_read(0x8000), 2);
        assert_eq!(mapper.cpu_read(0xC000), 3);
    }

    #[test]
    fn chr_4kb_mode_selects_banks_independently() {
        let mut mapper = Mmc1::new(prg_rom_with_banks(2), chr_rom_32kb(), Mirroring::Horizontal);

        load_register(&mut mapper, 0x8000, 0x10); // CHR 4KB mode
        load_register(&mut mapper, 0xA000, 3); // bank for $0000-$0FFF
        load_register(&mut mapper, 0xC000, 5); // bank for $1000-$1FFF

        assert_eq!(mapper.ppu_read(0x0000), 0xA3);
        assert_eq!(mapper.ppu_read(0x0FFF), 0xA3);
        assert_eq!(mapper.ppu_read(0x1000), 0xA5);
        assert_eq!(mapper.ppu_read(0x1FFF), 0xA5);
    }

    #[test]
    fn chr_8kb_mode_ignores_chr_bank_1() {
        // Default control is 8KB CHR mode: chr_bank_1 must not affect reads.
        let mut mapper = Mmc1::new(prg_rom_with_banks(2), chr_rom_32kb(), Mirroring::Horizontal);

        load_register(&mut mapper, 0xC000, 5); // chr_bank_1 (ignored in 8KB mode)

        assert_eq!(mapper.ppu_read(0x0000), 0xA0);
        assert_eq!(mapper.ppu_read(0x1000), 0xA1);
        assert_eq!(mapper.ppu_read(0x1FFF), 0xA1);
    }

    #[test]
    fn chr_8kb_mode_ignores_bank_bit_zero() {
        let mut mapper = Mmc1::new(prg_rom_with_banks(2), chr_rom_32kb(), Mirroring::Horizontal);

        load_register(&mut mapper, 0xA000, 1); // bit 0 ignored -> 8KB bank 0

        assert_eq!(mapper.ppu_read(0x0000), 0xA0);
    }

    #[test]
    fn chr_8kb_mode_selects_bank_from_bits_one_to_four() {
        let mut mapper = Mmc1::new(prg_rom_with_banks(2), chr_rom_32kb(), Mirroring::Horizontal);

        // Bits 1-4 select the 8KB bank, so register value 4 maps to bank 2 ($4000-$5FFF).
        load_register(&mut mapper, 0xA000, 4);

        assert_eq!(mapper.ppu_read(0x0000), 0xA4);
        assert_eq!(mapper.ppu_read(0x1FFF), 0xA5);
    }

    #[test]
    fn chr_8kb_read_past_end_of_rom_returns_zero() {
        let mut mapper = Mmc1::new(prg_rom_with_banks(2), chr_rom_32kb(), Mirroring::Horizontal);

        // 8KB bank 4 starts at $8000, past the end of the 32KB CHR ROM.
        load_register(&mut mapper, 0xA000, 8);

        assert_eq!(mapper.ppu_read(0x0000), 0);
    }

    #[test]
    fn ppu_write_updates_chr_ram_when_present() {
        // An 8KB CHR store is treated as writable CHR RAM.
        let mut mapper = Mmc1::new(prg_rom_with_banks(2), chr_rom_8kb(), Mirroring::Horizontal);

        mapper.ppu_write(0x0000, 0x12);
        mapper.ppu_write(0x1FFF, 0x34);

        assert_eq!(mapper.ppu_read(0x0000), 0x12);
        assert_eq!(mapper.ppu_read(0x1FFF), 0x34);
    }

    #[test]
    fn ppu_write_is_ignored_when_chr_is_rom() {
        let mut mapper = Mmc1::new(prg_rom_with_banks(2), chr_rom_32kb(), Mirroring::Horizontal);

        mapper.ppu_write(0x0000, 0x12);

        assert_eq!(mapper.ppu_read(0x0000), 0xA0);
    }

    #[test]
    fn cpu_read_outside_prg_returns_zero() {
        let mut mapper = Mmc1::new(prg_rom_with_banks(2), chr_rom_8kb(), Mirroring::Horizontal);

        assert_eq!(mapper.cpu_read(0x0000), 0);
        assert_eq!(mapper.cpu_read(0x6000), 0);
        assert_eq!(mapper.cpu_read(0x7FFF), 0);
    }

    #[test]
    fn ppu_read_outside_chr_returns_zero() {
        let mut mapper = Mmc1::new(prg_rom_with_banks(2), chr_rom_32kb(), Mirroring::Horizontal);

        assert_eq!(mapper.ppu_read(0x2000), 0);
    }
}
