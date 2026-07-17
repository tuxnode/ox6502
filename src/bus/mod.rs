pub trait Bus {
    fn cpu_read(&mut self, addr: u16) -> u8;
    fn cpu_write(&mut self, addr: u16, val: u8);
    fn ppu_read(&mut self, addr: u16) -> u8;
    fn ppu_write(&mut self, addr: u16, val: u8);

    /// Called after each CPU step. Returns extra cycles to account for (e.g. DMA).
    /// Also checks for pending interrupts (NMI).
    fn tick(&mut self) -> TickResult {
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
