use std::collections::{HashSet, VecDeque};

mod commands;
mod disass;

use crate::bus::Bus;
use crate::cpu::Cpu;

pub fn run<B: Bus>(cpu: &mut Cpu<B>) {}

pub struct TraceEntry {
    pub addr: u16,
    pub text: String,
    pub cycles: u8,
}

pub struct Monitor {
    breakpoints: HashSet<u16>,
    trace: VecDeque<TraceEntry>,
}

impl Monitor {
    pub fn new(&mut self) -> Self {
        Self {
            breakpoints: HashSet::new(),
            trace: VecDeque::new(),
        }
    }
}
