// 65C02 Opcode Definitions
// Reference: W65C02S Programming Manual

// LDA - Load Accumulator
pub const LDA_IMM: u8 = 0xA9;
pub const LDA_ZP: u8 = 0xA5;
pub const LDA_ZPX: u8 = 0xB5;
pub const LDA_ABS: u8 = 0xAD;
pub const LDA_ABSX: u8 = 0xBD;
pub const LDA_ABSY: u8 = 0xB9;
pub const LDA_ZPI: u8 = 0xB2; // CMOS: (zp)
pub const LDA_ZPXI: u8 = 0xA1; // CMOS: (zp,X)
pub const LDA_AIY: u8 = 0xB1; // CMOS: (zp),Y

// LDX - Load X
pub const LDX_IMM: u8 = 0xA2;
pub const LDX_ZP: u8 = 0xA6;
pub const LDX_ZPY: u8 = 0xB6;
pub const LDX_ABS: u8 = 0xAE;
pub const LDX_ABSY: u8 = 0xBE;

// LDY - Load Y
pub const LDY_IMM: u8 = 0xA0;
pub const LDY_ZP: u8 = 0xA4;
pub const LDY_ZPX: u8 = 0xB4;
pub const LDY_ABS: u8 = 0xAC;
pub const LDY_ABSX: u8 = 0xBC;

// STA - Store Accumulator
pub const STA_ZP: u8 = 0x85;
pub const STA_ZPX: u8 = 0x95;
pub const STA_ABS: u8 = 0x8D;
pub const STA_ABSX: u8 = 0x9D;
pub const STA_ABSY: u8 = 0x99;
pub const STA_ZPI: u8 = 0x92; // CMOS: (zp)
pub const STA_ZPXI: u8 = 0x81; // CMOS: (zp,X)
pub const STA_AIY: u8 = 0x91; // CMOS: (zp),Y

// STX - Store X
pub const STX_ZP: u8 = 0x86;
pub const STX_ZPY: u8 = 0x96;
pub const STX_ABS: u8 = 0x8E;

// STY - Store Y
pub const STY_ZP: u8 = 0x84;
pub const STY_ZPX: u8 = 0x94;
pub const STY_ABS: u8 = 0x8C;

// STZ - Store Zero (CMOS)
pub const STZ_ZP: u8 = 0x64;
pub const STZ_ZPX: u8 = 0x74;
pub const STZ_ABS: u8 = 0x9C;
pub const STZ_ABSX: u8 = 0x9E;

// ADC - Add with Carry
pub const ADC_IMM: u8 = 0x69;
pub const ADC_ZP: u8 = 0x65;
pub const ADC_ZPX: u8 = 0x75;
pub const ADC_ABS: u8 = 0x6D;
pub const ADC_ABSX: u8 = 0x7D;
pub const ADC_ABSY: u8 = 0x79;
pub const ADC_ZPI: u8 = 0x72; // CMOS: (zp)
pub const ADC_ZPXI: u8 = 0x61; // CMOS: (zp,X)
pub const ADC_AIY: u8 = 0x71; // CMOS: (zp),Y

// SBC - Subtract with Borrow
pub const SBC_IMM: u8 = 0xE9;
pub const SBC_ZP: u8 = 0xE5;
pub const SBC_ZPX: u8 = 0xF5;
pub const SBC_ABS: u8 = 0xED;
pub const SBC_ABSX: u8 = 0xFD;
pub const SBC_ABSY: u8 = 0xF9;
pub const SBC_ZPI: u8 = 0xF2; // CMOS: (zp)
pub const SBC_ZPXI: u8 = 0xE1; // CMOS: (zp,X)
pub const SBC_AIY: u8 = 0xF1; // CMOS: (zp),Y

// AND
pub const AND_IMM: u8 = 0x29;
pub const AND_ZP: u8 = 0x25;
pub const AND_ZPX: u8 = 0x35;
pub const AND_ABS: u8 = 0x2D;
pub const AND_ABSX: u8 = 0x3D;
pub const AND_ABSY: u8 = 0x39;
pub const AND_ZPI: u8 = 0x32;
pub const AND_ZPXI: u8 = 0x21;
pub const AND_AIY: u8 = 0x31;

// ORA
pub const ORA_IMM: u8 = 0x09;
pub const ORA_ZP: u8 = 0x05;
pub const ORA_ZPX: u8 = 0x15;
pub const ORA_ABS: u8 = 0x0D;
pub const ORA_ABSX: u8 = 0x1D;
pub const ORA_ABSY: u8 = 0x19;
pub const ORA_ZPI: u8 = 0x12;
pub const ORA_ZPXI: u8 = 0x01;
pub const ORA_AIY: u8 = 0x11;

// EOR
pub const EOR_IMM: u8 = 0x49;
pub const EOR_ZP: u8 = 0x45;
pub const EOR_ZPX: u8 = 0x55;
pub const EOR_ABS: u8 = 0x4D;
pub const EOR_ABSX: u8 = 0x5D;
pub const EOR_ABSY: u8 = 0x59;
pub const EOR_ZPI: u8 = 0x52;
pub const EOR_ZPXI: u8 = 0x41;
pub const EOR_AIY: u8 = 0x51;

// CMP - Compare Accumulator
pub const CMP_IMM: u8 = 0xC9;
pub const CMP_ZP: u8 = 0xC5;
pub const CMP_ZPX: u8 = 0xD5;
pub const CMP_ABS: u8 = 0xCD;
pub const CMP_ABSX: u8 = 0xDD;
pub const CMP_ABSY: u8 = 0xD9;
pub const CMP_ZPI: u8 = 0xD2;
pub const CMP_ZPXI: u8 = 0xC1;
pub const CMP_AIY: u8 = 0xD1;

// CPX - Compare X
pub const CPX_IMM: u8 = 0xE0;
pub const CPX_ZP: u8 = 0xE4;
pub const CPX_ABS: u8 = 0xEC;

// CPY - Compare Y
pub const CPY_IMM: u8 = 0xC0;
pub const CPY_ZP: u8 = 0xC4;
pub const CPY_ABS: u8 = 0xCC;

// INC - Increment
pub const INC_A: u8 = 0x1A; // CMOS: accumulator
pub const INC_ZP: u8 = 0xE6;
pub const INC_ZPX: u8 = 0xF6;
pub const INC_ABS: u8 = 0xEE;
pub const INC_ABSX: u8 = 0xFE;

// DEC - Decrement
pub const DEC_A: u8 = 0x3A; // CMOS: accumulator
pub const DEC_ZP: u8 = 0xC6;
pub const DEC_ZPX: u8 = 0xD6;
pub const DEC_ABS: u8 = 0xCE;
pub const DEC_ABSX: u8 = 0xDE;

// INX, INY, DEX, DEY
pub const INX: u8 = 0xE8;
pub const INY: u8 = 0xC8;
pub const DEX: u8 = 0xCA;
pub const DEY: u8 = 0x88;

// ASL - Arithmetic Shift Left
pub const ASL_A: u8 = 0x0A;
pub const ASL_ZP: u8 = 0x06;
pub const ASL_ZPX: u8 = 0x16;
pub const ASL_ABS: u8 = 0x0E;
pub const ASL_ABSX: u8 = 0x1E;

// LSR - Logical Shift Right
pub const LSR_A: u8 = 0x4A;
pub const LSR_ZP: u8 = 0x46;
pub const LSR_ZPX: u8 = 0x56;
pub const LSR_ABS: u8 = 0x4E;
pub const LSR_ABSX: u8 = 0x5E;

// ROL - Rotate Left
pub const ROL_A: u8 = 0x2A;
pub const ROL_ZP: u8 = 0x26;
pub const ROL_ZPX: u8 = 0x36;
pub const ROL_ABS: u8 = 0x2E;
pub const ROL_ABSX: u8 = 0x3E;

// ROR - Rotate Right
pub const ROR_A: u8 = 0x6A;
pub const ROR_ZP: u8 = 0x66;
pub const ROR_ZPX: u8 = 0x76;
pub const ROR_ABS: u8 = 0x6E;
pub const ROR_ABSX: u8 = 0x7E;

// BIT
pub const BIT_IMM: u8 = 0x89; // CMOS: immediate
pub const BIT_ZP: u8 = 0x24;
pub const BIT_ABS: u8 = 0x2C;
pub const BIT_ZPX: u8 = 0x34; // CMOS
pub const BIT_ABSX: u8 = 0x3C; // CMOS

// TRB - Test and Reset Bits (CMOS)
pub const TRB_ZP: u8 = 0x14;
pub const TRB_ABS: u8 = 0x1C;

// TSB - Test and Set Bits (CMOS)
pub const TSB_ZP: u8 = 0x04;
pub const TSB_ABS: u8 = 0x0C;

// Branch instructions
pub const BCC: u8 = 0x90;
pub const BCS: u8 = 0xB0;
pub const BEQ: u8 = 0xF0;
pub const BNE: u8 = 0xD0;
pub const BMI: u8 = 0x30;
pub const BPL: u8 = 0x10;
pub const BVC: u8 = 0x50;
pub const BVS: u8 = 0x70;
pub const BRA: u8 = 0x80; // CMOS: Branch Always

// Jump
pub const JMP_ABS: u8 = 0x4C;
pub const JMP_IND: u8 = 0x6C;
pub const JMP_INDX: u8 = 0x7C; // CMOS: (addr,X)
pub const JSR: u8 = 0x20;
pub const RTS: u8 = 0x60;
pub const RTI: u8 = 0x40;

// Stack
pub const PHA: u8 = 0x48;
pub const PHP: u8 = 0x08;
pub const PLA: u8 = 0x68;
pub const PLP: u8 = 0x28;
pub const PHX: u8 = 0xDA; // CMOS
pub const PHY: u8 = 0x5A; // CMOS
pub const PLX: u8 = 0xFA; // CMOS
pub const PLY: u8 = 0x7A; // CMOS

// Transfer
pub const TAX: u8 = 0xAA;
pub const TAY: u8 = 0xA8;
pub const TXA: u8 = 0x8A;
pub const TYA: u8 = 0x98;
pub const TSX: u8 = 0xBA;
pub const TXS: u8 = 0x9A;

// System
pub const NOP: u8 = 0xEA;
pub const BRK: u8 = 0x00;
pub const WAI: u8 = 0xCB; // CMOS
pub const STP: u8 = 0xDB; // CMOS

// Flags
pub const SEC: u8 = 0x38;
pub const SED: u8 = 0xF8;
pub const SEI: u8 = 0x78;
pub const CLC: u8 = 0x18;
pub const CLD: u8 = 0xD8;
pub const CLI: u8 = 0x58;
pub const CLV: u8 = 0xB8;
