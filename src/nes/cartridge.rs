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
++++++++- PRG RAM size
*/

/*
* Flag_9:
76543210
||||||||
|||||||+- TV system (0: NTSC; 1: PAL)
+++++++-- Reserved, set to zero
*/

/**
* Flag_10:
76543210
  ||  ||
  ||  ++- TV system (0: NTSC; 2: PAL; 1/3: dual compatible)
  |+----- PRG RAM ($6000-$7FFF) (0: present; 1: not present)
  +------ 0: Board has no bus conflicts; 1: Board has bus conflicts
*/
use std::error::Error;

pub struct Cartridge {
    Magic: u32,       // ascii "NES" with MS-DOS end of file
    prg_rom_size: u8, // Size of PRG ROM in 16 KB units
    chr_rom_size: u8, //Size of CHR ROM in 8 KB units (value 0 means the board uses CHR RAM)
    flags_6: u8,
    flags_7: u8,
    flags_8: u8,
    flags_9: u8,
    flags_10: u8,
    padding: [u8; 5],
}

pub fn parse(data: &[u8]) -> Result<Cartridge, Error> {}
