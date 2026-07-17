use crate::bus::Bus;

pub struct SimpleBus {
    memory: [u8; 0x10000],
}

impl SimpleBus {
    pub fn new() -> Self {
        Self {
            memory: [0; 0x10000],
        }
    }

    pub fn load(&mut self, data: &[u8], start: u16) {
        for (i, byte) in data.iter().enumerate() {
            let addr = start.wrapping_add(i as u16);
            self.memory[addr as usize] = *byte;
        }
    }
}

impl Bus for SimpleBus {
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
