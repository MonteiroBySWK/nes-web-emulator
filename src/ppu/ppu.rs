use crate::rom::Mirroring;

pub struct PPU {
    // MMIO registers (memory mapped I/O)
    pub ctrl: u8,    // $2000 PPUCTRL
    pub mask: u8,    // $2001 PPUMASK
    status: u8,      // $2002 PPUSTATUS
    oam_addr: u8,    // $2003 OAMADDR
    pub oam_data: [u8; 256], // $2004 OAMDATA array
    scroll: u8,      // $2005 PPUSCROLL
    addr: u16,       // $2006 PPUADDR
    data: u8,        // $2007 PPUDATA
    
    // Internal PPU registers
    v: u16,  // Current VRAM address (15 bits)
    t: u16,  // Temporary VRAM address (15 bits)
    x: u8,   // Fine X scroll (3 bits)
    w: bool, // Write toggle (1 bit)
    
    // Internal data buffer for reads
    read_buffer: u8,

    // Memory
    vram: [u8; 0x4000],       // VRAM (16KB)
    palette: [u8; 32],        // Palette RAM (32 bytes)
    oam: [u8; 256],          // Primary OAM (256 bytes)
    secondary_oam: [u8; 32],  // Secondary OAM (32 bytes)

    // Rendering state
    cycle: u16,      // 0-340
    scanline: u16,   // 0-261
    frame: u64,      // Frame counter
    
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
    sprite_count: usize,
    sprite_patterns: [u16; 8],
    sprite_positions: [u8; 8],
    sprite_priorities: [u8; 8],
    sprite_indexes: [u8; 8],
    sprite_zero_hit_possible: bool,
    sprite_zero_being_rendered: bool,

    // Output
    framebuffer: [u8; 256 * 240 * 3],
    
    // NMI flags
    nmi_occurred: bool,
    nmi_output: bool,
    nmi_previous: bool,
    nmi_delay: u8,

    // Additional state
    mirroring: Mirroring,
    odd_frame: bool,  // Novo campo para controlar frames par/ímpar
    rendering_enabled: bool, // Novo campo para verificar se rendering está ativo
}

// NOVO: Struct para resultado do avanço, inspirado em ppu2.
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

    pub fn get_all_registers(&self) -> (u8, u8, u8, u8, u8, u8, u16) {
        (
            self.ctrl, // PPUCTRL ($2000)
            self.mask, // PPUMASK ($2001)
            self.status, // PPUSTATUS ($2002)
            self.oam_addr, // OAMADDR ($2003)
            self.scroll, // PPUSCROLL ($2005)
            self.addr as u8, // PPUADDR ($2006) - lower byte
            self.addr, // PPUADDR ($2006) - full 16-bit address
        )
    }
    /// Novo método para carregar os dados do CHR-ROM na VRAM
    pub fn load_chr_data(&mut self, data: &[u8]) {
        let len = data.len().min(self.vram.len());
        self.vram[..len].copy_from_slice(&data[..len]);
    }

    // Mapeamento de registradores
    pub fn read_register(&mut self, address: u16) -> u8 {
        match address {
            0x2000 => 0, // PPUCTRL - Somente escrita
            0x2001 => 0, // PPUMASK - Somente escrita
            0x2002 => {
                // PPUSTATUS
                let result = self.status;
                // Limpa o bit 7 (vblank) e o latch de escrita
                self.status &= 0x7f;
                self.w = false;
                result
            }
            0x2003 => 0, // OAMADDR - Somente escrita
            0x2004 => self.oam_data[self.oam_addr as usize], // OAMDATA
            0x2005 => 0, // PPUSCROLL - Somente escrita
            0x2006 => 0, // PPUADDR - Somente escrita
            0x2007 => {
                // PPUDATA
                let mut data = self.read_buffer;

                // Atualiza o buffer com o novo valor
                self.read_buffer = self.read_ppu_memory(self.v);

                // Leitura imediata para paletas
                if self.v >= 0x3f00 {
                    data = self.read_ppu_memory(self.v);
                }

                // Incrementa o endereço após a leitura
                let increment = if (self.ctrl & 0x04) != 0 { 32 } else { 1 };
                self.v = self.v.wrapping_add(increment);

                data
            }
            _ => 0,
        }
    }

    pub fn write_register(&mut self, address: u16, value: u8) {
        web_sys::console::log_1(
            &format!("PPU Write to address: 0x{:04x}, value: 0x{:02x}", address, value).into()
        );
        match address {
            0x2000 => {
                // PPUCTRL
                self.ctrl = value;

                // t: ...BA.. ........ = d: ......BA
                self.t = (self.t & 0xf3ff) | (((value as u16) & 0x03) << 10);

                // Atualiza o estado de NMI
                self.nmi_output = (value & 0x80) != 0;

                // Se NMI mudou para ativado e VBLANK está setado, gera NMI
                if self.nmi_output && self.nmi_occurred {
                    self.trigger_nmi();
                }
            }
            0x2001 => {
                let old_enabled = self.is_rendering_enabled();
                self.mask = value;
                let new_enabled = self.is_rendering_enabled();
                
                // Se rendering foi habilitado, atualiza estado
                if !old_enabled && new_enabled {
                    self.rendering_enabled = true;
                }
            }
            0x2002 => {} // PPUSTATUS - Somente leitura
            0x2003 => {
                // OAMADDR
                self.oam_addr = value;
            }
            0x2004 => {
                // OAMDATA
                self.oam_data[self.oam_addr as usize] = value;
                self.oam_addr = self.oam_addr.wrapping_add(1);
            }
            0x2005 => {
                // PPUSCROLL
                if !self.w {
                    // Primeira escrita (X scroll)
                    // t: ....... ...HGFED = d: HGFED...
                    self.t = (self.t & 0xffe0) | ((value as u16) >> 3);
                    // x = d: .....CBA
                    self.x = value & 0x07;
                    self.w = true;
                } else {
                    // Segunda escrita (Y scroll)
                    // t: CBA..HG FED..... = d: HGFEDCBA
                    self.t = (self.t & 0x8fff) | (((value as u16) & 0x07) << 12);
                    self.t = (self.t & 0xfc1f) | (((value as u16) & 0xf8) << 2);
                    self.w = false;
                }
            }
            0x2006 => {
                // PPUADDR
                if !self.w {
                    // Primeira escrita (high byte)
                    // t: .FEDCBA ........ = d: ..FEDCBA
                    // t: X...... ........ = 0
                    self.t = (self.t & 0x00ff) | (((value as u16) & 0x3f) << 8);
                    self.w = true;
                } else {
                    // Segunda escrita (low byte)
                    // t: ....... HGFEDCBA = d: HGFEDCBA
                    self.t = (self.t & 0xff00) | (value as u16);
                    // v = t
                    self.v = self.t;
                    self.w = false;
                }
            }
            0x2007 => {
                // PPUDATA
                self.write_ppu_memory(self.v, value);

                // Incrementa o endereço após a escrita
                let increment = if (self.ctrl & 0x04) != 0 { 32 } else { 1 };
                self.v = self.v.wrapping_add(increment);
            }
            _ => {}
        }
    }

    // Leitura/escrita no espaço de endereçamento da PPU
    fn read_ppu_memory(&self, addr: u16) -> u8 {
        let addr = addr & 0x3fff; // Máscara para garantir endereço válido

        match addr {
            0x0000..=0x1fff => {
                // Pattern Tables (CHR ROM/RAM)
                // Normalmente mapeado através do cartridge, simplificado aqui
                self.vram[addr as usize]
            }
            0x2000..=0x3eff => {
                // Nametables com mirroring
                let mirrored_addr = self.mirror_nametable_address(addr);
                self.vram[mirrored_addr as usize]
            }
            0x3f00..=0x3fff => {
                // Palettes
                let palette_addr = (addr & 0x1f) as usize;

                // Mirrors de $3F10/$3F14/$3F18/$3F1C para $3F00/$3F04/$3F08/$3F0C
                let palette_addr = match palette_addr {
                    0x10 => 0x00,
                    0x14 => 0x04,
                    0x18 => 0x08,
                    0x1c => 0x0c,
                    _ => palette_addr,
                };

                self.palette[palette_addr]
            }
            _ => 0,
        }
    }

    fn write_ppu_memory(&mut self, addr: u16, data: u8) {
        let addr = addr & 0x3fff; // Máscara para garantir endereço válido

        match addr {
            0x0000..=0x1fff => {
                // Pattern Tables (CHR ROM/RAM)
                // Se for CHR RAM, podemos escrever aqui
                self.vram[addr as usize] = data;
            }
            0x2000..=0x3eff => {
                // Nametables com mirroring
                let mirrored_addr = self.mirror_nametable_address(addr);
                self.vram[mirrored_addr as usize] = data;
            }
            0x3f00..=0x3fff => {
                // Palettes
                let palette_addr = (addr & 0x1f) as usize;

                // Mirrors de $3F10/$3F14/$3F18/$3F1C para $3F00/$3F04/$3F08/$3F0C
                let palette_addr = match palette_addr {
                    0x10 => 0x00,
                    0x14 => 0x04,
                    0x18 => 0x08,
                    0x1c => 0x0c,
                    _ => palette_addr,
                };

                self.palette[palette_addr] = data;
            }
            _ => {}
        }
    }

    // Gera uma interrupção NMI
    pub fn trigger_nmi(&mut self) -> bool {
        self.nmi_output && self.nmi_occurred
    }

    // Funções auxiliares para renderização
    fn increment_scroll_x(&mut self) {
        // Incrementa o X scroll, com wrap ao redor do nametable
        if (self.mask & 0x08) != 0 || (self.mask & 0x10) != 0 {
            if (self.v & 0x001f) == 31 {
                self.v &= !0x001f; // Coarse X = 0
                self.v ^= 0x0400; // Troca bit de seleção horizontal de nametable
            } else {
                self.v += 1; // Incrementa coarse X
            }
        }
    }

    fn increment_scroll_y(&mut self) {
        if (self.mask & 0x08) != 0 || (self.mask & 0x10) != 0 {
            let mut y = (self.v & 0x7000) >> 12;
            if y == 7 {
                y = 0;
                let coarse_y = (self.v & 0x03e0) >> 5;

                self.v = if coarse_y == 29 {
                    (self.v & !0x03e0) ^ 0x0800 // Switch vertical nametable and reset coarse Y
                } else if coarse_y == 31 {
                    self.v & !0x03e0 // Just reset coarse Y
                } else {
                    (self.v & !0x03e0) | (((coarse_y + 1) & 0x1F) << 5) // Increment coarse Y
                };

                self.v = (self.v & !0x7000) | (y << 12);
            } else {
                self.v = (self.v & !0x7000) | ((y + 1) << 12);
            }
        }
    }

    fn transfer_address_x(&mut self) {
        // Transfere componente X do registro t para v
        if (self.mask & 0x08) != 0 || (self.mask & 0x10) != 0 {
            self.v = (self.v & !0x041f) | (self.t & 0x041f);
        }
    }

    fn transfer_address_y(&mut self) {
        // Transfere componente Y do registro t para v
        if (self.mask & 0x08) != 0 || (self.mask & 0x10) != 0 {
            self.v = (self.v & !0x7be0) | (self.t & 0x7be0);
        }
    }

    fn load_background_shifters(&mut self) {
        self.bg_shifter_pattern_lo =
            (self.bg_shifter_pattern_lo & 0xff00) | (self.bg_next_tile_lsb as u16);
        self.bg_shifter_pattern_hi =
            (self.bg_shifter_pattern_hi & 0xff00) | (self.bg_next_tile_msb as u16);

        self.bg_shifter_attrib_lo =
            (self.bg_shifter_attrib_lo & 0xff00) |
            (if (self.bg_next_tile_attrib & 0x01) != 0 { 0xff } else { 0x00 });
        self.bg_shifter_attrib_hi =
            (self.bg_shifter_attrib_hi & 0xff00) |
            (if (self.bg_next_tile_attrib & 0x02) != 0 { 0xff } else { 0x00 });
    }

    fn update_shifters(&mut self) {
        if (self.mask & 0x08) != 0 {
            // Background shifters
            self.bg_shifter_pattern_lo <<= 1;
            self.bg_shifter_pattern_hi <<= 1;
            self.bg_shifter_attrib_lo <<= 1;
            self.bg_shifter_attrib_hi <<= 1;

            // Carregar novos bits se estivermos em um ciclo de tile fetch
            if self.cycle >= 1 && self.cycle <= 256 && self.cycle % 8 == 0 {
                self.load_background_shifters();
            }
        }

        // Atualiza os shifters dos sprites.
        // Aqui, para cada sprite visível no scanline, deslocamos o padrão para a esquerda.
        for i in 0..self.sprite_count {
            self.sprite_patterns[i] <<= 1;
        }
    }

    // NOVO: Renderiza toda a scanline visível (256 pixels).
    fn render_scanline(&mut self) {
        // Renderiza cada pixel da scanline (x de 0 a 255)
        for pixel in 0..256 {
            self.cycle = (pixel + 1) as u16;
            self.render_pixel();
        }
    }

    /// Método de avanço melhorado inspirado em ppu2, com timing revisado.
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

    fn fetch_nametable_byte(&mut self) {
        let addr = 0x2000 | (self.v & 0x0FFF);
        self.bg_next_tile_id = self.read_ppu_memory(addr);
    }

    fn fetch_attribute_byte(&mut self) {
        let addr = 0x23C0 | (self.v & 0x0C00) | ((self.v >> 4) & 0x38) | ((self.v >> 2) & 0x07);
        let shift = ((self.v >> 4) & 4) | (self.v & 2);
        self.bg_next_tile_attrib = ((self.read_ppu_memory(addr) >> shift) & 3) * 0x55;
    }

    fn fetch_pattern_low(&mut self) {
        let fine_y = (self.v >> 12) & 7;
        let table = ((self.ctrl & 0x10) >> 4) as u16;
        let addr = (table << 12) | ((self.bg_next_tile_id as u16) << 4) | fine_y;
        self.bg_next_tile_lsb = self.read_ppu_memory(addr);
    }

    fn fetch_pattern_high(&mut self) {
        let fine_y = (self.v >> 12) & 7;
        let table = ((self.ctrl & 0x10) >> 4) as u16;
        let addr = (table << 12) | ((self.bg_next_tile_id as u16) << 4) | fine_y | 8;
        self.bg_next_tile_msb = self.read_ppu_memory(addr);
    }

    // Preparação de sprites para o próximo scanline
    fn load_sprites_for_next_scanline(&mut self) {
        // Reset da OAM secundária
        for i in 0..32 {
            self.secondary_oam[i] = 0xff;
        }

        // Contadores para o scanline atual
        self.sprite_count = 0;
        let mut n = 0;
        self.sprite_zero_hit_possible = false;

        // Altura dos sprites (8 ou 16 pixels)
        let sprite_height = if (self.ctrl & 0x20) != 0 { 16 } else { 8 };

        // Varredura da OAM para encontrar sprites visíveis no próximo scanline
        for i in 0..64 {
            let sprite_y = self.oam_data[i * 4 + 0] as i16;
            let sprite_id = self.oam_data[i * 4 + 1];
            let sprite_attr = self.oam_data[i * 4 + 2];
            let sprite_x = self.oam_data[i * 4 + 3];

            let diff = (self.scanline as i16) - sprite_y;

            // Verifica se o sprite está no próximo scanline
            if diff >= 0 && diff < sprite_height {
                // Verificamos se temos espaço na OAM secundária
                if self.sprite_count < 8 {
                    // Sprite-0 hit check
                    if i == 0 {
                        self.sprite_zero_hit_possible = true;
                    }

                    // Copia para a OAM secundária
                    self.secondary_oam[n + 0] = sprite_y as u8;
                    self.secondary_oam[n + 1] = sprite_id;
                    self.secondary_oam[n + 2] = sprite_attr;
                    self.secondary_oam[n + 3] = sprite_x;

                    n += 4;
                    self.sprite_count += 1;
                } else {
                    // Overflow de sprites
                    self.status |= 0x20;
                    break;
                }
            }
        }

        // Pré-cálculo dos padrões de sprites para uso durante o scanline
        for i in 0..self.sprite_count {
            let sprite_id = self.secondary_oam[i * 4 + 1];
            let sprite_attr = self.secondary_oam[i * 4 + 2];

            // Seleção de pattern table para sprites
            let pattern_addr_lo: u16;
            let pattern_addr_hi: u16;

            // Lógica para sprites 8x8 ou 8x16
            if (self.ctrl & 0x20) == 0 {
                // 8x8 sprites
                let pattern_table = ((self.ctrl & 0x08) >> 3) as u16;
                pattern_addr_lo = pattern_table * 0x1000 + (sprite_id as u16) * 16;
                pattern_addr_hi = pattern_addr_lo + 8;
            } else {
                // 8x16 sprites
                pattern_addr_lo =
                    ((sprite_id & 0x01) as u16) * 0x1000 + ((sprite_id & 0xfe) as u16) * 16;
                pattern_addr_hi = pattern_addr_lo + 8;
            }

            // Aqui precisaríamos calcular o padrão exato com base no Y do sprite
            // Simplificado para este exemplo
            let sprite_pattern_lo = self.read_ppu_memory(pattern_addr_lo);
            let sprite_pattern_hi = self.read_ppu_memory(pattern_addr_hi);

            // Armazenar dados para uso durante o scanline
            self.sprite_patterns[i] =
                ((sprite_pattern_hi as u16) << 8) | (sprite_pattern_lo as u16);
            self.sprite_positions[i] = self.secondary_oam[i * 4 + 3];
            self.sprite_priorities[i] = (sprite_attr & 0x20) >> 5;
            self.sprite_indexes[i] = i as u8;
        }
    }

    // Função para renderizar um pixel na posição atual (ciclo e scanline)
    fn render_pixel(&mut self) {
        let x = (self.cycle - 1) as usize;
        let y = self.scanline as usize;
        
        // Skip if outside visible area
        if x >= 256 || y >= 240 {
            return;
        }

        let mut pixel = 0u8;
        let mut palette = 0u8;

        // Background rendering
        if (self.mask & 0x08) != 0 && (x >= 8 || (self.mask & 0x02) != 0) {
            let bit_mux = 0x8000 >> self.x;
            
            let p0 = if (self.bg_shifter_pattern_lo & bit_mux) != 0 { 1 } else { 0 };
            let p1 = if (self.bg_shifter_pattern_hi & bit_mux) != 0 { 2 } else { 0 };
            pixel = p0 | p1;

            let pal0 = if (self.bg_shifter_attrib_lo & bit_mux) != 0 { 1 } else { 0 };
            let pal1 = if (self.bg_shifter_attrib_hi & bit_mux) != 0 { 2 } else { 0 };
            palette = pal0 | pal1;
        }

        // Sprite rendering
        if (self.mask & 0x10) != 0 && (x >= 8 || (self.mask & 0x04) != 0) {
            for i in 0..self.sprite_count {
                let sprite_x = self.sprite_positions[i];
                let offset = x as i16 - sprite_x as i16;
                
                if offset >= 0 && offset < 8 {
                    let pattern = self.sprite_patterns[i];
                    let sprite_pixel = ((pattern >> (7 - offset)) & 0x01) |
                                     (((pattern >> 8) >> (7 - offset)) & 0x01) << 1;

                    if sprite_pixel != 0 {
                        let sprite_palette = 4 + ((self.secondary_oam[i * 4 + 2] & 0x03) as u8);
                        let priority = (self.secondary_oam[i * 4 + 2] & 0x20) == 0;

                        // Sprite 0 hit detection
                        if i == 0 && pixel != 0 && x < 255 {
                            self.status |= 0x40;
                        }

                        // Render sprite pixel if it has priority or background is transparent
                        if priority || pixel == 0 {
                            pixel = sprite_pixel as u8;
                            palette = sprite_palette;
                        }
                    }
                }
            }
        }

        // Get color from palette
        let palette_addr = if pixel == 0 { 0 } else { (palette << 2) | pixel };
        let color_idx = self.read_ppu_memory(0x3F00 | palette_addr as u16) & 0x3F;
        let (r, g, b) = PPU::convert_color(color_idx);

        // Write to framebuffer
        let pixel_pos = (y * 256 + x) * 3;
        if pixel_pos + 2 < self.framebuffer.len() {
            self.framebuffer[pixel_pos] = r;
            self.framebuffer[pixel_pos + 1] = g;
            self.framebuffer[pixel_pos + 2] = b;
        }
    }

    // New color conversion function with full 64-color palette
    fn convert_color(value: u8) -> (u8, u8, u8) {
        const NES_PALETTE: [(u8, u8, u8); 64] = [
            (84,84,84),    (0,30,116),    (8,16,144),    (48,0,136),
            (68,0,100),    (92,0,48),     (84,4,0),      (60,24,0),
            (32,42,0),     (8,58,0),      (0,64,0),      (0,60,0),
            (0,50,60),     (0,0,0),       (0,0,0),       (0,0,0),
            
            (152,150,152), (8,76,196),    (48,50,236),   (92,30,228),
            (136,20,176),  (160,20,100),  (152,34,32),   (120,60,0),
            (84,90,0),     (40,114,0),    (8,124,0),     (0,118,40),
            (0,102,120),   (0,0,0),       (0,0,0),       (0,0,0),
            
            (236,238,236), (76,154,236),  (120,124,236), (176,98,236),
            (228,84,236),  (236,88,180),  (236,106,100), (212,136,32),
            (160,170,0),   (116,196,0),   (76,208,32),   (56,204,108),
            (56,180,204),  (60,60,60),    (0,0,0),       (0,0,0),
            
            (236,238,236), (168,204,236), (188,188,236), (212,178,236),
            (236,174,236), (236,174,212), (236,180,176), (228,196,144),
            (204,210,120), (180,222,120), (168,226,144), (152,226,180),
            (160,214,228), (160,162,160), (0,0,0),       (0,0,0),
        ];

        let index = (value & 0x3F) as usize;
        NES_PALETTE[index]
    }

    // Métodos públicos para debug e interface

    // Obtém o framebuffer atual
    pub fn get_framebuffer(&self) -> &[u8] {
        &self.framebuffer
    }

    // Define o tipo de mirroring (pode ser chamado pelo cartridge/mapper)
    pub fn set_mirroring(&mut self, mirroring_type: Mirroring) {
        self.mirroring = mirroring_type;
    }

    // Métodos para depuração
    pub fn debug_render_pattern_table(
        &self,
        pattern_table_index: usize,
        palette: u8
    ) -> [u8; 128 * 128 * 3] {
        let mut buffer = [0u8; 128 * 128 * 3];

        for tile_y in 0..16 {
            for tile_x in 0..16 {
                let tile_index = tile_y * 16 + tile_x;
                let tile_addr = (pattern_table_index * 0x1000) as u16 + (tile_index as u16) * 16;

                for row in 0..8 {
                    let plane0 = self.read_ppu_memory(tile_addr + row);
                    let plane1 = self.read_ppu_memory(tile_addr + row + 8);

                    for col in 0..8 {
                        let bit0 = (plane0 >> (7 - col)) & 1;
                        let bit1 = (plane1 >> (7 - col)) & 1;
                        let pixel = (bit1 << 1) | bit0;

                        let palette_addr = 0x3f00 | ((palette as u16) << 2) | (pixel as u16);
                        let color_index = self.read_ppu_memory(palette_addr & 0x3FFF);

                        let (r, g, b) = PPU::convert_color(color_index);

                        let x = tile_x * 8 + col;
                        let y = tile_y * 8 + row;
                        let pos = ((y * 128 + x) * 3) as usize;

                        if pos + 2 < buffer.len() {
                            buffer[pos] = r;
                            buffer[pos + 1] = g;
                            buffer[pos + 2] = b;
                        }
                    }
                }
            }
        }

        buffer
    }

    // Debugger para visualizar nametables
    pub fn debug_render_nametables(&self) -> [u8; 512 * 480 * 3] {
        let mut buffer = [0u8; 512 * 480 * 3];

        // Para cada nametable (2x2)
        for nt_y in 0..2 {
            for nt_x in 0..2 {
                let nt_addr = 0x2000 + nt_y * 0x800 + nt_x * 0x400;

                // Para cada tile no nametable (32x30)
                for tile_y in 0..30 {
                    for tile_x in 0..32 {
                        // Lê o índice do tile
                        let addr = nt_addr + tile_y * 32 + tile_x;
                        let tile_index = self.read_ppu_memory(addr as u16);

                        // Lê o atributo para determinar a paleta
                        let attr_x = tile_x / 4;
                        let attr_y = tile_y / 4;
                        let attr_addr = (nt_addr + 0x3C0 + attr_y * 8 + attr_x) as u16;
                        let attr_byte = self.read_ppu_memory(attr_addr);
                        
                        // Determina qual parte do byte de atributo usar
                        let attr_shift = ((tile_y % 4) / 2) * 4 + ((tile_x % 4) / 2) * 2;
                        let palette_index = (attr_byte >> attr_shift) & 0x03;

                        // Renderiza o tile 8x8
                        let pattern_table = (self.ctrl & 0x10) != 0;
                        let tile_addr = (if pattern_table { 0x1000 } else { 0 }) + tile_index as u16 * 16;

                        for row in 0..8 {
                            let plane0 = self.read_ppu_memory(tile_addr + row);
                            let plane1 = self.read_ppu_memory(tile_addr + row + 8);

                            for col in 0..8 {
                                let bit0 = (plane0 >> (7 - col)) & 1;
                                let bit1 = (plane1 >> (7 - col)) & 1;
                                let pixel = (bit1 << 1) | bit0;

                                let palette_addr = 0x3f00 + (palette_index * 4 + pixel) as u16;
                                let color_index = self.read_ppu_memory(palette_addr);
                                let (r, g, b) = PPU::convert_color(color_index);

                                let x = nt_x * 256 + tile_x * 8 + col;
                                let y = nt_y * 240 + tile_y * 8 + row;
                                let pos = (y * 512 + x) * 3;

                                if pos + 2 < buffer.len() as u16 {
                                    buffer[pos as usize] = r;
                                    buffer[(pos + 1) as usize] = g;
                                    buffer[(pos + 2) as usize] = b;
                                }
                            }
                        }
                    }
                }
            }
        }

        buffer
    }

    pub fn debug_pattern_tables(&self) {
        for addr in 0..0x2000 {
            if self.vram[addr] != 0 {
                web_sys::console::log_1(&format!(
                    "Pattern table data at {:04X}: {:02X}",
                    addr,
                    self.vram[addr]
                ).into());
            }
        }
    }

    pub fn debug_nametables(&self) {
        for nt in 0..4 {
            let base = 0x2000 + (nt * 0x400);
            let mut non_zero = false;
            for offset in 0..0x400 {
                if self.vram[base + offset] != 0 {
                    non_zero = true;
                    web_sys::console::log_1(&format!(
                        "NT{} data at {:04X}: {:02X}",
                        nt,
                        base + offset,
                        self.vram[base + offset]
                    ).into());
                }
            }
            if !non_zero {
                web_sys::console::log_1(&format!("Nametable {} is empty", nt).into());
            }
        }
    }

    fn is_rendering_enabled(&self) -> bool {
        (self.mask & 0x18) != 0 // Verifica se background ou sprites estão habilitados
    }
}

// Extensão da implementação para incluir o mirroring
impl PPU {
    // Implementação corrigida da função de mirror_nametable_address
    fn mirror_nametable_address(&self, addr: u16) -> u16 {
        let addr = addr & 0x2fff;
        let nametable_index = (addr - 0x2000) / 0x400;
        let offset = (addr - 0x2000) % 0x400;

        let mirrored_nametable = match self.mirroring {
            Mirroring::Horizontal => {
                // Nametables são espelhados horizontalmente: 0,1 => 0, 2,3 => 2
                match nametable_index {
                    0 => 0,
                    1 => 0,
                    2 => 2,
                    3 => 2,
                    _ => 0, // Não deve acontecer
                }
            }
            Mirroring::Vertical => {
                // Nametables são espelhados verticalmente: 0,2 => 0, 1,3 => 1
                match nametable_index {
                    0 => 0,
                    1 => 1,
                    2 => 0,
                    3 => 1,
                    _ => 0, // Não deve acontecer
                }
            }
            Mirroring::FourScreen => {
                // Sem espelhamento, cada nametable é independente
                nametable_index
            }
            Mirroring::OneScreenLo => {
                // Todos mapeiam para o primeiro nametable
                0
            }
            Mirroring::OneScreenHi => {
                // Todos mapeiam para o segundo nametable
                1
            }
        };

        0x2000 + mirrored_nametable * 0x400 + offset
    }
}
