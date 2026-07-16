/*
 * iNES Header Parser
 * Reference: https://www.nesdev.org/wiki/INES
 */

/*
Flags Reference:
76543210
||||||||
|||||||+- Nametable arrangement: 0: vertical arrangement ("horizontal mirrored") (CIRAM A10 = PPU A11)
|||||||                          1: horizontal arrangement ("vertically mirrored") (CIRAM A10 = PPU A10)
||||||+-- 1: Cartridge contains battery-backed PRG RAM ($6000-7FFF) or other persistent memory
|||||+--- 1: 512-byte trainer at $7000-$71FF (stored before PRG data)
||||+---- 1: Alternative nametable layout
++++----- Lower nybble of mapper number
*/

pub struct iNESHEADER {
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
