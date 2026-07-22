use crate::nes::cartridge::Mirroring;
use crate::nes::palette;

/*
 * PPU (Picture Processing Unit) — minimal register-level definition
 * NES Dev Wiki: https://www.nesdev.org/wiki/PPU
 *
 * CPU-facing registers are at $2000-$2007, mirrored every 8 bytes to $3FFF.
 * Only the register-level interface is implemented here — no rendering.
 */

// PPUCTRL ($2000) bits
const CTRL_NMI_ENABLE: u8 = 0x80; // bit 7: NMI enable
const CTRL_BG_ADDR: u8 = 0x10; // bit 4: background pattern table (0: $0000, 1: $1000)
const CTRL_SPR_ADDR: u8 = 0x08; // bit 3: sprite pattern table (0: $0000, 1: $1000)
const CTRL_VRAM_INCR: u8 = 0x04; // bit 2: VRAM address increment (0: +1 across, 1: +32 down)

// PPUMASK ($2001) bits
const MASK_SHOW_BG: u8 = 0x08; // bit 3: show background
const MASK_SHOW_SPR: u8 = 0x10; // bit 4: show sprites

// PPUSTATUS ($2002) bits
const STATUS_OVERFLOW: u8 = 0x20; // bit 5: sprite overflow
const STATUS_SPR0_HIT: u8 = 0x40; // bit 6: sprite 0 hit
const STATUS_VBLANK: u8 = 0x80; // bit 7: vblank

const FB_WIDTH: usize = 256;
const FB_HEIGHT: usize = 240;

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
    render_scroll_x: usize,
    render_scroll_y: usize,
    render_base_nt: usize,

    // PPUDATA read buffer (NES PPU reads ahead one byte)
    read_buffer: u8,

    // FRAME_BUFFER
    frame_buffer: [u8; FB_WIDTH * FB_HEIGHT * 3],

    // Internal memory
    chr_rom: Vec<u8>,  // Pattern tables from cartridge CHR ROM
    vram: [u8; 2048],  // 2KB VRAM (nametables $2000-$2FFF)
    palette: [u8; 32], // Palette RAM ($3F00-$3F1F)
    oam: [u8; 256],    // OAM (Object Attribute Memory)

    // NMI output line (to CPU)
    pub nmi_pending: bool,

    // Nametable mirroring mode (from mapper)
    mirroring: Mirroring,
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
            frame_buffer: [0; FB_WIDTH * FB_HEIGHT * 3],
            scanline: 0,
            dot: 0,
            frame: 0,
            render_scroll_x: 0,
            render_scroll_y: 0,
            render_base_nt: 0,
            chr_rom,
            vram: [0; 2048],
            palette: [0; 32],
            oam: [0; 256],
            nmi_pending: false,
            mirroring: Mirroring::Horizontal,
        }
    }

    pub fn tick(&mut self) {
        self.dot += 1;

        if self.scanline < FB_HEIGHT as u16 && self.rendering_enabled() {
            if self.dot == 1 {
                self.latch_render_scroll();
                self.render_scanline_without_sprite_zero_hit(self.scanline as usize);
            }
            self.update_sprite_zero_hit_for_dot();
        }

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
                    self.t = (self.t & 0xFFE0) | (((val as u16) >> 3) & 0x1F);
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
            0x2000..=0x2FFF => {
                let idx = self.mirror_nt_addr(addr);
                self.vram[idx]
            }

            // Nametable mirrors ($3000-$3EFF)
            0x3000..=0x3EFF => {
                let idx = self.mirror_nt_addr(addr - 0x1000);
                self.vram[idx]
            }

            // Palette ($3F00-$3F1F) with mirrors
            0x3F00..=0x3FFF => {
                let mut index = (addr & 0x1F) as usize;
                // 镜像: $3F10/$3F14/$3F18/$3F1C → $3F00/$3F04/$3F08/$3F0C
                if index >= 0x10 && index.is_multiple_of(4) {
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
            // Pattern tables — write to CHR RAM (for CHR RAM mappers like MMC1)
            0x0000..=0x1FFF => {
                if (addr as usize) < self.chr_rom.len() {
                    self.chr_rom[addr as usize] = val;
                }
            }

            // Nametables ($2000-$2FFF)
            0x2000..=0x2FFF => {
                let idx = self.mirror_nt_addr(addr);
                self.vram[idx] = val;
            }

            // Nametable mirrors ($3000-$3EFF)
            0x3000..=0x3EFF => {
                let idx = self.mirror_nt_addr(addr - 0x1000);
                self.vram[idx] = val;
            }

            // Palette ($3F00-$3F1F) with mirrors
            0x3F00..=0x3FFF => {
                let mut index = (addr & 0x1F) as usize;
                if index >= 0x10 && index.is_multiple_of(4) {
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
            self.oam[(base + i as usize) & 0xFF] = page_data[i as usize];
        }
    }

    /// Check if background or sprite rendering is enabled
    pub fn rendering_enabled(&self) -> bool {
        (self.mask & MASK_SHOW_BG) != 0 || (self.mask & MASK_SHOW_SPR) != 0
    }

    fn latch_render_scroll(&mut self) {
        self.render_scroll_x = (((self.t & 0x001F) as usize) * 8) + self.x as usize;
        self.render_scroll_y =
            ((((self.t >> 5) & 0x001F) as usize) * 8) + (((self.t >> 12) & 0x0007) as usize);
        self.render_base_nt = ((self.t >> 10) & 0x03) as usize;
    }

    pub fn render_scanline(&mut self, py: usize) {
        self.latch_render_scroll();
        self.render_scanline_inner(py, true);
    }

    fn render_scanline_without_sprite_zero_hit(&mut self, py: usize) {
        self.render_scanline_inner(py, false);
    }

    fn render_scanline_inner(&mut self, py: usize, update_sprite_zero_hit: bool) {
        if py >= FB_HEIGHT {
            return;
        }

        let mut bg_opaque = [false; FB_WIDTH];

        if (self.mask & MASK_SHOW_BG) != 0 {
            self.render_background_scanline(py, &mut bg_opaque);
        } else {
            self.clear_scanline(py);
        }

        if (self.mask & MASK_SHOW_SPR) != 0 {
            self.render_sprite_scanline(py, &bg_opaque, update_sprite_zero_hit);
        }
    }

    fn clear_scanline(&mut self, py: usize) {
        let (r, g, b) = palette::SYSTEM_PALETTE[self.palette[0] as usize];
        for px in 0..FB_WIDTH {
            let offset = (py * FB_WIDTH + px) * 3;
            self.frame_buffer[offset] = r;
            self.frame_buffer[offset + 1] = g;
            self.frame_buffer[offset + 2] = b;
        }
    }

    pub fn render_background(&mut self) {
        self.latch_render_scroll();
        let mut bg_opaque = [false; FB_WIDTH];
        for py in 0..FB_HEIGHT {
            self.render_background_scanline(py, &mut bg_opaque);
        }
    }

    fn render_background_scanline(&mut self, py: usize, bg_opaque: &mut [bool; FB_WIDTH]) {
        for (px, opaque) in bg_opaque.iter_mut().enumerate() {
            let (palette_entry, is_opaque) = self.background_pixel(
                px,
                py,
                self.render_scroll_x,
                self.render_scroll_y,
                self.render_base_nt,
            );
            *opaque = is_opaque;

            let (r, g, b) = palette::SYSTEM_PALETTE[palette_entry as usize];
            let offset = (py * FB_WIDTH + px) * 3;
            self.frame_buffer[offset] = r;
            self.frame_buffer[offset + 1] = g;
            self.frame_buffer[offset + 2] = b;
        }
    }

    fn background_pixel(
        &self,
        px: usize,
        py: usize,
        scroll_x: usize,
        scroll_y: usize,
        base_nt: usize,
    ) -> (u8, bool) {
        let world_y = py + scroll_y;
        let nt_y = (world_y / 240) & 0x01;
        let tile_row = (world_y / 8) % 30;
        let fine_y = world_y % 8;

        let world_x = px + scroll_x;
        let nt_x = (world_x / 256) & 0x01;
        let tile_col = (world_x / 8) % 32;
        let fine_x = world_x % 8;
        let nt = base_nt ^ nt_x ^ (nt_y << 1);

        let nt_base = 0x2000 + (nt as u16) * 0x0400;
        let tile_addr_in_nt = nt_base + (tile_row * 32 + tile_col) as u16;
        let tile_index = self.ppu_read(tile_addr_in_nt) as u16;

        let attr_addr = nt_base + 0x03C0 + ((tile_row / 4) * 8 + (tile_col / 4)) as u16;
        let attr_byte = self.ppu_read(attr_addr);
        let palette_shift = ((tile_col % 4) / 2) * 2 + ((tile_row % 4) / 2) * 4;
        let palette_idx = (attr_byte >> palette_shift) & 0x03;

        let bank: u16 = if (self.ctrl & CTRL_BG_ADDR) != 0 {
            0x1000
        } else {
            0x0000
        };
        let pattern_addr = (bank + tile_index * 16) as usize;
        if pattern_addr + 15 >= self.chr_rom.len() {
            return (self.palette[0], false);
        }

        let lo_byte = self.chr_rom[pattern_addr + fine_y];
        let hi_byte = self.chr_rom[pattern_addr + 8 + fine_y];
        let bit = 7 - fine_x;
        let lo = (lo_byte >> bit) & 1;
        let hi = (hi_byte >> bit) & 1;
        let color_idx = (hi << 1) | lo;
        let palette_entry = if color_idx == 0 {
            self.palette[0]
        } else {
            self.palette[(palette_idx as usize) * 4 + color_idx as usize]
        };

        (palette_entry, color_idx != 0)
    }

    pub fn render_sprites(&mut self) {
        let bg_opaque = [false; FB_WIDTH];
        for py in 0..FB_HEIGHT {
            self.render_sprite_scanline(py, &bg_opaque, true);
        }
    }

    fn render_sprite_scanline(
        &mut self,
        py: usize,
        bg_opaque: &[bool; FB_WIDTH],
        update_sprite_zero_hit: bool,
    ) {
        if (self.mask & MASK_SHOW_SPR) == 0 {
            return;
        }

        let bank: u16 = if (self.ctrl & CTRL_SPR_ADDR) != 0 {
            0x1000
        } else {
            0x0000
        };

        // Iterate in reverse so sprite 0 (highest priority) draws last
        for i in (0..self.oam.len()).step_by(4).rev() {
            let y = self.oam[i] as i16 + 1;
            let tile_index = self.oam[i + 1] as u16;
            let attr = self.oam[i + 2];
            let x = self.oam[i + 3] as i16;

            if py < y as usize || py >= (y + 8) as usize {
                continue;
            }

            let flip_h = (attr & 0x40) != 0;
            let flip_v = (attr & 0x80) != 0;
            let palette_idx = (attr & 0x03) as usize;
            let priority_behind = (attr & 0x20) != 0;

            let tile_addr = (bank + tile_index * 16) as usize;
            if tile_addr + 15 >= self.chr_rom.len() {
                continue;
            }

            let ty = py as i16 - y;
            let sy = if flip_v { 7 - ty } else { ty } as usize;
            let lo_byte = self.chr_rom[tile_addr + sy];
            let hi_byte = self.chr_rom[tile_addr + 8 + sy];

            for tx in 0..8 {
                let sx = if flip_h { 7 - tx } else { tx };
                let bit = 7 - sx;
                let lo = (lo_byte >> bit) & 1;
                let hi = (hi_byte >> bit) & 1;
                let color_idx = (hi << 1) | lo;

                if color_idx == 0 {
                    continue; // transparent
                }

                let px = x + tx as i16;
                if !(0..FB_WIDTH as i16).contains(&px) {
                    continue;
                }

                let px = px as usize;
                if update_sprite_zero_hit && i == 0 && px != 255 && bg_opaque[px] {
                    self.status |= STATUS_SPR0_HIT;
                }

                if priority_behind && bg_opaque[px] {
                    continue;
                }

                let palette_entry = self.palette[0x11 + palette_idx * 4 + (color_idx - 1) as usize];
                let (r, g, b) = palette::SYSTEM_PALETTE[palette_entry as usize];
                let offset = (py * FB_WIDTH + px) * 3;
                self.frame_buffer[offset] = r;
                self.frame_buffer[offset + 1] = g;
                self.frame_buffer[offset + 2] = b;
            }
        }
    }

    fn update_sprite_zero_hit_for_dot(&mut self) {
        if (self.status & STATUS_SPR0_HIT) != 0 {
            return;
        }
        if (self.mask & (MASK_SHOW_BG | MASK_SHOW_SPR)) != (MASK_SHOW_BG | MASK_SHOW_SPR) {
            return;
        }
        if self.scanline >= FB_HEIGHT as u16 || self.dot == 0 || self.dot > FB_WIDTH as u16 {
            return;
        }

        let px = (self.dot - 1) as usize;
        if px == 255 {
            return;
        }

        let py = self.scanline as usize;
        let (_, bg_opaque) = self.background_pixel(
            px,
            py,
            self.render_scroll_x,
            self.render_scroll_y,
            self.render_base_nt,
        );
        if !bg_opaque {
            return;
        }

        if self.sprite_zero_opaque_at(px, py) {
            self.status |= STATUS_SPR0_HIT;
        }
    }

    fn sprite_zero_opaque_at(&self, px: usize, py: usize) -> bool {
        let y = self.oam[0] as i16 + 1;
        if py < y as usize || py >= (y + 8) as usize {
            return false;
        }

        let x = self.oam[3] as i16;
        if px < x as usize || px >= (x + 8) as usize {
            return false;
        }

        let bank: u16 = if (self.ctrl & CTRL_SPR_ADDR) != 0 {
            0x1000
        } else {
            0x0000
        };
        let tile_addr = (bank + self.oam[1] as u16 * 16) as usize;
        if tile_addr + 15 >= self.chr_rom.len() {
            return false;
        }

        let attr = self.oam[2];
        let flip_h = (attr & 0x40) != 0;
        let flip_v = (attr & 0x80) != 0;
        let tx = px as i16 - x;
        let ty = py as i16 - y;
        let sx = if flip_h { 7 - tx } else { tx } as usize;
        let sy = if flip_v { 7 - ty } else { ty } as usize;
        let bit = 7 - sx;
        let lo = (self.chr_rom[tile_addr + sy] >> bit) & 1;
        let hi = (self.chr_rom[tile_addr + 8 + sy] >> bit) & 1;

        ((hi << 1) | lo) != 0
    }

    /// Get current scanline
    pub fn scanline(&self) -> u16 {
        self.scanline
    }

    /// Get current dot
    pub fn dot(&self) -> u16 {
        self.dot
    }

    /// Get completed frame count
    pub fn frame(&self) -> u64 {
        self.frame
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

    /// Get PPUMASK value
    pub fn mask(&self) -> u8 {
        self.mask
    }

    /// Get PPUSTATUS value
    pub fn status(&self) -> u8 {
        self.status
    }

    /// Set nametable mirroring mode (called by Bus from mapper)
    pub fn set_mirroring(&mut self, m: Mirroring) {
        self.mirroring = m;
    }

    /// Translate VRAM address with current mirroring
    fn mirror_nt_addr(&self, addr: u16) -> usize {
        let nt = ((addr >> 10) & 0x03) as usize;
        let offset = (addr & 0x03FF) as usize;
        match self.mirroring {
            Mirroring::Horizontal => match nt {
                0 | 1 => offset,
                2 | 3 => 0x400 + offset,
                _ => unreachable!(),
            },
            Mirroring::Vertical => match nt {
                0 | 2 => offset,
                1 | 3 => 0x400 + offset,
                _ => unreachable!(),
            },
            Mirroring::FourScreen => (addr as usize & 0x0FFF) % 2048,
            Mirroring::OneScreenA => offset,
            Mirroring::OneScreenB => 0x400 + offset,
        }
    }

    /// Copy t to v (used at end of vblank / start of rendering)
    pub fn copy_t_to_v(&mut self) {
        self.v = self.t;
    }

    /// Get the frame buffer slice (RGB bytes, 256x240)
    pub fn frame_buffer(&self) -> &[u8] {
        &self.frame_buffer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set_tile_color(ppu: &mut Ppu, tile: usize, color_idx: u8) {
        let base = tile * 16;
        let lo = if (color_idx & 0x01) != 0 { 0xFF } else { 0x00 };
        let hi = if (color_idx & 0x02) != 0 { 0xFF } else { 0x00 };
        for row in 0..8 {
            ppu.chr_rom[base + row] = lo;
            ppu.chr_rom[base + 8 + row] = hi;
        }
    }

    #[test]
    fn render_background_uses_horizontal_scroll() {
        let mut ppu = Ppu::new(vec![0; 0x2000]);
        set_tile_color(&mut ppu, 0, 1);
        set_tile_color(&mut ppu, 1, 2);
        ppu.ppu_write(0x2000, 0);
        ppu.ppu_write(0x2001, 1);
        ppu.ppu_write(0x3F01, 0x01);
        ppu.ppu_write(0x3F02, 0x02);

        ppu.write_register(0x2005, 8);
        ppu.write_register(0x2005, 0);
        ppu.render_background();

        let expected = palette::SYSTEM_PALETTE[0x02];
        assert_eq!(
            &ppu.frame_buffer()[0..3],
            &[expected.0, expected.1, expected.2]
        );
    }

    #[test]
    fn tick_renders_visible_scanline_at_end_of_line() {
        let mut ppu = Ppu::new(vec![0; 0x2000]);
        set_tile_color(&mut ppu, 0, 1);
        ppu.ppu_write(0x2000, 0);
        ppu.ppu_write(0x3F01, 0x01);
        ppu.write_register(0x2001, MASK_SHOW_BG);

        for _ in 0..341 {
            ppu.tick();
        }

        let expected = palette::SYSTEM_PALETTE[0x01];
        assert_eq!(
            &ppu.frame_buffer()[0..3],
            &[expected.0, expected.1, expected.2]
        );
        assert_eq!(ppu.scanline(), 1);
    }

    #[test]
    fn render_scanline_sets_sprite_zero_hit_on_opaque_overlap() {
        let mut ppu = Ppu::new(vec![0; 0x2000]);
        set_tile_color(&mut ppu, 0, 1);
        ppu.ppu_write(0x2000, 0);
        ppu.ppu_write(0x3F01, 0x01);
        ppu.write_register(0x2001, MASK_SHOW_BG | MASK_SHOW_SPR);

        ppu.oam[0] = 0;
        ppu.oam[1] = 0;
        ppu.oam[2] = 0;
        ppu.oam[3] = 0;

        ppu.render_scanline(1);

        assert_ne!(ppu.status & STATUS_SPR0_HIT, 0);
    }

    #[test]
    fn tick_sets_sprite_zero_hit_before_scanline_ends() {
        let mut ppu = Ppu::new(vec![0; 0x2000]);
        set_tile_color(&mut ppu, 0, 1);
        ppu.ppu_write(0x2000, 0);
        ppu.ppu_write(0x3F01, 0x01);
        ppu.write_register(0x2001, MASK_SHOW_BG | MASK_SHOW_SPR);

        ppu.oam[0] = 0;
        ppu.oam[1] = 0;
        ppu.oam[2] = 0;
        ppu.oam[3] = 8;

        for _ in 0..351 {
            ppu.tick();
        }

        assert_eq!(ppu.scanline(), 1);
        assert_ne!(ppu.status & STATUS_SPR0_HIT, 0);
    }
}
