mod commands;
mod disass;

use crate::bus::Bus;
use crate::cpu::Cpu;

pub fn run<B: Bus>(cpu: &mut Cpu<B>) {}
