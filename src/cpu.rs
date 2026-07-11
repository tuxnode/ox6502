// mos 6502 CPU mod

// CMOS(65C02) Documents:
// - W65C02S datasheet：https://www.westerndesigncenter.com/wdc/documentation/w65c02s.pdf
// - W65C02S Programming Manual：https://www.westerndesigncenter.com/wdc/documentation/w65c02-programming-manual.pdf

use crate::bus::Bus;

/*
Status Registers Layout:
Bit:  7  6  5  4  3  2  1  0
      N  V  -  B  D  I  Z  C
*/
struct Cpu<B: Bus> {
    pub a: u8, // Accumulator Register
    pub x: u8,
    pub y: u8,
    pub sp: u8,
    pub pc: u16,
    pub status: u8,

    bus: B,
}
