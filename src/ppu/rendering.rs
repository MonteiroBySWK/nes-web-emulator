use super::*;
use crate::rom::Mirroring;
use super::colors::convert_color;

pub struct PPU {
    // MMIO registers (memory mapped I/O)
    pub ctrl: u8,    // $2000 PPUCTRL
    pub mask: u8,    // $2001 PPUMASK
    pub(crate) status: u8,      // $2002 PPUSTATUS
    pub(crate) oam_addr: u8,    // $2003 OAMADDR
    pub oam_data: [u8; 256], // $2004 OAMDATA array
    pub(crate) scroll: u8,      // $2005 PPUSCROLL
    pub(crate) addr: u16,       // $2006 PPUADDR
    pub(crate) data: u8,        // $2007 PPUDATA
    
    // Internal PPU registers
    pub(crate) v: u16,  // Current VRAM address (15 bits)
    pub(crate) t: u16,  // Temporary VRAM address (15 bits)
    pub(crate) x: u8,   // Fine X scroll (3 bits)
    pub(crate) w: bool, // Write toggle (1 bit)
    
    // Internal data buffer for reads
    read_buffer: u8,

    // Memory
    vram: [u8; 0x4000],       // VRAM (16KB)
    palette: [u8; 32],        // Palette RAM (32 bytes)
    oam: [u8; 256],          // Primary OAM (256 bytes)
    pub(crate) secondary_oam: [u8; 32],  // Secondary OAM (32 bytes)

    // Rendering state
    pub(crate) cycle: u16,      // 0-340
    pub(crate) scanline: u16,   // 0-261
    pub(crate) frame: u64,      // Frame counter
    
    // Background shift registers
    bg_shifter_pattern_lo: u16,
    bg_shifter_pattern_hi: u16,
    bg_shifter_attrib_lo: u16,
    bg_shifter_attrib_hi: u16,
    bg_next_tile_id: u8,
    bg_next_tile_attrib: u8,
    bg_next_tile_lsb: u8, 
    bg_next_tile_msb: u8,

    // Sprite rendering state
    pub(crate) sprite_count: usize,
    sprite_patterns: [u16; 8],
    sprite_positions: [u8; 8],
    sprite_priorities: [u8; 8],
    sprite_indexes: [u8; 8],
    pub(crate) sprite_zero_hit_possible: bool,
    sprite_zero_being_rendered: bool,

    // Output
    framebuffer: [u8; 256 * 240 * 3],
    
    // NMI flags
    pub(crate) nmi_occurred: bool,
    pub(crate) nmi_output: bool,
    nmi_previous: bool,
    nmi_delay: u8,

    // Additional state
    mirroring: Mirroring,
    pub(crate) odd_frame: bool,  // Novo campo para controlar frames par/ímpar
    pub(crate) rendering_enabled: bool, // Novo campo para verificar se rendering está ativo
}

// NOVO: Struct para resultado do avanço, inspirado em ppu2.
pub struct StepResult {
    pub new_frame: bool,
    pub vblank_nmi: bool,
}

impl PPU {
    pub fn step(&mut self) -> StepResult {
        let mut result = StepResult {
            new_frame: false,
            vblank_nmi: false,
        };

        // Atualiza estado de rendering
        self.rendering_enabled = self.is_rendering_enabled();

        // Manejo especial do último ciclo do frame para frames ímpares
        if self.rendering_enabled && self.odd_frame && self.scanline == 261 && self.cycle == 339 {
            self.cycle = 0;
            self.scanline = 0;
            self.odd_frame = !self.odd_frame;
            self.frame += 1;
            result.new_frame = true;
            return result;
        }

        // Add actual pixel rendering during visible scanlines
        if self.scanline < 240 && self.cycle > 0 && self.cycle <= 256 {
            self.render_pixel();
        }

        // Pre-render scanline (261)
        if self.scanline == 261 {
            if self.cycle == 1 {
                self.status &= !0xE0; // Clear VBlank, sprite 0 hit, sprite overflow
                self.nmi_occurred = false;
            }

            // Transferência vertical durante pré-render
            if self.cycle >= 280 && self.cycle <= 304 && self.rendering_enabled {
                self.transfer_address_y();
            }
        }

        // Clear frame at start of new frame
        if self.scanline == 0 && self.cycle == 0 {
            // Clear framebuffer
            self.framebuffer = [0; 256 * 240 * 3];
        }

        // Scanlines visíveis (0-239) e pré-render
        if self.scanline <= 239 || self.scanline == 261 {
            // Ciclos para fetch de tiles e renderização
            match self.cycle {
                0 => {} // Idle cycle
                1..=256 => {
                    self.update_shifters();
                    
                    match (self.cycle - 1) % 8 {
                        0 => self.fetch_nametable_byte(),
                        2 => self.fetch_attribute_byte(),
                        4 => self.fetch_pattern_low(),
                        6 => self.fetch_pattern_high(),
                        7 => self.increment_scroll_x(),
                        _ => {}
                    }

                    if self.cycle == 256 {
                        self.increment_scroll_y();
                    }
                }
                257 => {
                    self.transfer_address_x();
                    if self.scanline < 240 {
                        self.load_sprites_for_next_scanline();
                    }
                }
                321..=336 => {
                    self.update_shifters();
                    
                    match (self.cycle - 321) % 8 {
                        0 => self.fetch_nametable_byte(),
                        2 => self.fetch_attribute_byte(),
                        4 => self.fetch_pattern_low(),
                        6 => self.fetch_pattern_high(),
                        _ => {}
                    }
                }
                337..=340 => { /* Dois NT bytes dummy */ }
                _ => {}
            }
        }

        // VBlank (scanlines 241-260)
        if self.scanline == 241 && self.cycle == 1 {
            self.status |= 0x80; // Set VBlank flag
            self.nmi_occurred = true;
            if self.nmi_output {
                result.vblank_nmi = true;
            }
        }

        // Avança ciclo/scanline
        self.cycle += 1;
        if self.cycle > 340 {
            self.cycle = 0;
            self.scanline += 1;
            if self.scanline > 261 {
                self.scanline = 0;
                self.odd_frame = !self.odd_frame;
                self.frame += 1;
                result.new_frame = true;
            }
        }

        result
    }

    // Adicionar métodos que outros módulos precisam
    pub(crate) fn read_ppu_memory(&self, addr: u16) -> u8 {
        let addr = addr & 0x3fff;
        match addr {
            0x0000..=0x1fff => self.vram[addr as usize],
            0x2000..=0x3eff => {
                let mirrored_addr = self.mirror_nametable_address(addr);
                self.vram[mirrored_addr as usize]
            },
            0x3f00..=0x3fff => {
                let addr = addr & 0x1f;
                let addr = match addr {
                    0x10 => 0x00,
                    0x14 => 0x04,
                    0x18 => 0x08,
                    0x1c => 0x0c,
                    _ => addr
                };
                self.palette[addr as usize]
            },
            _ => 0
        }
    }

    pub(crate) fn write_ppu_memory(&mut self, addr: u16, data: u8) {
        let addr = addr & 0x3fff;
        match addr {
            0x0000..=0x1fff => self.vram[addr as usize] = data,
            0x2000..=0x3eff => {
                let mirrored_addr = self.mirror_nametable_address(addr);
                self.vram[mirrored_addr as usize] = data;
            },
            0x3f00..=0x3fff => {
                let addr = addr & 0x1f;
                let addr = match addr {
                    0x10 => 0x00,
                    0x14 => 0x04,
                    0x18 => 0x08,
                    0x1c => 0x0c,
                    _ => addr
                };
                self.palette[addr as usize] = data;
            },
            _ => {}
        }
    }

    pub(crate) fn is_rendering_enabled(&self) -> bool {
        (self.mask & 0x18) != 0 // Background ou sprites habilitados
    }

    // Adicionar métodos públicos faltantes
    pub fn read_register(&mut self, address: u16) -> u8 {
        match address {
            0x2000 => 0,
            0x2001 => 0,
            0x2002 => {
                let result = self.status;
                self.status &= 0x7f;
                self.w = false;
                result
            },
            0x2003 => 0,
            0x2004 => self.oam_data[self.oam_addr as usize],
            0x2005 => 0,
            0x2006 => 0,
            0x2007 => self.read_data(),
            _ => 0
        }
    }

    pub fn write_register(&mut self, address: u16, value: u8) {
        match address {
            0x2000 => self.write_control(value),
            0x2001 => self.write_mask(value),
            0x2002 => {},
            0x2003 => self.oam_addr = value,
            0x2004 => {
                self.oam_data[self.oam_addr as usize] = value;
                self.oam_addr = self.oam_addr.wrapping_add(1);
            },
            0x2005 => self.write_scroll(value),
            0x2006 => self.write_address(value),
            0x2007 => self.write_data(value),
            _ => {}
        }
    }

    pub fn get_framebuffer(&self) -> &[u8] {
        &self.framebuffer
    }

    pub fn get_all_registers(&self) -> (u8, u8, u8, u8, u8, u8, u16) {
        (self.ctrl, self.mask, self.status, self.oam_addr, 
         self.scroll, self.addr as u8, self.addr)
    }

    pub fn set_mirroring(&mut self, mirroring: Mirroring) {
        self.mirroring = mirroring;
    }

    pub fn load_chr_data(&mut self, data: &[u8]) {
        let len = data.len().min(self.vram.len());
        self.vram[..len].copy_from_slice(&data[..len]);
    }

    // Métodos privados auxiliares
    fn read_data(&mut self) -> u8 {
        let addr = self.v & 0x3FFF;
        let data = self.read_buffer;
        self.read_buffer = self.read_ppu_memory(addr);
        
        if addr >= 0x3F00 {
            self.read_buffer = self.read_ppu_memory(addr - 0x1000);
            return self.read_ppu_memory(addr);
        }
        
        self.v += if (self.ctrl & 0x04) != 0 { 32 } else { 1 };
        data
    }

    fn write_data(&mut self, value: u8) {
        let addr = self.v & 0x3FFF;
        self.write_ppu_memory(addr, value);
        self.v += if (self.ctrl & 0x04) != 0 { 32 } else { 1 };
    }

    pub(crate) fn write_control(&mut self, value: u8) {
        self.ctrl = value;
        self.t = (self.t & 0xf3ff) | (((value as u16) & 0x03) << 10);
        self.nmi_output = (value & 0x80) != 0;
        if self.nmi_output && self.nmi_occurred {
            self.trigger_nmi();
        }
    }

    pub(crate) fn write_mask(&mut self, value: u8) {
        self.mask = value;
        self.rendering_enabled = self.is_rendering_enabled();
    }

    pub(crate) fn write_scroll(&mut self, value: u8) {
        if (!self.w) {
            self.t = (self.t & 0xffe0) | ((value as u16) >> 3);
            self.x = value & 0x07;
            self.w = true;
        } else {
            self.t = (self.t & 0x8fff) | (((value as u16) & 0x07) << 12);
            self.t = (self.t & 0xfc1f) | (((value as u16) & 0xf8) << 2);
            self.w = false;
        }
    }

    pub(crate) fn write_address(&mut self, value: u8) {
        if (!self.w) {
            self.t = (self.t & 0x00ff) | (((value as u16) & 0x3f) << 8);
            self.w = true;
        } else {
            self.t = (self.t & 0xff00) | (value as u16);
            self.v = self.t;
            self.w = false;
        }
    }

    // Método render_pixel completo
    fn render_pixel(&mut self) {
        if self.cycle == 0 || self.cycle > 256 || self.scanline >= 240 {
            return;
        }

        let x = (self.cycle - 1) as usize;
        let y = self.scanline as usize;

        let mut bg_pixel = 0u8;
        let mut bg_palette = 0u8;

        if (self.mask & 0x08) != 0 {
            let bit_mux = 0x8000 >> self.x;
            
            let p0 = if (self.bg_shifter_pattern_lo & bit_mux) != 0 { 1 } else { 0 };
            let p1 = if (self.bg_shifter_pattern_hi & bit_mux) != 0 { 2 } else { 0 };
            bg_pixel = p0 | p1;

            let pal0 = if (self.bg_shifter_attrib_lo & bit_mux) != 0 { 1 } else { 0 };
            let pal1 = if (self.bg_shifter_attrib_hi & bit_mux) != 0 { 2 } else { 0 };
            bg_palette = pal0 | pal1;
        }

        let mut sprite_pixel = 0u8;
        let mut sprite_palette = 0u8;
        let mut sprite_priority = false;

        if (self.mask & 0x10) != 0 {
            for i in 0..self.sprite_count {
                let offset = (self.cycle - 1) as i16 - self.sprite_positions[i] as i16;
                
                if offset >= 0 && offset < 8 {
                    let shift = 7 - offset;
                    let pattern = self.sprite_patterns[i];
                    
                    let pixel = ((pattern >> shift) & 0x01) | 
                               (((pattern >> 8) >> shift) & 0x01) << 1;

                    if pixel != 0 {
                        sprite_pixel = pixel as u8;
                        sprite_palette = 4 + ((self.secondary_oam[i * 4 + 2] & 0x03) as u8);
                        sprite_priority = (self.secondary_oam[i * 4 + 2] & 0x20) == 0;
                        break;
                    }
                }
            }
        }

        let pixel_color = if sprite_pixel == 0 {
            let palette_addr = if bg_pixel == 0 { 0 } else { (bg_palette << 2) | bg_pixel };
            self.read_ppu_memory(0x3F00 | palette_addr as u16)
        } else if bg_pixel == 0 {
            self.read_ppu_memory(0x3F00 | (sprite_palette << 2 | sprite_pixel) as u16)
        } else {
            if sprite_priority {
                self.read_ppu_memory(0x3F00 | (sprite_palette << 2 | sprite_pixel) as u16)
            } else {
                self.read_ppu_memory(0x3F00 | (bg_palette << 2 | bg_pixel) as u16)
            }
        };

        let (r, g, b) = convert_color(pixel_color);
        let offset = (y * 256 + x) * 3;
        if offset + 2 < self.framebuffer.len() {
            self.framebuffer[offset] = r;
            self.framebuffer[offset + 1] = g;
            self.framebuffer[offset + 2] = b;
        }
    }

    fn update_shifters(&mut self) {
        if (self.mask & 0x08) != 0 {
            self.bg_shifter_pattern_lo <<= 1;
            self.bg_shifter_pattern_hi <<= 1;
            self.bg_shifter_attrib_lo <<= 1;
            self.bg_shifter_attrib_hi <<= 1;
        }

        if (self.mask & 0x10) != 0 {
            for i in 0..self.sprite_count {
                self.sprite_patterns[i] <<= 1;
            }
        }
    }

    // outros métodos específicos de rendering
}