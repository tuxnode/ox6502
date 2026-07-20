use ox6502::bus::Bus;
use ox6502::cpu::Cpu;
use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
struct SstTest {
    name: String,
    initial: CpuState,
    #[serde(rename = "final")]
    final_state: CpuState,
    #[allow(dead_code)]
    cycles: Vec<Vec<serde_json::Value>>,
}

#[derive(Debug, Deserialize)]
struct CpuState {
    pc: u16,
    s: u8,
    a: u8,
    x: u8,
    y: u8,
    p: u8,
    ram: Vec<Vec<u64>>,
}

struct TestBus {
    memory: [u8; 0x10000],
}

impl TestBus {
    fn new() -> Self {
        Self {
            memory: [0; 0x10000],
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

fn run_sst_test(test: &SstTest) -> bool {
    let mut bus = TestBus::new();

    // Load RAM
    for ram_entry in &test.initial.ram {
        let addr = ram_entry[0] as u16;
        let val = ram_entry[1] as u8;
        bus.memory[addr as usize] = val;
    }

    // Create CPU with bus, then set registers
    let mut cpu = Cpu::new(bus);

    // Set registers (after construction, override the reset vector behavior)
    cpu.a = test.initial.a;
    cpu.x = test.initial.x;
    cpu.y = test.initial.y;
    cpu.sp = test.initial.s;
    cpu.pc = test.initial.pc;
    cpu.status = test.initial.p;

    // Execute one instruction
    cpu.step();

    // Debug: print first failure details
    let mut failed_check = "";
    if cpu.a != test.final_state.a { failed_check = "A"; }
    else if cpu.x != test.final_state.x { failed_check = "X"; }
    else if cpu.y != test.final_state.y { failed_check = "Y"; }
    else if cpu.sp != test.final_state.s { failed_check = "SP"; }
    else if cpu.pc != test.final_state.pc { failed_check = "PC"; }
    else if cpu.status != test.final_state.p { failed_check = "P"; }
    else {
        for ram_entry in &test.final_state.ram {
            let addr = ram_entry[0] as u16;
            let expected_val = ram_entry[1] as u8;
            if cpu.read(addr) != expected_val {
                failed_check = "RAM";
                break;
            }
        }
    }
    if !failed_check.is_empty() {
        eprintln!("FAIL({}): name={}", failed_check, test.name);
        eprintln!("  expected: PC={:#06X} A={:#04X} X={:#04X} Y={:#04X} SP={:#04X} P={:#04X}", 
            test.final_state.pc, test.final_state.a, test.final_state.x, test.final_state.y, test.final_state.s, test.final_state.p);
        eprintln!("  actual:   PC={:#06X} A={:#04X} X={:#04X} Y={:#04X} SP={:#04X} P={:#04X}", 
            cpu.pc, cpu.a, cpu.x, cpu.y, cpu.sp, cpu.status);
    }

    // Check registers
    if cpu.a != test.final_state.a {
        return false;
    }
    if cpu.x != test.final_state.x {
        return false;
    }
    if cpu.y != test.final_state.y {
        return false;
    }
    if cpu.sp != test.final_state.s {
        return false;
    }
    if cpu.pc != test.final_state.pc {
        return false;
    }
    if cpu.status != test.final_state.p {
        return false;
    }

    // Check RAM (only the locations listed in final.ram)
    for ram_entry in &test.final_state.ram {
        let addr = ram_entry[0] as u16;
        let expected_val = ram_entry[1] as u8;
        if cpu.read(addr) != expected_val {
            return false;
        }
    }

    true
}

fn run_opcode_tests(opcode_hex: &str) {
    let path = format!("tests/sst_tests/nes6502/v1/{}.json", opcode_hex);
    let data = fs::read_to_string(&path).unwrap_or_else(|e| panic!("Failed to read {}: {}", path, e));
    let tests: Vec<SstTest> = serde_json::from_str(&data).unwrap_or_else(|e| {
        panic!("Failed to parse {}: {}", path, e)
    });

    let mut passed = 0;
    let mut failed = 0;
    let mut first_failure_detail: Option<String> = None;

    for test in &tests {
        if run_sst_test(test) {
            passed += 1;
        } else {
            failed += 1;
            if first_failure_detail.is_none() {
                first_failure_detail = Some(format!(
                    "name={}, init_pc={:04X} init_a={:02X} init_p={:02X} final_pc={:04X} final_a={:02X} final_p={:02X}",
                    test.name, test.initial.pc, test.initial.a, test.initial.p,
                    test.final_state.pc, test.final_state.a, test.final_state.p
                ));
            }
        }
    }

    if failed > 0 {
        panic!(
            "Opcode 0x{}: {}/{} passed\n  first failure: {}",
            opcode_hex,
            passed,
            tests.len(),
            first_failure_detail.unwrap_or_default()
        );
    }
}

// Generate tests for each opcode
macro_rules! sst_test {
    ($name:ident, $hex:expr) => {
        #[test]
        fn $name() {
            run_opcode_tests($hex);
        }
    };
}

sst_test!(test_sst_00, "00");
sst_test!(test_sst_01, "01");
sst_test!(test_sst_02, "02");
sst_test!(test_sst_03, "03");
sst_test!(test_sst_04, "04");
sst_test!(test_sst_05, "05");
sst_test!(test_sst_06, "06");
sst_test!(test_sst_07, "07");
sst_test!(test_sst_08, "08");
sst_test!(test_sst_09, "09");
sst_test!(test_sst_0a, "0a");
sst_test!(test_sst_0b, "0b");
sst_test!(test_sst_0c, "0c");
sst_test!(test_sst_0d, "0d");
sst_test!(test_sst_0e, "0e");
sst_test!(test_sst_0f, "0f");
sst_test!(test_sst_10, "10");
sst_test!(test_sst_11, "11");
sst_test!(test_sst_12, "12");
sst_test!(test_sst_13, "13");
sst_test!(test_sst_14, "14");
sst_test!(test_sst_15, "15");
sst_test!(test_sst_16, "16");
sst_test!(test_sst_17, "17");
sst_test!(test_sst_18, "18");
sst_test!(test_sst_19, "19");
sst_test!(test_sst_1a, "1a");
sst_test!(test_sst_1b, "1b");
sst_test!(test_sst_1c, "1c");
sst_test!(test_sst_1d, "1d");
sst_test!(test_sst_1e, "1e");
sst_test!(test_sst_1f, "1f");
sst_test!(test_sst_20, "20");
sst_test!(test_sst_21, "21");
sst_test!(test_sst_22, "22");
sst_test!(test_sst_23, "23");
sst_test!(test_sst_24, "24");
sst_test!(test_sst_25, "25");
sst_test!(test_sst_26, "26");
sst_test!(test_sst_27, "27");
sst_test!(test_sst_28, "28");
sst_test!(test_sst_29, "29");
sst_test!(test_sst_2a, "2a");
sst_test!(test_sst_2b, "2b");
sst_test!(test_sst_2c, "2c");
sst_test!(test_sst_2d, "2d");
sst_test!(test_sst_2e, "2e");
sst_test!(test_sst_2f, "2f");
sst_test!(test_sst_30, "30");
sst_test!(test_sst_31, "31");
sst_test!(test_sst_32, "32");
sst_test!(test_sst_33, "33");
sst_test!(test_sst_34, "34");
sst_test!(test_sst_35, "35");
sst_test!(test_sst_36, "36");
sst_test!(test_sst_37, "37");
sst_test!(test_sst_38, "38");
sst_test!(test_sst_39, "39");
sst_test!(test_sst_3a, "3a");
sst_test!(test_sst_3b, "3b");
sst_test!(test_sst_3c, "3c");
sst_test!(test_sst_3d, "3d");
sst_test!(test_sst_3e, "3e");
sst_test!(test_sst_3f, "3f");
sst_test!(test_sst_40, "40");
sst_test!(test_sst_41, "41");
sst_test!(test_sst_42, "42");
sst_test!(test_sst_43, "43");
sst_test!(test_sst_44, "44");
sst_test!(test_sst_45, "45");
sst_test!(test_sst_46, "46");
sst_test!(test_sst_47, "47");
sst_test!(test_sst_48, "48");
sst_test!(test_sst_49, "49");
sst_test!(test_sst_4a, "4a");
sst_test!(test_sst_4b, "4b");
sst_test!(test_sst_4c, "4c");
sst_test!(test_sst_4d, "4d");
sst_test!(test_sst_4e, "4e");
sst_test!(test_sst_4f, "4f");
sst_test!(test_sst_50, "50");
sst_test!(test_sst_51, "51");
sst_test!(test_sst_52, "52");
sst_test!(test_sst_53, "53");
sst_test!(test_sst_54, "54");
sst_test!(test_sst_55, "55");
sst_test!(test_sst_56, "56");
sst_test!(test_sst_57, "57");
sst_test!(test_sst_58, "58");
sst_test!(test_sst_59, "59");
sst_test!(test_sst_5a, "5a");
sst_test!(test_sst_5b, "5b");
sst_test!(test_sst_5c, "5c");
sst_test!(test_sst_5d, "5d");
sst_test!(test_sst_5e, "5e");
sst_test!(test_sst_5f, "5f");
sst_test!(test_sst_60, "60");
sst_test!(test_sst_61, "61");
sst_test!(test_sst_62, "62");
sst_test!(test_sst_63, "63");
sst_test!(test_sst_64, "64");
sst_test!(test_sst_65, "65");
sst_test!(test_sst_66, "66");
sst_test!(test_sst_67, "67");
sst_test!(test_sst_68, "68");
sst_test!(test_sst_69, "69");
sst_test!(test_sst_6a, "6a");
sst_test!(test_sst_6b, "6b");
sst_test!(test_sst_6c, "6c");
sst_test!(test_sst_6d, "6d");
sst_test!(test_sst_6e, "6e");
sst_test!(test_sst_6f, "6f");
sst_test!(test_sst_70, "70");
sst_test!(test_sst_71, "71");
sst_test!(test_sst_72, "72");
sst_test!(test_sst_73, "73");
sst_test!(test_sst_74, "74");
sst_test!(test_sst_75, "75");
sst_test!(test_sst_76, "76");
sst_test!(test_sst_77, "77");
sst_test!(test_sst_78, "78");
sst_test!(test_sst_79, "79");
sst_test!(test_sst_7a, "7a");
sst_test!(test_sst_7b, "7b");
sst_test!(test_sst_7c, "7c");
sst_test!(test_sst_7d, "7d");
sst_test!(test_sst_7e, "7e");
sst_test!(test_sst_7f, "7f");
sst_test!(test_sst_80, "80");
sst_test!(test_sst_81, "81");
sst_test!(test_sst_82, "82");
sst_test!(test_sst_83, "83");
sst_test!(test_sst_84, "84");
sst_test!(test_sst_85, "85");
sst_test!(test_sst_86, "86");
sst_test!(test_sst_87, "87");
sst_test!(test_sst_88, "88");
sst_test!(test_sst_89, "89");
sst_test!(test_sst_8a, "8a");
sst_test!(test_sst_8b, "8b");
sst_test!(test_sst_8c, "8c");
sst_test!(test_sst_8d, "8d");
sst_test!(test_sst_8e, "8e");
sst_test!(test_sst_8f, "8f");
sst_test!(test_sst_90, "90");
sst_test!(test_sst_91, "91");
sst_test!(test_sst_92, "92");
sst_test!(test_sst_93, "93");
sst_test!(test_sst_94, "94");
sst_test!(test_sst_95, "95");
sst_test!(test_sst_96, "96");
sst_test!(test_sst_97, "97");
sst_test!(test_sst_98, "98");
sst_test!(test_sst_99, "99");
sst_test!(test_sst_9a, "9a");
sst_test!(test_sst_9b, "9b");
sst_test!(test_sst_9c, "9c");
sst_test!(test_sst_9d, "9d");
sst_test!(test_sst_9e, "9e");
sst_test!(test_sst_9f, "9f");
sst_test!(test_sst_a0, "a0");
sst_test!(test_sst_a1, "a1");
sst_test!(test_sst_a2, "a2");
sst_test!(test_sst_a3, "a3");
sst_test!(test_sst_a4, "a4");
sst_test!(test_sst_a5, "a5");
sst_test!(test_sst_a6, "a6");
sst_test!(test_sst_a7, "a7");
sst_test!(test_sst_a8, "a8");
sst_test!(test_sst_a9, "a9");
sst_test!(test_sst_aa, "aa");
sst_test!(test_sst_ab, "ab");
sst_test!(test_sst_ac, "ac");
sst_test!(test_sst_ad, "ad");
sst_test!(test_sst_ae, "ae");
sst_test!(test_sst_af, "af");
sst_test!(test_sst_b0, "b0");
sst_test!(test_sst_b1, "b1");
sst_test!(test_sst_b2, "b2");
sst_test!(test_sst_b3, "b3");
sst_test!(test_sst_b4, "b4");
sst_test!(test_sst_b5, "b5");
sst_test!(test_sst_b6, "b6");
sst_test!(test_sst_b7, "b7");
sst_test!(test_sst_b8, "b8");
sst_test!(test_sst_b9, "b9");
sst_test!(test_sst_ba, "ba");
sst_test!(test_sst_bb, "bb");
sst_test!(test_sst_bc, "bc");
sst_test!(test_sst_bd, "bd");
sst_test!(test_sst_be, "be");
sst_test!(test_sst_bf, "bf");
sst_test!(test_sst_c0, "c0");
sst_test!(test_sst_c1, "c1");
sst_test!(test_sst_c2, "c2");
sst_test!(test_sst_c3, "c3");
sst_test!(test_sst_c4, "c4");
sst_test!(test_sst_c5, "c5");
sst_test!(test_sst_c6, "c6");
sst_test!(test_sst_c7, "c7");
sst_test!(test_sst_c8, "c8");
sst_test!(test_sst_c9, "c9");
sst_test!(test_sst_ca, "ca");
sst_test!(test_sst_cb, "cb");
sst_test!(test_sst_cc, "cc");
sst_test!(test_sst_cd, "cd");
sst_test!(test_sst_ce, "ce");
sst_test!(test_sst_cf, "cf");
sst_test!(test_sst_d0, "d0");
sst_test!(test_sst_d1, "d1");
sst_test!(test_sst_d2, "d2");
sst_test!(test_sst_d3, "d3");
sst_test!(test_sst_d4, "d4");
sst_test!(test_sst_d5, "d5");
sst_test!(test_sst_d6, "d6");
sst_test!(test_sst_d7, "d7");
sst_test!(test_sst_d8, "d8");
sst_test!(test_sst_d9, "d9");
sst_test!(test_sst_da, "da");
sst_test!(test_sst_db, "db");
sst_test!(test_sst_dc, "dc");
sst_test!(test_sst_dd, "dd");
sst_test!(test_sst_de, "de");
sst_test!(test_sst_df, "df");
sst_test!(test_sst_e0, "e0");
sst_test!(test_sst_e1, "e1");
sst_test!(test_sst_e2, "e2");
sst_test!(test_sst_e3, "e3");
sst_test!(test_sst_e4, "e4");
sst_test!(test_sst_e5, "e5");
sst_test!(test_sst_e6, "e6");
sst_test!(test_sst_e7, "e7");
sst_test!(test_sst_e8, "e8");
sst_test!(test_sst_e9, "e9");
sst_test!(test_sst_ea, "ea");
sst_test!(test_sst_eb, "eb");
sst_test!(test_sst_ec, "ec");
sst_test!(test_sst_ed, "ed");
sst_test!(test_sst_ee, "ee");
sst_test!(test_sst_ef, "ef");
sst_test!(test_sst_f0, "f0");
sst_test!(test_sst_f1, "f1");
sst_test!(test_sst_f2, "f2");
sst_test!(test_sst_f3, "f3");
sst_test!(test_sst_f4, "f4");
sst_test!(test_sst_f5, "f5");
sst_test!(test_sst_f6, "f6");
sst_test!(test_sst_f7, "f7");
sst_test!(test_sst_f8, "f8");
sst_test!(test_sst_f9, "f9");
sst_test!(test_sst_fa, "fa");
sst_test!(test_sst_fb, "fb");
sst_test!(test_sst_fc, "fc");
sst_test!(test_sst_fd, "fd");
sst_test!(test_sst_fe, "fe");
sst_test!(test_sst_ff, "ff");
