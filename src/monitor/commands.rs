use crate::bus::Bus;
use crate::cpu::Cpu;
use crate::monitor::disass::disassemble_at;
use crate::monitor::{Monitor, TraceEntry};

pub enum Command {
    Step,
    Continue,
    Registers,
    Disassemble {
        addr: Option<u16>,
        count: Option<u8>,
    },
    Memory {
        addr: Option<u16>,
        len: Option<u16>,
    },
    Break {
        addr: u16,
    },
    BreakClear {
        id: usize,
    },
    BreakList,
    Trace {
        count: Option<usize>,
    },
    Quit,
    Help,
    Unknown(String),
}

impl Monitor {
    pub fn cmd_help() {
        println!("Commands:");
        println!("  s, step              Step one instruction");
        println!("  c, continue          Continue execution until breakpoint or trap");
        println!("  r, regs              Show registers and flags");
        println!("  d [addr] [count]     Disassemble [count] instructions at [addr]");
        println!("  m [addr] [len]       Hex dump [len] bytes at [addr]");
        println!("  b <addr>             Set breakpoint at <addr>");
        println!("  bc <id>              Clear breakpoint by id");
        println!("  bl                   List all breakpoints");
        println!("  t [count]            Show last [count] trace entries");
        println!("  q, quit              Exit monitor");
        println!("  h, help              Show this help");
        println!();
        println!("Addresses are hex, with or without $ prefix (e.g. 0400 or $0400).");
    }

    pub fn cmd_step<B: Bus>(&mut self, cpu: &mut Cpu<B>) {
        let pc = cpu.pc;
        let (text, _) = disassemble_at(pc, |a| cpu.read(a));
        let cycles = cpu.step();
        self.trace.push_back(TraceEntry {
            addr: pc,
            text,
            cycles,
        });
        if self.trace.len() > 1000 {
            self.trace.pop_front();
        }
    }

    pub fn cmd_continue<B: Bus>(&mut self, cpu: &mut Cpu<B>) {
        loop {
            let pc_before = cpu.pc;
            cpu.step();
            if self.breakpoints.contains(pc_before) {
                println!("Breakpoint hit at ${:04X}", pc_before);
                break;
            }
            if cpu.pc == pc_before {
                println!("Trap at ${:04X}", cpu.pc);
                break;
            }
        }
    }

    pub fn cmd_regs<B: Bus>(&self, cpu: &Cpu<B>) {
        println!("A:  ${:02X}  X: ${:02X}  Y: ${:02X}", cpu.a, cpu.x, cpu.y);
        println!("SP: ${:02X}  PC: ${:04X}", cpu.sp, cpu.pc);
        println!("NV-BDIZC");
        let n = if cpu.status & 0x80 != 0 { 'N' } else { 'n' };
        let v = if cpu.status & 0x40 != 0 { 'V' } else { 'v' };
        let b = if cpu.status & 0x10 != 0 { 'B' } else { 'b' };
        let d = if cpu.status & 0x08 != 0 { 'D' } else { 'd' };
        let i = if cpu.status & 0x04 != 0 { 'I' } else { 'i' };
        let z = if cpu.status & 0x02 != 0 { 'Z' } else { 'z' };
        let c = if cpu.status & 0x01 != 0 { 'C' } else { 'c' };
        println!("{}{}-{}{}{}{}{}", n, v, b, d, i, z, c);
    }

    pub fn cmd_disass<B: Bus>(&self, cpu: &mut Cpu<B>, addr: u16, count: u8) {
        let mut pc = addr;
        for _ in 0..count {
            let (text, len) = disassemble_at(pc, |a| cpu.read(a));
            println!("{:04X}  {}", pc, text);
            pc = pc.wrapping_add(len as u16);
        }
    }

    pub fn cmd_hexdump<B: Bus>(&self, cpu: &mut Cpu<B>, addr: u16, len: u8) {
        let mut offset = 0u16;
        while offset < len as u16 {
            let line_addr = addr.wrapping_add(offset);
            let remaining = (len as u16) - offset;
            let line_len = if remaining > 16 { 16 } else { remaining };

            let mut hex = String::new();
            let mut ascii = String::new();

            for i in 0..16u16 {
                if i < line_len {
                    let byte = cpu.read(line_addr.wrapping_add(i));
                    hex.push_str(&format!("{:02X} ", byte));
                    if byte >= 0x20 && byte <= 0x7E {
                        ascii.push(byte as char);
                    } else {
                        ascii.push('.');
                    }
                } else {
                    hex.push_str("   ");
                    ascii.push(' ');
                }
            }

            println!("{:04X}  {} |{}|", line_addr, hex, ascii);
            offset += line_len;
        }
    }

    pub fn cmd_set_breakpoint(&mut self, addr: u16) {
        let id = self.breakpoints.insert(addr);
        println!("Breakpoint {} set at ${:04X}", id, addr);
    }

    pub fn cmd_break_clear(&mut self, id: usize) {
        if self.breakpoints.remove_by_id(id) {
            println!("Breakpoint {} deleted", id);
        } else {
            println!("No breakpoint with id {}", id);
        }
    }

    pub fn cmd_break_list(&self) {
        let bps = self.breakpoints.list();
        if bps.is_empty() {
            println!("No breakpoints");
        } else {
            for bp in bps {
                println!("  {}  ${:04X}", bp.id, bp.addr);
            }
        }
    }

    pub fn cmd_trace(&self, count: Option<usize>) {
        let total = self.trace.len();
        if total == 0 {
            println!("No trace entries");
            return;
        }
        let n = count.unwrap_or(total).min(total);
        let start = total - n;
        for entry in self.trace.iter().skip(start) {
            println!(
                "{:04X}  {:<24} (+{} cycles)",
                entry.addr, entry.text, entry.cycles
            );
        }
    }

    pub fn execute<B: Bus>(&mut self, cmd: Command, cpu: &mut Cpu<B>) -> bool {
        match cmd {
            Command::Step => {
                self.cmd_step(cpu);
            }
            Command::Continue => {
                self.cmd_continue(cpu);
            }
            Command::Registers => {
                self.cmd_regs(cpu);
            }
            Command::Disassemble { addr, count } => {
                let a = addr.unwrap_or(cpu.pc);
                let n = count.unwrap_or(10);
                self.cmd_disass(cpu, a, n);
            }
            Command::Memory { addr, len } => {
                let a = addr.unwrap_or(0);
                let l = len.unwrap_or(128) as u8;
                self.cmd_hexdump(cpu, a, l);
            }
            Command::Break { addr } => {
                self.cmd_set_breakpoint(addr);
            }
            Command::BreakClear { id } => {
                self.cmd_break_clear(id);
            }
            Command::BreakList => {
                self.cmd_break_list();
            }
            Command::Trace { count } => {
                self.cmd_trace(count);
            }
            Command::Help => {
                Self::cmd_help();
            }
            Command::Quit => {
                return false;
            }
            Command::Unknown(input) => {
                println!("Unknown command: {}", input);
                println!("Type 'h' for help.");
            }
        }
        true
    }

    pub fn parse(input: &str) -> Command {
        let input = input.trim();
        if input.is_empty() {
            return Command::Unknown(input.to_string());
        }
        let mut parts = input.split_whitespace();
        let cmd = parts.next().unwrap();

        match cmd {
            "s" | "step" => Command::Step,
            "c" | "continue" => Command::Continue,
            "r" | "regs" => Command::Registers,
            "d" | "dis" | "disassemble" => {
                let addr = parts.next().and_then(|s| parse_addr(s));
                let count = parts.next().and_then(|s| s.parse::<u8>().ok());
                Command::Disassemble { addr, count }
            }
            "m" | "mem" | "memory" => {
                let addr = parts.next().and_then(|s| parse_addr(s));
                let len = parts.next().and_then(|s| parse_addr(s));
                Command::Memory { addr, len }
            }
            "b" => {
                let addr = parts.next().and_then(|s| parse_addr(s));
                match addr {
                    Some(a) => Command::Break { addr: a },
                    None => Command::Unknown(input.to_string()),
                }
            }
            "bc" => {
                let id = parts.next().and_then(|s| s.parse::<usize>().ok());
                match id {
                    Some(i) => Command::BreakClear { id: i },
                    None => Command::Unknown(input.to_string()),
                }
            }
            "bl" | "breaklist" => Command::BreakList,
            "t" | "trace" => {
                let count = parts.next().and_then(|s| s.parse::<usize>().ok());
                Command::Trace { count }
            }
            "q" | "quit" => Command::Quit,
            "h" | "help" => Command::Help,
            _ => Command::Unknown(input.to_string()),
        }
    }
}

fn parse_addr(s: &str) -> Option<u16> {
    let s = s.trim_start_matches('$');
    u16::from_str_radix(s, 16).ok()
}
