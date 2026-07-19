pub trait Bus {
    fn cpu_read(&mut self, addr: u16) -> u8;
    fn cpu_write(&mut self, addr: u16, val: u8);
    fn ppu_read(&mut self, addr: u16) -> u8;
    fn ppu_write(&mut self, addr: u16, val: u8);

    /// Called after each CPU step. `cpu_cycles` is the number of CPU cycles the
    /// just-executed instruction took. Returns extra cycles to account for (e.g. DMA).
    /// Also checks for pending interrupts (NMI).
    fn tick(&mut self, _cpu_cycles: u8) -> TickResult {
        TickResult::default()
    }
}

pub struct TickResult {
    pub extra_cycles: u32,
    pub nmi: bool,
}

impl Default for TickResult {
    fn default() -> Self {
        Self {
            extra_cycles: 0,
            nmi: false,
        }
    }
}

pub mod nes;
pub mod simple;
