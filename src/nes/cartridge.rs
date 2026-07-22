/*
 * iNES Header Parser
 * Reference: https://www.nesdev.org/wiki/INES
 */

/*
Flags_6 Reference:
76543210
||||||||
|||||||+- Nametable arrangement: 0: vertical arrangement ("horizontal mirrored") (CIRAM A10 = PPU A11)
|||||||                          1: horizontal arrangement ("vertically mirrored") (CIRAM A10 = PPU A10)
||||||+-- 1: Cartridge contains battery-backed PRG RAM ($6000-7FFF) or other persistent memory
|||||+--- 1: 512-byte trainer at $7000-$71FF (stored before PRG data)
||||+---- 1: Alternative nametable layout
++++----- Lower nybble of mapper number
*/

/*
* Flag_7 Reference:
76543210
||||||||
|||||||+- VS Unisystem
||||||+-- PlayChoice-10 (8 KB of Hint Screen data stored after CHR data)
||||++--- If equal to 2, flags 8-15 are in NES 2.0 format
++++----- Upper nybble of mapper number
*/

/*
* Flag_8:
76543210
||||||||
++++++++ PRG RAM size
*/

/*
* Flag_9:
76543210
||||||||
|||||||+- TV system (0: NTSC; 1: PAL)
+++++++-- Reserved, set to zero
*/

/*
 * Flag_10:
 * 76543210
  ||  ||
  ||  ++- TV system (0: NTSC; 2: PAL; 1/3: dual compatible)
  |+----- PRG RAM ($6000-$7FFF) (0: present; 1: not present)
  +------ 0: Board has no bus conflicts; 1: Board has bus conflicts
*/

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mirroring {
    Horizontal,
    Vertical,
    FourScreen,
    OneScreenA,
    OneScreenB,
}

#[derive(Debug)]
pub enum CartridgeError {
    TooShort,
    InvalidMagic,
    NoPrgRom,
}

impl std::fmt::Display for CartridgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TooShort => write!(f, "ROM data too short for iNES header"),
            Self::InvalidMagic => write!(f, "Invalid iNES magic bytes (expected NES\\x1A)"),
            Self::NoPrgRom => write!(f, "No PRG ROM data in file"),
        }
    }
}

impl std::error::Error for CartridgeError {}

pub struct Cartridge {
    pub mapper: u8,
    pub mirroring: Mirroring,
    pub battery: bool,
    pub trainer: bool,
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,
}

pub fn parse(data: &[u8]) -> Result<Cartridge, CartridgeError> {
    if data.len() < 16 {
        return Err(CartridgeError::TooShort);
    }

    if data[0] != b'N' || data[1] != b'E' || data[2] != b'S' || data[3] != 0x1A {
        return Err(CartridgeError::InvalidMagic);
    }

    let prg_rom_units = data[4];
    let chr_rom_units = data[5];
    let flags_6 = data[6];
    let flags_7 = data[7];

    let mapper = (flags_7 & 0xF0) | (flags_6 >> 4);
    let trainer = (flags_6 & 0x04) != 0;
    let battery = (flags_6 & 0x02) != 0;
    let mirroring = if flags_6 & 0x08 != 0 {
        Mirroring::FourScreen
    } else if flags_6 & 0x01 != 0 {
        Mirroring::Vertical
    } else {
        Mirroring::Horizontal
    };

    let header_size = 16;
    let trainer_size: usize = if trainer { 512 } else { 0 };
    let prg_start = header_size + trainer_size;
    let prg_size = prg_rom_units as usize * 0x4000;
    let chr_start = prg_start + prg_size;
    let chr_size = chr_rom_units as usize * 0x2000;

    if data.len() < prg_start + prg_size {
        return Err(CartridgeError::NoPrgRom);
    }

    let prg_rom = data[prg_start..prg_start + prg_size].to_vec();

    let chr_rom = if chr_rom_units == 0 {
        vec![0u8; 0x2000]
    } else {
        if data.len() < chr_start + chr_size {
            return Err(CartridgeError::NoPrgRom);
        }
        data[chr_start..chr_start + chr_size].to_vec()
    };

    Ok(Cartridge {
        mapper,
        mirroring,
        battery,
        trainer,
        prg_rom,
        chr_rom,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ines_header(flags_6: u8) -> Vec<u8> {
        let mut data = vec![0; 16 + 0x4000];
        data[0..4].copy_from_slice(b"NES\x1A");
        data[4] = 1;
        data[6] = flags_6;
        data
    }

    #[test]
    fn parse_flags_6_bit_0_clear_as_horizontal_mirroring() {
        let cart = parse(&ines_header(0x00)).expect("valid cartridge");

        assert_eq!(cart.mirroring, Mirroring::Horizontal);
    }

    #[test]
    fn parse_flags_6_bit_0_set_as_vertical_mirroring() {
        let cart = parse(&ines_header(0x01)).expect("valid cartridge");

        assert_eq!(cart.mirroring, Mirroring::Vertical);
    }

    #[test]
    fn parse_flags_6_four_screen_overrides_bit_0_mirroring() {
        let cart = parse(&ines_header(0x09)).expect("valid cartridge");

        assert_eq!(cart.mirroring, Mirroring::FourScreen);
    }
}
