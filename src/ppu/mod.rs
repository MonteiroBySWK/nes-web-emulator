mod frame_timing;
mod nmi;
mod clock;
mod rendering;
mod sprite;
mod scrolling;
mod memory;
mod registers;
mod debug;
mod colors;

use crate::rom::Mirroring;
use self::colors::convert_color;

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
    pub(crate) odd_frame: bool,  
    pub(crate) rendering_enabled: bool,
}

pub struct StepResult {
    pub new_frame: bool,
    pub vblank_nmi: bool,
}

impl PPU {
    pub fn new() -> Self {
        PPU {
            // Initialize registers to power-up state
            ctrl: 0,
            mask: 0,
            status: 0xA0, // Bit 7,5 set on powerup
            oam_addr: 0,
            oam_data: [0xFF; 256], // OAM initialized to $FF
            scroll: 0,
            addr: 0,
            data: 0,
            
            // Internal registers
            v: 0,
            t: 0,
            x: 0,
            w: false,
            read_buffer: 0,

            // Memory
            vram: [0; 0x4000],
            palette: [0; 32],
            oam: [0xFF; 256],
            secondary_oam: [0; 32],

            // Rendering state
            cycle: 0,
            scanline: 0,
            frame: 0,
            bg_shifter_pattern_lo: 0,
            bg_shifter_pattern_hi: 0,
            bg_shifter_attrib_lo: 0,
            bg_shifter_attrib_hi: 0,
            bg_next_tile_id: 0,
            bg_next_tile_attrib: 0,
            bg_next_tile_lsb: 0,
            bg_next_tile_msb: 0,
            sprite_count: 0,
            sprite_patterns: [0; 8],
            sprite_positions: [0; 8],
            sprite_priorities: [0; 8],
            sprite_indexes: [0; 8],
            sprite_zero_hit_possible: false,
            sprite_zero_being_rendered: false,
            framebuffer: [0; 256 * 240 * 3],
            nmi_occurred: false,
            nmi_output: false,
            nmi_previous: false,
            nmi_delay: 0,
            mirroring: Mirroring::Horizontal,
            odd_frame: false,
            rendering_enabled: false,
        }
    }

    // Métodos básicos/globais que não se encaixam em categorias específicas
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

    pub(crate) fn is_rendering_enabled(&self) -> bool {
        (self.mask & 0x18) != 0
    }
}

// Export main struct
pub use self::rendering::*;