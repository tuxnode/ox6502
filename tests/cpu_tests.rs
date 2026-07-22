// Unit Tests for 6502/65C02 CPU Instructions
// Based on Klaus Dormann's 6502 functional test patterns

// Integration tests for 6502/65C02 CPU Instructions

use ox6502::bus::Bus;
use ox6502::cpu::Cpu;
use ox6502::instructions::{FLAG_C, FLAG_D, FLAG_I, FLAG_N, FLAG_V, FLAG_Z};
use ox6502::opcodes::*;

// Test bus implementation
struct TestBus {
    memory: [u8; 0x10000],
}

impl TestBus {
    fn new() -> Self {
        Self {
            memory: [0; 0x10000],
        }
    }

    fn load_program(&mut self, code: &[u8], start: u16) {
        for (i, byte) in code.iter().enumerate() {
            self.memory[start as usize + i] = *byte;
        }
    }
}

impl Bus for TestBus {
    fn cpu_read(&mut self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    fn cpu_write(&mut self, addr: u16, val: u8) {
        self.memory[addr as usize] = val;
    }

    fn ppu_read(&mut self, _addr: u16) -> u8 {
        0
    }

    fn ppu_write(&mut self, _addr: u16, _val: u8) {}
}

// Helper to create a CPU with reset vector pointing to $0400
#[allow(dead_code)]
fn create_test_cpu() -> Cpu<TestBus> {
    let mut bus = TestBus::new();
    // Set reset vector at $FFFC-$FFFD to point to $0400
    bus.memory[0xFFFC] = 0x00;
    bus.memory[0xFFFD] = 0x04;
    Cpu::new(bus)
}

struct SyncTestBus {
    memory: [u8; 0x10000],
    ticked_cycles: u8,
}

impl SyncTestBus {
    fn new() -> Self {
        Self {
            memory: [0; 0x10000],
            ticked_cycles: 0,
        }
    }
}

impl Bus for SyncTestBus {
    fn cpu_read(&mut self, addr: u16) -> u8 {
        if addr == 0x2002 {
            self.ticked_cycles
        } else {
            self.memory[addr as usize]
        }
    }

    fn cpu_write(&mut self, addr: u16, val: u8) {
        self.memory[addr as usize] = val;
    }

    fn ppu_read(&mut self, _addr: u16) -> u8 {
        0
    }

    fn ppu_write(&mut self, _addr: u16, _val: u8) {}

    fn tick(&mut self, cpu_cycles: u8) -> ox6502::bus::TickResult {
        self.ticked_cycles = self.ticked_cycles.saturating_add(cpu_cycles);
        ox6502::bus::TickResult::default()
    }
}

#[test]
fn test_step_nes_ticks_before_ppu_status_read() {
    let mut bus = SyncTestBus::new();
    bus.memory[0x0400] = LDA_ABS;
    bus.memory[0x0401] = 0x02;
    bus.memory[0x0402] = 0x20;
    bus.memory[0xFFFC] = 0x00;
    bus.memory[0xFFFD] = 0x04;
    let mut cpu = Cpu::new(bus);

    cpu.step_nes();

    assert_eq!(cpu.a, 4);
}

// ==================== Load/Store Tests ====================

#[test]
fn test_lda_immediate() {
    let mut bus = TestBus::new();
    bus.load_program(&[LDA_IMM, 0x42], 0x0400);
    bus.memory[0xFFFC] = 0x00;
    bus.memory[0xFFFD] = 0x04;
    let mut cpu = Cpu::new(bus);

    cpu.step(); // LDA #$42
    assert_eq!(cpu.a, 0x42);
    assert!(!cpu.get_flag(FLAG_N));
    assert!(!cpu.get_flag(FLAG_Z));
}

#[test]
fn test_lda_immediate_zero() {
    let mut bus = TestBus::new();
    bus.load_program(&[LDA_IMM, 0x00], 0x0400);
    bus.memory[0xFFFC] = 0x00;
    bus.memory[0xFFFD] = 0x04;
    let mut cpu = Cpu::new(bus);

    cpu.step();
    assert_eq!(cpu.a, 0x00);
    assert!(!cpu.get_flag(FLAG_N));
    assert!(cpu.get_flag(FLAG_Z));
}

#[test]
fn test_lda_immediate_negative() {
    let mut bus = TestBus::new();
    bus.load_program(&[LDA_IMM, 0x80], 0x0400);
    bus.memory[0xFFFC] = 0x00;
    bus.memory[0xFFFD] = 0x04;
    let mut cpu = Cpu::new(bus);

    cpu.step();
    assert_eq!(cpu.a, 0x80);
    assert!(cpu.get_flag(FLAG_N));
    assert!(!cpu.get_flag(FLAG_Z));
}

#[test]
fn test_lda_zeropage() {
    let mut bus = TestBus::new();
    bus.memory[0x10] = 0x55;
    bus.load_program(&[LDA_ZP, 0x10], 0x0400);
    bus.memory[0xFFFC] = 0x00;
    bus.memory[0xFFFD] = 0x04;
    let mut cpu = Cpu::new(bus);

    cpu.step();
    assert_eq!(cpu.a, 0x55);
}

#[test]
fn test_lda_zeropage_x() {
    let mut bus = TestBus::new();
    bus.memory[0x15] = 0xAA;
    bus.load_program(&[LDX_IMM, 0x05, LDA_ZPX, 0x10], 0x0400);
    bus.memory[0xFFFC] = 0x00;
    bus.memory[0xFFFD] = 0x04;
    let mut cpu = Cpu::new(bus);

    cpu.step(); // LDX #$05
    cpu.step(); // LDA $10,X -> $15
    assert_eq!(cpu.a, 0xAA);
}

#[test]
fn test_lda_absolute() {
    let mut bus = TestBus::new();
    bus.memory[0x1234] = 0x77;
    bus.load_program(&[LDA_ABS, 0x34, 0x12], 0x0400);
    bus.memory[0xFFFC] = 0x00;
    bus.memory[0xFFFD] = 0x04;
    let mut cpu = Cpu::new(bus);

    cpu.step();
    assert_eq!(cpu.a, 0x77);
}

#[test]
fn test_ldx_immediate() {
    let mut bus = TestBus::new();
    bus.load_program(&[LDX_IMM, 0xBB], 0x0400);
    bus.memory[0xFFFC] = 0x00;
    bus.memory[0xFFFD] = 0x04;
    let mut cpu = Cpu::new(bus);

    cpu.step();
    assert_eq!(cpu.x, 0xBB);
}

#[test]
fn test_ldy_immediate() {
    let mut bus = TestBus::new();
    bus.load_program(&[LDY_IMM, 0xCC], 0x0400);
    bus.memory[0xFFFC] = 0x00;
    bus.memory[0xFFFD] = 0x04;
    let mut cpu = Cpu::new(bus);

    cpu.step();
    assert_eq!(cpu.y, 0xCC);
}

#[test]
fn test_sta_zeropage() {
    let mut bus = TestBus::new();
    bus.load_program(&[LDA_IMM, 0x42, STA_ZP, 0x10], 0x0400);
    bus.memory[0xFFFC] = 0x00;
    bus.memory[0xFFFD] = 0x04;
    let mut cpu = Cpu::new(bus);

    cpu.step(); // LDA #$42
    cpu.step(); // STA $10
    assert_eq!(cpu.read(0x0010), 0x42);
}

#[test]
fn test_stx_zeropage() {
    let mut bus = TestBus::new();
    bus.load_program(&[LDX_IMM, 0x55, STX_ZP, 0x20], 0x0400);
    bus.memory[0xFFFC] = 0x00;
    bus.memory[0xFFFD] = 0x04;
    let mut cpu = Cpu::new(bus);

    cpu.step(); // LDX #$55
    cpu.step(); // STX $20
    assert_eq!(cpu.read(0x0020), 0x55);
}

#[test]
fn test_sty_zeropage() {
    let mut bus = TestBus::new();
    bus.load_program(&[LDY_IMM, 0x66, STY_ZP, 0x30], 0x0400);
    bus.memory[0xFFFC] = 0x00;
    bus.memory[0xFFFD] = 0x04;
    let mut cpu = Cpu::new(bus);

    cpu.step(); // LDY #$66
    cpu.step(); // STY $30
    assert_eq!(cpu.read(0x0030), 0x66);
}

#[test]
fn test_stz_zeropage() {
    // NMOS 6502: STZ zp is a NOP (reads operand but does not write)
    let mut bus = TestBus::new();
    bus.memory[0x40] = 0xFF;
    bus.load_program(&[STZ_ZP, 0x40], 0x0400);
    bus.memory[0xFFFC] = 0x00;
    bus.memory[0xFFFD] = 0x04;
    let mut cpu = Cpu::new(bus);

    cpu.step(); // NOP (STZ $40 on CMOS, NOP on NMOS)
    assert_eq!(cpu.read(0x0040), 0xFF); // Memory unchanged
}

// ==================== Stack Operations Tests ====================

#[test]
fn test_pha_pla() {
    let mut bus = TestBus::new();
    bus.load_program(&[LDA_IMM, 0x55, PHA, LDA_IMM, 0x00, PLA], 0x0400);
    bus.memory[0xFFFC] = 0x00;
    bus.memory[0xFFFD] = 0x04;
    let mut cpu = Cpu::new(bus);

    cpu.step(); // LDA #$55
    cpu.step(); // PHA
    cpu.step(); // LDA #$00
    cpu.step(); // PLA
    assert_eq!(cpu.a, 0x55);
}

#[test]
fn test_php_plp() {
    let mut bus = TestBus::new();
    bus.load_program(
        &[
            SEC, // Set carry
            PHP, // Push status
            CLC, // Clear carry
            PLP, // Pull status
        ],
        0x0400,
    );
    bus.memory[0xFFFC] = 0x00;
    bus.memory[0xFFFD] = 0x04;
    let mut cpu = Cpu::new(bus);

    cpu.step(); // SEC
    cpu.step(); // PHP
    cpu.step(); // CLC
    cpu.step(); // PLP
    assert!(cpu.get_flag(FLAG_C)); // Carry should be restored
}

#[test]
fn test_pha_pla_flags() {
    let mut bus = TestBus::new();
    bus.load_program(&[LDA_IMM, 0xFF, PHA, PLA], 0x0400);
    bus.memory[0xFFFC] = 0x00;
    bus.memory[0xFFFD] = 0x04;
    let mut cpu = Cpu::new(bus);

    cpu.step(); // LDA #$FF
    cpu.step(); // PHA
    cpu.step(); // PLA
    assert_eq!(cpu.a, 0xFF);
    assert!(cpu.get_flag(FLAG_N));
    assert!(!cpu.get_flag(FLAG_Z));
}

#[test]
fn test_stack_push_pop_order() {
    let mut bus = TestBus::new();
    bus.load_program(
        &[LDA_IMM, 0xAA, PHA, LDA_IMM, 0x55, PHA, PLA, TAX, PLA],
        0x0400,
    );
    bus.memory[0xFFFC] = 0x00;
    bus.memory[0xFFFD] = 0x04;
    let mut cpu = Cpu::new(bus);

    cpu.step(); // LDA #$AA
    cpu.step(); // PHA
    cpu.step(); // LDA #$55
    cpu.step(); // PHA
    cpu.step(); // PLA -> A = $55
    assert_eq!(cpu.a, 0x55);
    cpu.step(); // TAX -> X = $55
    assert_eq!(cpu.x, 0x55);
    cpu.step(); // PLA -> A = $AA
    assert_eq!(cpu.a, 0xAA);
}

#[test]
fn test_phx_plx() {
    // NMOS 6502: PHX/PLX are NOPs
    let mut bus = TestBus::new();
    bus.load_program(&[LDX_IMM, 0xBB, PHX, LDX_IMM, 0x00, PLX], 0x0400);
    bus.memory[0xFFFC] = 0x00;
    bus.memory[0xFFFD] = 0x04;
    let mut cpu = Cpu::new(bus);

    cpu.step(); // LDX #$BB
    cpu.step(); // PHX (NOP on NMOS)
    cpu.step(); // LDX #$00
    cpu.step(); // PLX (NOP on NMOS)
    assert_eq!(cpu.x, 0x00); // X unchanged by NOPs
}

#[test]
fn test_phy_ply() {
    // NMOS 6502: PHY/PLY are NOPs
    let mut bus = TestBus::new();
    bus.load_program(&[LDY_IMM, 0xCC, PHY, LDY_IMM, 0x00, PLY], 0x0400);
    bus.memory[0xFFFC] = 0x00;
    bus.memory[0xFFFD] = 0x04;
    let mut cpu = Cpu::new(bus);

    cpu.step(); // LDY #$CC
    cpu.step(); // PHY (NOP on NMOS)
    cpu.step(); // LDY #$00
    cpu.step(); // PLY (NOP on NMOS)
    assert_eq!(cpu.y, 0x00); // Y unchanged by NOPs
}

// ==================== Transfer Tests ====================

#[test]
fn test_tax() {
    let mut bus = TestBus::new();
    bus.load_program(&[LDA_IMM, 0x42, TAX], 0x0400);
    bus.memory[0xFFFC] = 0x00;
    bus.memory[0xFFFD] = 0x04;
    let mut cpu = Cpu::new(bus);

    cpu.step(); // LDA #$42
    cpu.step(); // TAX
    assert_eq!(cpu.x, 0x42);
}

#[test]
fn test_tay() {
    let mut bus = TestBus::new();
    bus.load_program(&[LDA_IMM, 0x42, TAY], 0x0400);
    bus.memory[0xFFFC] = 0x00;
    bus.memory[0xFFFD] = 0x04;
    let mut cpu = Cpu::new(bus);

    cpu.step(); // LDA #$42
    cpu.step(); // TAY
    assert_eq!(cpu.y, 0x42);
}

#[test]
fn test_txa() {
    let mut bus = TestBus::new();
    bus.load_program(&[LDX_IMM, 0x42, TXA], 0x0400);
    bus.memory[0xFFFC] = 0x00;
    bus.memory[0xFFFD] = 0x04;
    let mut cpu = Cpu::new(bus);

    cpu.step(); // LDX #$42
    cpu.step(); // TXA
    assert_eq!(cpu.a, 0x42);
}

#[test]
fn test_tya() {
    let mut bus = TestBus::new();
    bus.load_program(&[LDY_IMM, 0x42, TYA], 0x0400);
    bus.memory[0xFFFC] = 0x00;
    bus.memory[0xFFFD] = 0x04;
    let mut cpu = Cpu::new(bus);

    cpu.step(); // LDY #$42
    cpu.step(); // TYA
    assert_eq!(cpu.a, 0x42);
}

#[test]
fn test_tsx() {
    let mut bus = TestBus::new();
    bus.load_program(&[LDX_IMM, 0xFF, TXS, TSX], 0x0400);
    bus.memory[0xFFFC] = 0x00;
    bus.memory[0xFFFD] = 0x04;
    let mut cpu = Cpu::new(bus);

    cpu.step(); // LDX #$FF
    cpu.step(); // TXS
    assert_eq!(cpu.sp, 0xFF);
    cpu.step(); // TSX
    assert_eq!(cpu.x, 0xFF);
}

#[test]
fn test_txs() {
    let mut bus = TestBus::new();
    bus.load_program(&[LDX_IMM, 0xFD, TXS], 0x0400);
    bus.memory[0xFFFC] = 0x00;
    bus.memory[0xFFFD] = 0x04;
    let mut cpu = Cpu::new(bus);

    cpu.step(); // LDX #$FD
    cpu.step(); // TXS
    assert_eq!(cpu.sp, 0xFD);
}

// ==================== Compare Tests ====================

#[test]
fn test_cmp_immediate_equal() {
    let mut bus = TestBus::new();
    bus.load_program(&[LDA_IMM, 0x42, CMP_IMM, 0x42], 0x0400);
    bus.memory[0xFFFC] = 0x00;
    bus.memory[0xFFFD] = 0x04;
    let mut cpu = Cpu::new(bus);

    cpu.step(); // LDA #$42
    cpu.step(); // CMP #$42
    assert!(cpu.get_flag(FLAG_Z)); // Equal
    assert!(cpu.get_flag(FLAG_C)); // A >= val
    assert!(!cpu.get_flag(FLAG_N));
}

#[test]
fn test_cmp_immediate_less() {
    let mut bus = TestBus::new();
    bus.load_program(&[LDA_IMM, 0x42, CMP_IMM, 0x43], 0x0400);
    bus.memory[0xFFFC] = 0x00;
    bus.memory[0xFFFD] = 0x04;
    let mut cpu = Cpu::new(bus);

    cpu.step(); // LDA #$42
    cpu.step(); // CMP #$43
    assert!(!cpu.get_flag(FLAG_Z));
    assert!(!cpu.get_flag(FLAG_C)); // A < val
    assert!(cpu.get_flag(FLAG_N));
}

#[test]
fn test_cmp_immediate_greater() {
    let mut bus = TestBus::new();
    bus.load_program(&[LDA_IMM, 0x42, CMP_IMM, 0x41], 0x0400);
    bus.memory[0xFFFC] = 0x00;
    bus.memory[0xFFFD] = 0x04;
    let mut cpu = Cpu::new(bus);

    cpu.step(); // LDA #$42
    cpu.step(); // CMP #$41
    assert!(!cpu.get_flag(FLAG_Z));
    assert!(cpu.get_flag(FLAG_C)); // A >= val
    assert!(!cpu.get_flag(FLAG_N));
}

#[test]
fn test_cpx_immediate() {
    let mut bus = TestBus::new();
    bus.load_program(&[LDX_IMM, 0x42, CPX_IMM, 0x42], 0x0400);
    bus.memory[0xFFFC] = 0x00;
    bus.memory[0xFFFD] = 0x04;
    let mut cpu = Cpu::new(bus);

    cpu.step(); // LDX #$42
    cpu.step(); // CPX #$42
    assert!(cpu.get_flag(FLAG_Z));
    assert!(cpu.get_flag(FLAG_C));
}

#[test]
fn test_cpy_immediate() {
    let mut bus = TestBus::new();
    bus.load_program(&[LDY_IMM, 0x42, CPY_IMM, 0x42], 0x0400);
    bus.memory[0xFFFC] = 0x00;
    bus.memory[0xFFFD] = 0x04;
    let mut cpu = Cpu::new(bus);

    cpu.step(); // LDY #$42
    cpu.step(); // CPY #$42
    assert!(cpu.get_flag(FLAG_Z));
    assert!(cpu.get_flag(FLAG_C));
}

// ==================== Branch Tests ====================

#[test]
fn test_beq_taken() {
    let mut bus = TestBus::new();
    bus.load_program(
        &[
            LDA_IMM, 0x00, CMP_IMM, 0x00, // Z=1
            BEQ, 0x02, // Branch forward 2 bytes
            NOP, NOP,
        ],
        0x0400,
    );
    bus.memory[0xFFFC] = 0x00;
    bus.memory[0xFFFD] = 0x04;
    let mut cpu = Cpu::new(bus);

    cpu.step(); // LDA #$00
    cpu.step(); // CMP #$00
    cpu.step(); // BEQ +2
    // BEQ at $0404, offset fetched at $0405, PC=$0406, target=$0406+2=$0408
    assert_eq!(cpu.pc, 0x0408);
}

#[test]
fn test_beq_not_taken() {
    let mut bus = TestBus::new();
    bus.load_program(
        &[
            LDA_IMM, 0x42, CMP_IMM, 0x00, // Z=0
            BEQ, 0x02, // Should not branch
            NOP, NOP,
        ],
        0x0400,
    );
    bus.memory[0xFFFC] = 0x00;
    bus.memory[0xFFFD] = 0x04;
    let mut cpu = Cpu::new(bus);

    cpu.step(); // LDA #$42
    cpu.step(); // CMP #$00
    cpu.step(); // BEQ +2
    // Should execute next instruction
    assert_eq!(cpu.pc, 0x0406);
}

#[test]
fn test_bne_taken() {
    let mut bus = TestBus::new();
    bus.load_program(
        &[
            LDA_IMM, 0x42, CMP_IMM, 0x00, // Z=0
            BNE, 0x02, // Branch forward 2 bytes
            NOP, NOP,
        ],
        0x0400,
    );
    bus.memory[0xFFFC] = 0x00;
    bus.memory[0xFFFD] = 0x04;
    let mut cpu = Cpu::new(bus);

    cpu.step(); // LDA #$42
    cpu.step(); // CMP #$00
    cpu.step(); // BNE +2
    assert_eq!(cpu.pc, 0x0408);
}

#[test]
fn test_bmi_taken() {
    let mut bus = TestBus::new();
    bus.load_program(
        &[
            LDA_IMM, 0x80, // N=1
            BMI, 0x02, NOP, NOP,
        ],
        0x0400,
    );
    bus.memory[0xFFFC] = 0x00;
    bus.memory[0xFFFD] = 0x04;
    let mut cpu = Cpu::new(bus);

    cpu.step(); // LDA #$80
    cpu.step(); // BMI +2
    assert_eq!(cpu.pc, 0x0406);
}

#[test]
fn test_bpl_taken() {
    let mut bus = TestBus::new();
    bus.load_program(
        &[
            LDA_IMM, 0x00, // N=0
            BPL, 0x02, NOP, NOP,
        ],
        0x0400,
    );
    bus.memory[0xFFFC] = 0x00;
    bus.memory[0xFFFD] = 0x04;
    let mut cpu = Cpu::new(bus);

    cpu.step(); // LDA #$00
    cpu.step(); // BPL +2
    assert_eq!(cpu.pc, 0x0406);
}

#[test]
fn test_bcc_taken() {
    let mut bus = TestBus::new();
    bus.load_program(&[CLC, BCC, 0x02, NOP, NOP], 0x0400);
    bus.memory[0xFFFC] = 0x00;
    bus.memory[0xFFFD] = 0x04;
    let mut cpu = Cpu::new(bus);

    cpu.step(); // CLC
    cpu.step(); // BCC +2
    assert_eq!(cpu.pc, 0x0405);
}

#[test]
fn test_bcs_taken() {
    let mut bus = TestBus::new();
    bus.load_program(&[SEC, BCS, 0x02, NOP, NOP], 0x0400);
    bus.memory[0xFFFC] = 0x00;
    bus.memory[0xFFFD] = 0x04;
    let mut cpu = Cpu::new(bus);

    cpu.step(); // SEC
    cpu.step(); // BCS +2
    assert_eq!(cpu.pc, 0x0405);
}

#[test]
fn test_bra() {
    // NMOS 6502: BRA is a 2-byte NOP (reads operand but does not branch)
    let mut bus = TestBus::new();
    bus.load_program(
        &[
            BRA, 0x04, // NOP on NMOS (reads operand)
            NOP, NOP, NOP, NOP,
        ],
        0x0400,
    );
    bus.memory[0xFFFC] = 0x00;
    bus.memory[0xFFFD] = 0x04;
    let mut cpu = Cpu::new(bus);

    cpu.step(); // NOP (reads operand, advances PC by 2)
    assert_eq!(cpu.pc, 0x0402); // PC advances past the 2-byte NOP
}

// ==================== Jump/JSR/RTS Tests ====================

#[test]
fn test_jmp_absolute() {
    let mut bus = TestBus::new();
    bus.load_program(
        &[
            JMP_ABS, 0x50, 0x04, // Jump to $0450
            NOP,
            // at $0450:
        ],
        0x0400,
    );
    bus.memory[0x0450] = NOP;
    bus.memory[0xFFFC] = 0x00;
    bus.memory[0xFFFD] = 0x04;
    let mut cpu = Cpu::new(bus);

    cpu.step(); // JMP $0450
    assert_eq!(cpu.pc, 0x0450);
}

#[test]
fn test_jsr_rts() {
    let mut bus = TestBus::new();
    bus.load_program(
        &[
            JSR, 0x04, 0x04, // JSR $0404
            NOP,  // $0403: return here
            RTS,  // $0404
        ],
        0x0400,
    );
    bus.memory[0xFFFC] = 0x00;
    bus.memory[0xFFFD] = 0x04;
    let mut cpu = Cpu::new(bus);

    let sp_before = cpu.sp;
    cpu.step(); // JSR $0404
    assert_eq!(cpu.pc, 0x0404);
    assert_eq!(cpu.sp, sp_before - 2);

    cpu.step(); // RTS
    assert_eq!(cpu.pc, 0x0403);
    assert_eq!(cpu.sp, sp_before);
}

#[test]
fn test_jsr_pushes_correct_return_address() {
    let mut bus = TestBus::new();
    bus.load_program(
        &[
            JSR, 0x10, 0x04, // $0400: JSR $0410
            NOP,  // $0403
            // at $0410:
            LDA_IMM, 0x00, RTS,
        ],
        0x0400,
    );
    bus.memory[0xFFFC] = 0x00;
    bus.memory[0xFFFD] = 0x04;
    let mut cpu = Cpu::new(bus);

    cpu.step(); // JSR $0410
    // JSR pushes (PC-1) = $0402 (return addr - 1)
    // Stack: SP=$FC has lo byte, SP=$FD has hi byte
    let hi = cpu.read(0x01FD);
    let lo = cpu.read(0x01FC);
    let return_addr = (hi as u16) << 8 | lo as u16;
    assert_eq!(return_addr, 0x0402);
}

#[test]
fn test_jsr_nested() {
    let mut bus = TestBus::new();
    // Layout:
    // $0400: JSR $0408 (3 bytes)
    // $0403: LDA #$11 (2 bytes)
    // $0405: JMP $0411 (3 bytes)
    // $0408: LDA #$22 (2 bytes)
    // $040A: JSR $040E (3 bytes)
    // $040D: RTS (1 byte)
    // $040E: LDA #$33 (2 bytes)
    // $0410: RTS (1 byte)
    // $0411: NOP (1 byte)
    bus.load_program(
        &[
            JSR, 0x08, 0x04, // $0400: JSR $0408
            LDA_IMM, 0x11, // $0403
            JMP_ABS, 0x11, 0x04, // $0405: JMP $0411
            LDA_IMM, 0x22, // $0408
            JSR, 0x0E, 0x04, // $040A: JSR $040E
            RTS,  // $040D
            LDA_IMM, 0x33, // $040E
            RTS,  // $0410
            NOP,  // $0411
        ],
        0x0400,
    );
    bus.memory[0xFFFC] = 0x00;
    bus.memory[0xFFFD] = 0x04;
    let mut cpu = Cpu::new(bus);

    cpu.step(); // JSR $0408
    assert_eq!(cpu.pc, 0x0408);

    cpu.step(); // LDA #$22
    assert_eq!(cpu.a, 0x22);

    cpu.step(); // JSR $040E
    assert_eq!(cpu.pc, 0x040E);

    cpu.step(); // LDA #$33
    assert_eq!(cpu.a, 0x33);

    cpu.step(); // RTS -> $040D + 1 = $040E? No, RTS returns to pushed addr + 1
    // JSR at $040A pushed $040C, RTS returns to $040D
    assert_eq!(cpu.pc, 0x040D);

    cpu.step(); // RTS -> returns to $0403
    assert_eq!(cpu.pc, 0x0403);

    cpu.step(); // LDA #$11
    assert_eq!(cpu.a, 0x11);
}

// ==================== Logic Tests ====================

#[test]
fn test_and_immediate() {
    let mut bus = TestBus::new();
    bus.load_program(&[LDA_IMM, 0xFF, AND_IMM, 0x0F], 0x0400);
    bus.memory[0xFFFC] = 0x00;
    bus.memory[0xFFFD] = 0x04;
    let mut cpu = Cpu::new(bus);

    cpu.step(); // LDA #$FF
    cpu.step(); // AND #$0F
    assert_eq!(cpu.a, 0x0F);
}

#[test]
fn test_ora_immediate() {
    let mut bus = TestBus::new();
    bus.load_program(&[LDA_IMM, 0xF0, ORA_IMM, 0x0F], 0x0400);
    bus.memory[0xFFFC] = 0x00;
    bus.memory[0xFFFD] = 0x04;
    let mut cpu = Cpu::new(bus);

    cpu.step(); // LDA #$F0
    cpu.step(); // ORA #$0F
    assert_eq!(cpu.a, 0xFF);
}

#[test]
fn test_eor_immediate() {
    let mut bus = TestBus::new();
    bus.load_program(&[LDA_IMM, 0xFF, EOR_IMM, 0x0F], 0x0400);
    bus.memory[0xFFFC] = 0x00;
    bus.memory[0xFFFD] = 0x04;
    let mut cpu = Cpu::new(bus);

    cpu.step(); // LDA #$FF
    cpu.step(); // EOR #$0F
    assert_eq!(cpu.a, 0xF0);
}

#[test]
fn test_bit_immediate() {
    let mut bus = TestBus::new();
    bus.load_program(&[LDA_IMM, 0xFF, BIT_IMM, 0x80], 0x0400);
    bus.memory[0xFFFC] = 0x00;
    bus.memory[0xFFFD] = 0x04;
    let mut cpu = Cpu::new(bus);

    cpu.step(); // LDA #$FF
    cpu.step(); // BIT #$80
    assert!(!cpu.get_flag(FLAG_Z)); // A & $80 != 0
    assert!(cpu.get_flag(FLAG_N)); // bit 7 of operand
}

#[test]
fn test_bit_zeropage() {
    let mut bus = TestBus::new();
    bus.memory[0x10] = 0x80;
    bus.load_program(&[LDA_IMM, 0xFF, BIT_ZP, 0x10], 0x0400);
    bus.memory[0xFFFC] = 0x00;
    bus.memory[0xFFFD] = 0x04;
    let mut cpu = Cpu::new(bus);

    cpu.step(); // LDA #$FF
    cpu.step(); // BIT $10
    assert!(!cpu.get_flag(FLAG_Z));
    assert!(cpu.get_flag(FLAG_N));
}

// ==================== Shift/Rotate Tests ====================

#[test]
fn test_asl_accumulator() {
    let mut bus = TestBus::new();
    bus.load_program(&[LDA_IMM, 0x40, ASL_A], 0x0400);
    bus.memory[0xFFFC] = 0x00;
    bus.memory[0xFFFD] = 0x04;
    let mut cpu = Cpu::new(bus);

    cpu.step(); // LDA #$40
    cpu.step(); // ASL A
    assert_eq!(cpu.a, 0x80);
    assert!(!cpu.get_flag(FLAG_C));
    assert!(cpu.get_flag(FLAG_N));
}

#[test]
fn test_asl_carry() {
    let mut bus = TestBus::new();
    bus.load_program(&[LDA_IMM, 0x80, ASL_A], 0x0400);
    bus.memory[0xFFFC] = 0x00;
    bus.memory[0xFFFD] = 0x04;
    let mut cpu = Cpu::new(bus);

    cpu.step(); // LDA #$80
    cpu.step(); // ASL A
    assert_eq!(cpu.a, 0x00);
    assert!(cpu.get_flag(FLAG_C));
    assert!(cpu.get_flag(FLAG_Z));
}

#[test]
fn test_lsr_accumulator() {
    let mut bus = TestBus::new();
    bus.load_program(&[LDA_IMM, 0x01, LSR_A], 0x0400);
    bus.memory[0xFFFC] = 0x00;
    bus.memory[0xFFFD] = 0x04;
    let mut cpu = Cpu::new(bus);

    cpu.step(); // LDA #$01
    cpu.step(); // LSR A
    assert_eq!(cpu.a, 0x00);
    assert!(cpu.get_flag(FLAG_C)); // bit 0 was 1
    assert!(cpu.get_flag(FLAG_Z));
}

#[test]
fn test_rol_accumulator() {
    let mut bus = TestBus::new();
    bus.load_program(&[LDA_IMM, 0x80, CLC, ROL_A], 0x0400);
    bus.memory[0xFFFC] = 0x00;
    bus.memory[0xFFFD] = 0x04;
    let mut cpu = Cpu::new(bus);

    cpu.step(); // LDA #$80
    cpu.step(); // CLC
    cpu.step(); // ROL A
    assert_eq!(cpu.a, 0x00);
    assert!(cpu.get_flag(FLAG_C));
}

#[test]
fn test_ror_accumulator() {
    let mut bus = TestBus::new();
    bus.load_program(&[LDA_IMM, 0x01, CLC, ROR_A], 0x0400);
    bus.memory[0xFFFC] = 0x00;
    bus.memory[0xFFFD] = 0x04;
    let mut cpu = Cpu::new(bus);

    cpu.step(); // LDA #$01
    cpu.step(); // CLC
    cpu.step(); // ROR A
    assert_eq!(cpu.a, 0x00);
    assert!(cpu.get_flag(FLAG_C));
}

#[test]
fn test_asl_zeropage() {
    let mut bus = TestBus::new();
    bus.memory[0x10] = 0x40;
    bus.load_program(&[ASL_ZP, 0x10], 0x0400);
    bus.memory[0xFFFC] = 0x00;
    bus.memory[0xFFFD] = 0x04;
    let mut cpu = Cpu::new(bus);

    cpu.step(); // ASL $10
    assert_eq!(cpu.read(0x10), 0x80);
}

#[test]
fn test_lsr_zeropage() {
    let mut bus = TestBus::new();
    bus.memory[0x10] = 0x03;
    bus.load_program(&[LSR_ZP, 0x10], 0x0400);
    bus.memory[0xFFFC] = 0x00;
    bus.memory[0xFFFD] = 0x04;
    let mut cpu = Cpu::new(bus);

    cpu.step(); // LSR $10
    assert_eq!(cpu.read(0x10), 0x01);
    assert!(cpu.get_flag(FLAG_C));
}

// ==================== Increment/Decrement Tests ====================

#[test]
fn test_inx() {
    let mut bus = TestBus::new();
    bus.load_program(&[LDX_IMM, 0x00, INX], 0x0400);
    bus.memory[0xFFFC] = 0x00;
    bus.memory[0xFFFD] = 0x04;
    let mut cpu = Cpu::new(bus);

    cpu.step(); // LDX #$00
    cpu.step(); // INX
    assert_eq!(cpu.x, 0x01);
}

#[test]
fn test_inx_zero_to_one() {
    let mut bus = TestBus::new();
    bus.load_program(&[LDX_IMM, 0xFF, INX], 0x0400);
    bus.memory[0xFFFC] = 0x00;
    bus.memory[0xFFFD] = 0x04;
    let mut cpu = Cpu::new(bus);

    cpu.step(); // LDX #$FF
    cpu.step(); // INX
    assert_eq!(cpu.x, 0x00);
    assert!(cpu.get_flag(FLAG_Z));
}

#[test]
fn test_iny() {
    let mut bus = TestBus::new();
    bus.load_program(&[LDY_IMM, 0x00, INY], 0x0400);
    bus.memory[0xFFFC] = 0x00;
    bus.memory[0xFFFD] = 0x04;
    let mut cpu = Cpu::new(bus);

    cpu.step(); // LDY #$00
    cpu.step(); // INY
    assert_eq!(cpu.y, 0x01);
}

#[test]
fn test_dex() {
    let mut bus = TestBus::new();
    bus.load_program(&[LDX_IMM, 0x01, DEX], 0x0400);
    bus.memory[0xFFFC] = 0x00;
    bus.memory[0xFFFD] = 0x04;
    let mut cpu = Cpu::new(bus);

    cpu.step(); // LDX #$01
    cpu.step(); // DEX
    assert_eq!(cpu.x, 0x00);
    assert!(cpu.get_flag(FLAG_Z));
}

#[test]
fn test_dey() {
    let mut bus = TestBus::new();
    bus.load_program(&[LDY_IMM, 0x01, DEY], 0x0400);
    bus.memory[0xFFFC] = 0x00;
    bus.memory[0xFFFD] = 0x04;
    let mut cpu = Cpu::new(bus);

    cpu.step(); // LDY #$01
    cpu.step(); // DEY
    assert_eq!(cpu.y, 0x00);
    assert!(cpu.get_flag(FLAG_Z));
}

#[test]
fn test_inc_zeropage() {
    let mut bus = TestBus::new();
    bus.memory[0x10] = 0x05;
    bus.load_program(&[INC_ZP, 0x10], 0x0400);
    bus.memory[0xFFFC] = 0x00;
    bus.memory[0xFFFD] = 0x04;
    let mut cpu = Cpu::new(bus);

    cpu.step(); // INC $10
    assert_eq!(cpu.read(0x10), 0x06);
}

#[test]
fn test_dec_zeropage() {
    let mut bus = TestBus::new();
    bus.memory[0x10] = 0x05;
    bus.load_program(&[DEC_ZP, 0x10], 0x0400);
    bus.memory[0xFFFC] = 0x00;
    bus.memory[0xFFFD] = 0x04;
    let mut cpu = Cpu::new(bus);

    cpu.step(); // DEC $10
    assert_eq!(cpu.read(0x10), 0x04);
}

#[test]
fn test_inc_a() {
    // NMOS 6502: INC A is a NOP
    let mut bus = TestBus::new();
    bus.load_program(&[LDA_IMM, 0x7F, INC_A], 0x0400);
    bus.memory[0xFFFC] = 0x00;
    bus.memory[0xFFFD] = 0x04;
    let mut cpu = Cpu::new(bus);

    cpu.step(); // LDA #$7F
    cpu.step(); // NOP (INC A on NMOS)
    assert_eq!(cpu.a, 0x7F); // A unchanged by NOP
}

#[test]
fn test_dec_a() {
    // NMOS 6502: DEC A is a NOP
    let mut bus = TestBus::new();
    bus.load_program(&[LDA_IMM, 0x80, DEC_A], 0x0400);
    bus.memory[0xFFFC] = 0x00;
    bus.memory[0xFFFD] = 0x04;
    let mut cpu = Cpu::new(bus);

    cpu.step(); // LDA #$80
    cpu.step(); // NOP (DEC A on NMOS)
    assert_eq!(cpu.a, 0x80); // A unchanged by NOP
}

// ==================== Flag Tests ====================

#[test]
fn test_clc_sec() {
    let mut bus = TestBus::new();
    bus.load_program(&[SEC, CLC], 0x0400);
    bus.memory[0xFFFC] = 0x00;
    bus.memory[0xFFFD] = 0x04;
    let mut cpu = Cpu::new(bus);

    cpu.step(); // SEC
    assert!(cpu.get_flag(FLAG_C));
    cpu.step(); // CLC
    assert!(!cpu.get_flag(FLAG_C));
}

#[test]
fn test_sed_cld() {
    let mut bus = TestBus::new();
    bus.load_program(&[SED, CLD], 0x0400);
    bus.memory[0xFFFC] = 0x00;
    bus.memory[0xFFFD] = 0x04;
    let mut cpu = Cpu::new(bus);

    cpu.step(); // SED
    assert!(cpu.get_flag(FLAG_D));
    cpu.step(); // CLD
    assert!(!cpu.get_flag(FLAG_D));
}

#[test]
fn test_sei_cli() {
    let mut bus = TestBus::new();
    bus.load_program(&[SEI, CLI], 0x0400);
    bus.memory[0xFFFC] = 0x00;
    bus.memory[0xFFFD] = 0x04;
    let mut cpu = Cpu::new(bus);

    cpu.step(); // SEI
    assert!(cpu.get_flag(FLAG_I));
    cpu.step(); // CLI
    assert!(!cpu.get_flag(FLAG_I));
}

#[test]
fn test_clv() {
    let mut bus = TestBus::new();
    bus.load_program(&[CLV], 0x0400);
    bus.memory[0xFFFC] = 0x00;
    bus.memory[0xFFFD] = 0x04;
    let mut cpu = Cpu::new(bus);

    cpu.status = FLAG_V;
    cpu.step(); // CLV
    assert!(!cpu.get_flag(FLAG_V));
}

// ==================== NOP Test ====================

#[test]
fn test_nop() {
    let mut bus = TestBus::new();
    bus.load_program(&[NOP, NOP, NOP], 0x0400);
    bus.memory[0xFFFC] = 0x00;
    bus.memory[0xFFFD] = 0x04;
    let mut cpu = Cpu::new(bus);

    let pc_before = cpu.pc;
    cpu.step(); // NOP
    assert_eq!(cpu.pc, pc_before + 1);
}

// ==================== ADC/SBC Tests ====================

#[test]
fn test_adc_immediate_basic() {
    let mut bus = TestBus::new();
    bus.load_program(&[CLC, LDA_IMM, 0x01, ADC_IMM, 0x01], 0x0400);
    bus.memory[0xFFFC] = 0x00;
    bus.memory[0xFFFD] = 0x04;
    let mut cpu = Cpu::new(bus);

    cpu.step(); // CLC
    cpu.step(); // LDA #$01
    cpu.step(); // ADC #$01
    assert_eq!(cpu.a, 0x02);
    assert!(!cpu.get_flag(FLAG_C));
}

#[test]
fn test_adc_immediate_carry() {
    let mut bus = TestBus::new();
    bus.load_program(&[CLC, LDA_IMM, 0xFF, ADC_IMM, 0x01], 0x0400);
    bus.memory[0xFFFC] = 0x00;
    bus.memory[0xFFFD] = 0x04;
    let mut cpu = Cpu::new(bus);

    cpu.step(); // CLC
    cpu.step(); // LDA #$FF
    cpu.step(); // ADC #$01
    assert_eq!(cpu.a, 0x00);
    assert!(cpu.get_flag(FLAG_C));
    assert!(cpu.get_flag(FLAG_Z));
}

#[test]
fn test_sbc_immediate_basic() {
    let mut bus = TestBus::new();
    bus.load_program(&[SEC, LDA_IMM, 0x05, SBC_IMM, 0x01], 0x0400);
    bus.memory[0xFFFC] = 0x00;
    bus.memory[0xFFFD] = 0x04;
    let mut cpu = Cpu::new(bus);

    cpu.step(); // SEC (no borrow)
    cpu.step(); // LDA #$05
    cpu.step(); // SBC #$01
    assert_eq!(cpu.a, 0x04);
    assert!(cpu.get_flag(FLAG_C)); // No borrow
}

#[test]
fn test_sbc_immediate_borrow() {
    let mut bus = TestBus::new();
    bus.load_program(&[SEC, LDA_IMM, 0x00, SBC_IMM, 0x01], 0x0400);
    bus.memory[0xFFFC] = 0x00;
    bus.memory[0xFFFD] = 0x04;
    let mut cpu = Cpu::new(bus);

    cpu.step(); // SEC (no borrow)
    cpu.step(); // LDA #$00
    cpu.step(); // SBC #$01
    assert_eq!(cpu.a, 0xFF);
    assert!(!cpu.get_flag(FLAG_C)); // Borrow occurred
}

#[test]
fn test_adc_overflow_set() {
    let mut bus = TestBus::new();
    bus.load_program(&[CLC, LDA_IMM, 0x7F, ADC_IMM, 0x01], 0x0400);
    bus.memory[0xFFFC] = 0x00;
    bus.memory[0xFFFD] = 0x04;
    let mut cpu = Cpu::new(bus);

    cpu.step(); // CLC
    cpu.step(); // LDA #$7F
    cpu.step(); // ADC #$01
    assert_eq!(cpu.a, 0x80);
    assert!(cpu.get_flag(FLAG_V)); // Signed overflow
    assert!(cpu.get_flag(FLAG_N));
}

#[test]
fn test_adc_overflow_clear() {
    let mut bus = TestBus::new();
    bus.load_program(&[CLC, LDA_IMM, 0x80, ADC_IMM, 0x80], 0x0400);
    bus.memory[0xFFFC] = 0x00;
    bus.memory[0xFFFD] = 0x04;
    let mut cpu = Cpu::new(bus);

    cpu.step(); // CLC
    cpu.step(); // LDA #$80
    cpu.step(); // ADC #$80
    assert_eq!(cpu.a, 0x00);
    assert!(cpu.get_flag(FLAG_C));
    // $80 + $80 = $100: both negative, result positive = signed overflow
    assert!(cpu.get_flag(FLAG_V));
}

// ==================== ADC/SBC Binary Mode Tests ====================

#[test]
fn test_adc_binary_mode() {
    let mut bus = TestBus::new();
    bus.load_program(
        &[
            CLC, CLD, // Binary mode
            LDA_IMM, 0x10, ADC_IMM, 0x20,
        ],
        0x0400,
    );
    bus.memory[0xFFFC] = 0x00;
    bus.memory[0xFFFD] = 0x04;
    let mut cpu = Cpu::new(bus);

    cpu.step(); // CLC
    cpu.step(); // CLD
    cpu.step(); // LDA #$10
    cpu.step(); // ADC #$20
    assert_eq!(cpu.a, 0x30);
}

#[test]
fn test_sbc_binary_mode() {
    let mut bus = TestBus::new();
    bus.load_program(
        &[
            SEC, CLD, // Binary mode
            LDA_IMM, 0x30, SBC_IMM, 0x10,
        ],
        0x0400,
    );
    bus.memory[0xFFFC] = 0x00;
    bus.memory[0xFFFD] = 0x04;
    let mut cpu = Cpu::new(bus);

    cpu.step(); // SEC
    cpu.step(); // CLD
    cpu.step(); // LDA #$30
    cpu.step(); // SBC #$10
    assert_eq!(cpu.a, 0x20);
}

// ==================== Complex Integration Tests ====================

#[test]
fn test_sbc_binary_minimal() {
    let mut bus = TestBus::new();
    // CLD; SEC; LDA #$99; SBC #$00
    bus.load_program(&[CLD, SEC, LDA_IMM, 0x99, SBC_IMM, 0x00], 0x0400);
    bus.memory[0xFFFC] = 0x00;
    bus.memory[0xFFFD] = 0x04;
    let mut cpu = Cpu::new(bus);

    cpu.step(); // CLD
    cpu.step(); // SEC
    cpu.step(); // LDA #$99
    assert_eq!(cpu.a, 0x99);
    assert!(!cpu.get_flag(FLAG_D), "D should be 0");
    assert!(cpu.get_flag(FLAG_C), "C should be 1");
    cpu.step(); // SBC #$00
    assert_eq!(cpu.a, 0x99, "99-00=99, got {:02X}", cpu.a);
    assert!(cpu.get_flag(FLAG_C), "no borrow expected");
}

#[test]
fn test_lda_sta_roundtrip() {
    let mut bus = TestBus::new();
    bus.load_program(
        &[
            LDA_IMM, 0x42, STA_ZP, 0x10, LDX_IMM, 0x00, LDA_ZP, 0x10, STX_ZP, 0x10,
        ],
        0x0400,
    );
    bus.memory[0xFFFC] = 0x00;
    bus.memory[0xFFFD] = 0x04;
    let mut cpu = Cpu::new(bus);

    cpu.step(); // LDA #$42
    cpu.step(); // STA $10
    cpu.step(); // LDX #$00
    cpu.step(); // LDA $10
    assert_eq!(cpu.a, 0x42);
    cpu.step(); // STX $10
    assert_eq!(cpu.read(0x10), 0x00);
}

#[test]
fn test_branch_loop() {
    let mut bus = TestBus::new();
    // $0400: LDX #$05 (2 bytes)
    // $0402: DEX (1 byte) - loop target
    // $0403: BNE $FD (2 bytes) - $FD = -3 signed, branch back to DEX
    // $0405: NOP (1 byte)
    bus.load_program(
        &[
            LDX_IMM, 0x05, DEX, BNE, 0xFD, // -3 signed, branch back to DEX at $0402
            NOP,
        ],
        0x0400,
    );
    bus.memory[0xFFFC] = 0x00;
    bus.memory[0xFFFD] = 0x04;
    let mut cpu = Cpu::new(bus);

    cpu.step(); // LDX #$05
    // Loop until X=0
    loop {
        cpu.step(); // DEX
        let pc_before = cpu.pc;
        cpu.step(); // BNE
        if cpu.pc == pc_before + 2 {
            break; // BNE not taken, we're done
        }
    }
    assert_eq!(cpu.x, 0x00);
    assert_eq!(cpu.pc, 0x0405);
}

#[test]
fn test_stack_push_pop_full() {
    // NMOS 6502: PHX/PLX/PHY/PLY are NOPs, so only PHA/PLA work
    let mut bus = TestBus::new();
    bus.load_program(
        &[
            LDX_IMM, 0xFF, TXS, // SP = $FF
            LDA_IMM, 0xAA, PHA, // Push A ($AA) to $01FF, SP=$FE
            LDX_IMM, 0xBB, PHX, // NOP on NMOS
            LDY_IMM, 0xCC, PHY, // NOP on NMOS
            // Pull in reverse order
            PLA, // Pull -> A = $AA (only PHA pushed)
            TAX, // X = $AA
            PLA, // Pull -> (empty, reads garbage)
            TAY, // Y = garbage
            PLA, // Pull -> (empty)
        ],
        0x0400,
    );
    bus.memory[0xFFFC] = 0x00;
    bus.memory[0xFFFD] = 0x04;
    let mut cpu = Cpu::new(bus);

    cpu.step();
    cpu.step(); // LDX #$FF, TXS
    cpu.step();
    cpu.step(); // LDA #$AA, PHA
    cpu.step();
    cpu.step(); // LDX #$BB, PHX (NOP)
    cpu.step();
    cpu.step(); // LDY #$CC, PHY (NOP)

    cpu.step(); // PLA -> $AA
    cpu.step(); // TAX
    assert_eq!(cpu.x, 0xAA);

    cpu.step(); // PLA -> garbage (PHX didn't push)
    cpu.step(); // TAY

    cpu.step(); // PLA
}
