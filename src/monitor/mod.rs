use std::collections::VecDeque;
use std::io::{self, Write};

mod commands;
mod disass;

use crate::bus::Bus;
use crate::cpu::Cpu;
use commands::Command;

pub fn run<B: Bus>(cpu: &mut Cpu<B>) {
    let mut monitor = Monitor::new();

    println!("ox6502 monitor. Type 'h' for help.");
    monitor.print_current(cpu);

    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            break;
        }

        let cmd = Monitor::parse(&input);
        if !monitor.execute(cmd, cpu) {
            println!("Bye.");
            break;
        }
    }
}

pub struct TraceEntry {
    pub addr: u16,
    pub text: String,
    pub cycles: u8,
}

pub struct Breakpoint {
    pub id: usize,
    pub addr: u16,
}

pub struct Breakpoints {
    inner: Vec<Breakpoint>,
    next_id: usize,
}

impl Breakpoints {
    pub fn new() -> Self {
        Self {
            inner: Vec::new(),
            next_id: 1,
        }
    }

    pub fn insert(&mut self, addr: u16) -> usize {
        let id = self.next_id;
        self.inner.push(Breakpoint { id, addr });
        self.next_id += 1;
        id
    }

    pub fn remove_by_id(&mut self, id: usize) -> bool {
        let before = self.inner.len();
        self.inner.retain(|bp| bp.id != id);
        self.inner.len() < before
    }

    pub fn contains(&self, addr: u16) -> bool {
        self.inner.iter().any(|bp| bp.addr == addr)
    }

    pub fn list(&self) -> &[Breakpoint] {
        &self.inner
    }
}

pub struct Monitor {
    pub breakpoints: Breakpoints,
    pub trace: VecDeque<TraceEntry>,
}

impl Monitor {
    pub fn new() -> Self {
        Self {
            breakpoints: Breakpoints::new(),
            trace: VecDeque::new(),
        }
    }
}
