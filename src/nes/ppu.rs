/**
 * PPU (Picture Processing Unit) — minimal register-level definition
 * NES Dev Wiki: https://www.nesdev.org/wiki/PPU
 *
 * CPU-facing registers are at $2000-$2007, mirrored every 8 bytes to $3FFF.
 * Only the register-level interface is implemented here — no rendering.
 */

// PPUCTRL ($2000) bits
const CTRL_NMI_ENABLE: u8 = 0x80; // bit 7: NMI enable
const CTRL_MASTER_SLAVE: u8 = 0x40; // bit 6: PPU master/slave
const CTRL_SPR_SIZE: u8 = 0x20; // bit 5: sprite size (0: 8x8, 1: 8x16)
const CTRL_BG_ADDR: u8 = 0x10; // bit 4: background pattern table (0: $0000, 1: $1000)
const CTRL_SPR_ADDR: u8 = 0x08; // bit 3: sprite pattern table (0: $0000, 1: $1000)
const CTRL_VRAM_INCR: u8 = 0x04; // bit 2: VRAM address increment (0: +1 across, 1: +32 down)

// PPUMASK ($2001) bits
const MASK_GREYSCALE: u8 = 0x01; // bit 0: greyscale
const MASK_SHOW_BG_LEFT: u8 = 0x02; // bit 1: show background in leftmost 8px
const MASK_SHOW_SPR_LEFT: u8 = 0x04; // bit 2: show sprites in leftmost 8px
const MASK_SHOW_BG: u8 = 0x08; // bit 3: show background
const MASK_SHOW_SPR: u8 = 0x10; // bit 4: show sprites

// PPUSTATUS ($2002) bits
const STATUS_OVERFLOW: u8 = 0x20; // bit 5: sprite overflow
const STATUS_SPR0_HIT: u8 = 0x40; // bit 6: sprite 0 hit
const STATUS_VBLANK: u8 = 0x80; // bit 7: vblank

pub struct Ppu {
    // CPU-facing registers
    ctrl: u8,         // $2000 PPUCTRL (write only)
    mask: u8,         // $2001 PPUMASK (write only)
    status: u8,       // $2002 PPUSTATUS (read only)
    pub oam_addr: u8, // $2003 OAMADDR (write only)

    // Loopy registers (internal VRAM address system)
    // v: current VRAM address (15-bit, used for reads/writes)
    // t: temporary VRAM address (15-bit, used for scroll/addr setup)
    // x: fine X scroll (3-bit)
    // w: write toggle (1-bit, alternates between first/second write)
    v: u16,  // current VRAM address
    t: u16,  // temporary VRAM address
    x: u8,   // fine X scroll (0-7)
    w: bool, // write toggle

    scanline: u16, // scanline index
    dot: u16,      // point
    frame: u64,    // frame counter

    // PPUDATA read buffer (NES PPU reads ahead one byte)
    read_buffer: u8,

    // Internal memory
    chr_rom: Vec<u8>,  // Pattern tables from cartridge CHR ROM
    vram: [u8; 2048],  // 2KB VRAM (nametables $2000-$2FFF)
    palette: [u8; 32], // Palette RAM ($3F00-$3F1F)
    oam: [u8; 256],    // OAM (Object Attribute Memory)

    // NMI output line (to CPU)
    pub nmi_pending: bool,
}

impl Ppu {
    pub fn new(chr_rom: Vec<u8>) -> Self {
        Self {
            ctrl: 0,
            mask: 0,
            status: 0,
            oam_addr: 0,
            v: 0,
            t: 0,
            x: 0,
            w: false,
            read_buffer: 0,
            scanline: 0,
            dot: 0,
            frame: 0,
            chr_rom,
            vram: [0; 2048],
            palette: [0; 32],
            oam: [0; 256],
            nmi_pending: false,
        }
    }

    pub fn tick(&mut self) {
        self.dot += 1;

        if self.dot > 340 {
            self.dot = 0;
            self.scanline += 1;

            if self.scanline == 241 {
                self.set_vblank();
            }

            if self.scanline == 262 {
                self.scanline = 0;
                self.frame += 1;
                self.clear_vblank();
            }
        }
    }

    /// CPU read from PPU register ($2000-$2007, mirrored)
    pub fn read_register(&mut self, addr: u16) -> u8 {
        match addr & 0x07 {
            // $2000 PPUCTRL — write only, reading returns open bus
            0 => self.ctrl,

            // $2001 PPUMASK — write only, reading returns open bus
            1 => self.mask,

            // $2002 PPUSTATUS — read clears vblank flag and resets write latch
            2 => {
                let val = self.status;
                self.status &= !STATUS_VBLANK;
                self.w = false;
                val
            }

            // $2003 OAMADDR — write only
            3 => self.oam_addr,

            // $2004 OAMDATA — read from OAM at current address
            4 => self.oam[self.oam_addr as usize],

            // $2005 PPUSCROLL — write only
            5 => 0,

            // $2006 PPUADDR — write only
            6 => 0,

            // $2007 PPUDATA — read VRAM with buffering
            7 => {
                let addr = self.v;
                let val = self.read_buffer;
                self.read_buffer = self.ppu_read(addr);
                self.increment_vram_addr();
                val
            }

            _ => 0,
        }
    }

    /// CPU write to PPU register ($2000-$2007, mirrored)
    pub fn write_register(&mut self, addr: u16, val: u8) {
        match addr & 0x07 {
            // $2000 PPUCTRL
            0 => {
                let old_nmi = self.ctrl & CTRL_NMI_ENABLE;
                self.ctrl = val;
                // Bits 0-1 of ctrl go into t bits 10-11
                self.t = (self.t & 0xF3FF) | ((val as u16 & 0x03) << 10);
                // 如果之前 NMI 禁用，现在启用，且当前处于 vblank，触发 NMI
                if old_nmi == 0
                    && (val & CTRL_NMI_ENABLE) != 0
                    && (self.status & STATUS_VBLANK) != 0
                {
                    self.nmi_pending = true;
                }
            }

            // $2001 PPUMASK
            1 => self.mask = val,

            // $2002 PPUSTATUS — write only, writing has no effect
            2 => {}

            // $2003 OAMADDR
            3 => self.oam_addr = val,

            // $2004 OAMDATA — write to OAM at current address, then increment
            4 => {
                self.oam[self.oam_addr as usize] = val;
                self.oam_addr = self.oam_addr.wrapping_add(1);
            }

            // $2005 PPUSCROLL — two writes sharing latch with $2006
            // First write: coarse X (bits 0-4) and fine X (bits 0-2)
            // Second write: coarse Y (bits 0-4) and fine Y (bits 0-2)
            5 => {
                if !self.w {
                    // First write: X scroll
                    self.t = (self.t & 0xFFE0) | ((val as u16) & 0x1F);
                    self.x = val & 0x07;
                    self.w = true;
                } else {
                    // Second write: Y scroll
                    self.t = (self.t & 0x0C1F)
                        | ((val as u16 & 0x07) << 12)      // fine Y -> bits 12-14
                        | ((val as u16 & 0xF8) << 2); // coarse Y -> bits 5-9
                    self.w = false;
                }
            }

            // $2006 PPUADDR — two writes sharing latch with $2005
            // First write: upper 6 bits of address
            // Second write: lower 8 bits of address, then copy t -> v
            6 => {
                if !self.w {
                    // First write: upper address bits
                    self.t = (self.t & 0x00FF) | ((val as u16 & 0x3F) << 8);
                    self.w = true;
                } else {
                    // Second write: lower address bits, then copy t -> v
                    self.t = (self.t & 0xFF00) | (val as u16);
                    self.v = self.t;
                    self.w = false;
                }
            }

            // $2007 PPUDATA — write VRAM, then auto-increment address
            7 => {
                let addr = self.v;
                self.ppu_write(addr, val);
                self.increment_vram_addr();
            }

            _ => {}
        }
    }

    /// 设置 vblank 标志（由 PPU 扫描线逻辑调用）
    pub fn set_vblank(&mut self) {
        self.status |= STATUS_VBLANK;
        if (self.ctrl & CTRL_NMI_ENABLE) != 0 {
            self.nmi_pending = true;
        }
    }

    /// 清除 vblank 标志（每帧开始时调用）
    pub fn clear_vblank(&mut self) {
        self.status &= !STATUS_VBLANK;
        self.status &= !STATUS_OVERFLOW;
        self.status &= !STATUS_SPR0_HIT;
    }

    /// 读取并清除 NMI pending 状态（CPU 每周期检查）
    pub fn take_nmi(&mut self) -> bool {
        if self.nmi_pending {
            self.nmi_pending = false;
            true
        } else {
            false
        }
    }

    /// Read byte from PPU address space
    pub fn ppu_read(&self, addr: u16) -> u8 {
        match addr {
            // Pattern tables ($0000-$1FFF) — from cartridge CHR ROM
            0x0000..=0x1FFF => {
                if (addr as usize) < self.chr_rom.len() {
                    self.chr_rom[addr as usize]
                } else {
                    0
                }
            }

            // Nametables ($2000-$2FFF) — internal VRAM with mirroring
            0x2000..=0x2FFF => self.vram[(addr & 0x0FFF) as usize],

            // Nametable mirrors ($3000-$3EFF)
            0x3000..=0x3EFF => self.vram[((addr - 0x1000) & 0x0FFF) as usize],

            // Palette ($3F00-$3F1F) with mirrors
            0x3F00..=0x3FFF => {
                let mut index = (addr & 0x1F) as usize;
                // 镜像: $3F10/$3F14/$3F18/$3F1C → $3F00/$3F04/$3F08/$3F0C
                if index >= 0x10 && index % 4 == 0 {
                    index -= 0x10;
                }
                self.palette[index]
            }

            _ => 0,
        }
    }

    /// Write byte to PPU address space
    pub fn ppu_write(&mut self, addr: u16, val: u8) {
        match addr {
            // Pattern tables — read only, ignore writes
            0x0000..=0x1FFF => {}

            // Nametables ($2000-$2FFF)
            0x2000..=0x2FFF => {
                self.vram[(addr & 0x0FFF) as usize] = val;
            }

            // Nametable mirrors ($3000-$3EFF)
            0x3000..=0x3EFF => {
                self.vram[((addr - 0x1000) & 0x0FFF) as usize] = val;
            }

            // Palette ($3F00-$3F1F) with mirrors
            0x3F00..=0x3FFF => {
                let mut index = (addr & 0x1F) as usize;
                if index >= 0x10 && index % 4 == 0 {
                    index -= 0x10;
                }
                self.palette[index] = val;
            }

            _ => {}
        }
    }

    /// Auto-increment VRAM address by 1 or 32 (based on PPUCTRL bit 2)
    fn increment_vram_addr(&mut self) {
        let increment = if (self.ctrl & CTRL_VRAM_INCR) != 0 {
            32
        } else {
            1
        };
        self.v = self.v.wrapping_add(increment);
    }

    /// OAM DMA: copy 256 bytes to OAM starting at current oam_addr
    /// Called when CPU writes to $4014
    pub fn dma_write_oam(&mut self, page_data: &[u8; 256]) {
        let base = self.oam_addr as usize;
        for i in 0..256u16 {
            self.oam[((base + i as usize) & 0xFF)] = page_data[i as usize];
        }
    }

    /// Check if background or sprite rendering is enabled
    pub fn rendering_enabled(&self) -> bool {
        (self.mask & MASK_SHOW_BG) != 0 || (self.mask & MASK_SHOW_SPR) != 0
    }

    /// Get current VRAM address (for rendering)
    pub fn vram_addr(&self) -> u16 {
        self.v
    }

    /// Get fine X scroll (for rendering)
    pub fn fine_x(&self) -> u8 {
        self.x
    }

    /// Get PPUCTRL value (for pattern table selection)
    pub fn ctrl(&self) -> u8 {
        self.ctrl
    }

    /// Copy t to v (used at end of vblank / start of rendering)
    pub fn copy_t_to_v(&mut self) {
        self.v = self.t;
    }
}
