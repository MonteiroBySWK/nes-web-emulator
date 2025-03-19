use crate::rom::Mirroring;

pub struct PPU {
    // Registradores mapeados
    pub ctrl: u8,
    pub mask: u8,
    status: u8,
    oam_addr: u8,
    pub oam_data: [u8; 256],
    scroll: u8,
    addr: u16,

    // VRAM interna da PPU (dados gráficos)
    vram: [u8; 0x4000],

    // Paletas de cores (32 bytes)
    palette: [u8; 32],

    // OAM secundária (para os 8 sprites do scanline atual)
    secondary_oam: [u8; 32],

    // Registradores internos auxiliares para scroll e endereço
    v: u16, // Endereço VRAM atual/ativo
    t: u16, // Endereço VRAM temporário
    x: u8, // Fine X scroll
    w: bool, // Latch de escrita

    // Buffering de leitura VRAM
    read_buffer: u8,

    // Contadores de ciclo e scanline
    pub cycle: u16,
    pub scanline: u16, // changed from i16 to u16 for consistency with ppu2 timing

    // Frame atual
    frame: u64,

    // Framebuffer para saída
    framebuffer: [u8; 256 * 240 * 3], // RGB, 3 bytes por pixel

    // Suporte a interrupções
    pub nmi_occurred: bool,
    nmi_output: bool,

    // Dados para renderização
    bg_next_tile_id: u8,
    bg_next_tile_attrib: u8,
    bg_next_tile_lsb: u8,
    bg_next_tile_msb: u8,
    bg_shifter_pattern_lo: u16,
    bg_shifter_pattern_hi: u16,
    bg_shifter_attrib_lo: u16,
    bg_shifter_attrib_hi: u16,

    // Sprite zero hit possível nesse scanline
    sprite_zero_hit_possible: bool,
    sprite_zero_being_rendered: bool,

    // Contagem de sprites no scanline atual
    sprite_count: usize,

    // Dados dos sprites para o scanline atual
    sprite_patterns: [u16; 8],
    sprite_positions: [u8; 8],
    sprite_priorities: [u8; 8],
    sprite_indexes: [u8; 8],

    mirroring: Mirroring,
}

// NOVO: Struct para resultado do avanço, inspirado em ppu2.
pub struct StepResult {
    pub new_frame: bool,
    pub vblank_nmi: bool,
}

impl PPU {
    pub fn new() -> Self {
        let mut ppu = PPU {
            ctrl: 0,
            // Para testes, habilite renderização de background e sprites:
            mask: 0x18, // Habilita background (0x08) e sprites (0x10)
            status: 0,
            oam_addr: 0,
            oam_data: [0; 256],
            scroll: 0,
            addr: 0,
            vram: [0; 0x4000],
            // Inicializa a paleta com valores padrão (exemplo simplificado)
            palette: [
                0x0f, 0x30, 0x21, 0x31, 0x0f, 0x30, 0x21, 0x31, 0x0f, 0x30, 0x21, 0x31, 0x0f, 0x30,
                0x21, 0x31, 0x0f, 0x30, 0x21, 0x31, 0x0f, 0x30, 0x21, 0x31, 0x0f, 0x30, 0x21, 0x31,
                0x0f, 0x30, 0x21, 0x31,
            ],
            secondary_oam: [0; 32],
            v: 0,
            t: 0,
            x: 0,
            w: false,
            read_buffer: 0,
            cycle: 0,
            scanline: 0,
            frame: 0,
            framebuffer: [0; 256 * 240 * 3],
            nmi_occurred: false,
            nmi_output: false,
            bg_next_tile_id: 0,
            bg_next_tile_attrib: 0,
            bg_next_tile_lsb: 0,
            bg_next_tile_msb: 0,
            bg_shifter_pattern_lo: 0,
            bg_shifter_pattern_hi: 0,
            bg_shifter_attrib_lo: 0,
            bg_shifter_attrib_hi: 0,
            sprite_zero_hit_possible: false,
            sprite_zero_being_rendered: false,
            sprite_count: 0,
            sprite_patterns: [0; 8],
            sprite_positions: [0; 8],
            sprite_priorities: [0; 8],
            sprite_indexes: [0; 8],
            mirroring: Mirroring::Horizontal,
        };

        // Se os dados de CHR-ROM estiverem disponíveis, use o método load_chr_data.
        // Exemplo: ppu.load_chr_data(&chr_data);

        ppu
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
        web_sys::console::log_1(&format!("PPU Write to address: 0x{:04x}, value: 0x{:02x}", address, value).into());
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
                // PPUMASK
                self.mask = value;
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
        // Incrementa o Y scroll, com wrap ao redor do nametable
        if (self.mask & 0x08) != 0 || (self.mask & 0x10) != 0 {
            let mut y = (self.v & 0x7000) >> 12; // Fine Y
            if y == 7 {
                y = 0;
                let mut coarse_y = (self.v & 0x03e0) >> 5;

                if coarse_y == 29 {
                    coarse_y = 0;
                    self.v ^= 0x0800; // Troca bit de seleção vertical de nametable
                } else if coarse_y == 31 {
                    coarse_y = 0;
                } else {
                    self.v = (self.v & !0x03e0) | ((coarse_y + 1) << 5);
                }

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
            self.bg_shifter_pattern_lo <<= 1;
            self.bg_shifter_pattern_hi <<= 1;
            self.bg_shifter_attrib_lo <<= 1;
            self.bg_shifter_attrib_hi <<= 1;
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

        if self.scanline < 240 {
            // Renderiza scanline visível
            self.render_scanline();
        }

        self.scanline += 1;
        self.cycle = 0;

        if self.scanline == 241 {
            self.status |= 0x80; // seta a flag de VBlank
            self.nmi_occurred = true;
            if self.nmi_output {
                result.vblank_nmi = true;
            }
        } else if self.scanline == 261 {
            // Final do frame; reinicia scanline e atualiza frame
            self.scanline = 0;
            self.frame += 1;
            self.status &= 0x1F; // limpa flags de VBlank e Sprite
            result.new_frame = true;
        }

        result
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

            let diff = self.scanline as i16 - sprite_y;

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
        let x = self.cycle - 1;
        let y = self.scanline as usize;

        let mut bg_pixel = 0u8;
        let mut bg_palette = 0u8;

        let mut fg_pixel = 0u8;
        let mut fg_palette = 0u8;
        let mut fg_priority = 0u8;
        let mut fg_is_sprite_zero = false;

        // ---------- Background ----------
        if (self.mask & 0x08) != 0 {
            // Calcula o "mux" com base na posição fina (self.x)
            let mux = 0x8000 >> self.x;

            let p0 = (self.bg_shifter_pattern_lo & mux) != 0;
            let p1 = (self.bg_shifter_pattern_hi & mux) != 0;
            bg_pixel = ((p1 as u8) << 1) | (p0 as u8);

            let a0 = (self.bg_shifter_attrib_lo & mux) != 0;
            let a1 = (self.bg_shifter_attrib_hi & mux) != 0;
            bg_palette = ((a1 as u8) << 1) | (a0 as u8);
        }

        // ---------- Sprites ----------
        if (self.mask & 0x10) != 0 {
            for i in 0..self.sprite_count {
                let sprite_x = self.sprite_positions[i] as u16;

                if sprite_x <= x && x < sprite_x + 8 {
                    let pattern = self.sprite_patterns[i];
                    let offset = x - sprite_x;
                    let p0 = (pattern >> offset) & 0x01;
                    // Para sprites, se houver suporte para flip vertical/horizontal, ajuste a lógica aqui.
                    let p1 = (pattern >> (offset + 8)) & 0x01; 
                    let sprite_pixel = (p1 << 1) | p0;

                    // Se o pixel não é transparente (não zero)
                    if sprite_pixel != 0 {
                        fg_pixel = sprite_pixel as u8;
                        // A paleta para sprites começa na posição 4 na paleta geral
                        fg_palette = 4 + (self.secondary_oam[i * 4 + 2] & 0x03);
                        fg_priority = self.sprite_priorities[i];
                        if i == 0 {
                            fg_is_sprite_zero = true;
                        }
                        break;
                    }
                }
            }
        }

        // ---------- Composição final ----------
        let mut final_pixel = 0u8;
        let mut final_palette = 0u8;

        if bg_pixel == 0 && fg_pixel == 0 {
            // Cor de fundo (normalmente definida em 0x3F00)
            final_pixel = 0;
            final_palette = 0;
        } else if bg_pixel == 0 && fg_pixel > 0 {
            final_pixel = fg_pixel;
            final_palette = fg_palette;
        } else if bg_pixel > 0 && fg_pixel == 0 {
            final_pixel = bg_pixel;
            final_palette = bg_palette;
        } else {
            // Ambos presentes; resolver prioridade
            if fg_priority == 0 {
                final_pixel = fg_pixel;
                final_palette = fg_palette;
            } else {
                final_pixel = bg_pixel;
                final_palette = bg_palette;
            }

            // Detecta Sprite Zero Hit, se aplicável
            if fg_is_sprite_zero && bg_pixel != 0 && fg_pixel != 0 && x < 255 {
                self.status |= 0x40;
            }
        }

        // Converter para RGB usando a paleta (endereço da paleta = 0x3F00 + (paleta*4 + pixel))
        let palette_addr = 0x3f00 + ((final_palette as u16) * 4 + (final_pixel as u16));
        let palette_value = self.read_ppu_memory(palette_addr);
        let (r, g, b) = self.convert_palette_to_rgb(palette_value);

        // Armazena o pixel no framebuffer (3 bytes RGB)
        let pixel_pos = (y * 256 + (x as usize)) * 3;
        if pixel_pos + 2 < self.framebuffer.len() {
            self.framebuffer[pixel_pos] = r;
            self.framebuffer[pixel_pos + 1] = g;
            self.framebuffer[pixel_pos + 2] = b;
        }
    }

    // Converte valor da paleta para RGB
    fn convert_palette_to_rgb(&self, palette_value: u8) -> (u8, u8, u8) {
        // Tabela de cores 2C02 (simplificada)
        // Na prática, uma tabela completa de 64 entradas seria usada
        match palette_value & 0x3f {
            0x00 => (84, 84, 84), // Cinza escuro
            0x01 => (0, 30, 116), // Azul escuro
            0x02 => (8, 16, 144), // Roxo escuro
            0x03 => (48, 0, 136), // Roxo
            0x04 => (68, 0, 100), // Roxo avermelhado
            0x05 => (92, 0, 48), // Vermelho escuro
            0x06 => (84, 4, 0), // Marrom escuro
            0x07 => (60, 24, 0), // Marrom
            0x08 => (32, 42, 0), // Verde escuro
            0x09 => (8, 58, 0), // Verde
            0x0a => (0, 64, 0), // Verde oliva
            0x0b => (0, 60, 0), // Verde claro
            0x0c => (0, 50, 60), // Ciano escuro
            0x0d => (0, 0, 0), // Preto
            0x0e => (0, 0, 0), // Preto
            0x0f => (0, 0, 0), // Preto

            0x10 => (152, 150, 152), // Cinza claro
            0x11 => (8, 76, 196), // Azul
            0x12 => (48, 50, 236), // Azul claro
            0x13 => (92, 30, 228), // Roxo claro
            0x14 => (136, 20, 176), // Rosa
            0x15 => (160, 20, 100), // Vermelho
            0x16 => (152, 34, 32), // Vermelho claro
            0x17 => (120, 60, 0), // Laranja
            0x18 => (84, 90, 0), // Amarelo escuro
            0x19 => (40, 114, 0), // Verde limão
            0x1a => (8, 124, 0), // Verde claro
            0x1b => (0, 118, 40), // Verde azulado
            0x1c => (0, 102, 120), // Ciano
            0x1d => (0, 0, 0), // Preto
            0x1e => (0, 0, 0), // Preto
            0x1f => (0, 0, 0), // Preto

            0x20 => (236, 238, 236), // Branco
            0x21 => (76, 154, 236), // Azul claro
            0x22 => (120, 124, 236), // Azul lavanda
            0x23 => (176, 98, 236), // Roxo claro
            0x24 => (228, 84, 236), // Rosa claro
            0x25 => (236, 88, 180), // Rosa salmão
            0x26 => (236, 106, 100), // Coral
            0x27 => (212, 136, 32), // Laranja
            0x28 => (160, 170, 0), // Amarelo
            0x29 => (116, 196, 0), // Verde limão
            0x2a => (76, 208, 32), // Verde claro
            0x2b => (56, 204, 108), // Verde esmeralda
            0x2c => (56, 180, 204), // Ciano claro
            0x2d => (60, 60, 60), // Cinza
            0x2e => (0, 0, 0), // Preto
            0x2f => (0, 0, 0), // Preto

            0x30 => (236, 238, 236), // Branco
            0x31 => (168, 204, 236), // Azul pastel
            0x32 => (188, 188, 236), // Lavanda pastel
            0x33 => (212, 178, 236), // Roxo pastel
            0x34 => (236, 174, 236), // Rosa pastel
            0x35 => (236, 174, 212), // Rosa claro pastel
            0x36 => (236, 180, 176), // Coral pastel
            0x37 => (228, 196, 144), // Laranja pastel
            0x38 => (204, 210, 120), // Amarelo pastel
            0x39 => (180, 222, 120), // Verde limão pastel
            0x3a => (168, 226, 144), // Verde pastel
            0x3b => (152, 226, 180), // Verde menta pastel
            0x3c => (160, 214, 228), // Ciano pastel
            0x3d => (160, 162, 160), // Cinza claro
            0x3e => (0, 0, 0), // Preto
            0x3f => (0, 0, 0), // Preto

            _ => (0, 0, 0), // Default (preto)
        }
    }

    // Métodos públicos para debug e interface

    // Obtém o framebuffer atual
    pub fn get_framebuffer(&self) -> &[u8] {
        // Debug the first few pixels
        web_sys::console::log_1(&format!("First 10 pixels: {:?}", &self.framebuffer[..30]).into());

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

        // Offset da pattern table
        let pattern_table_addr = (pattern_table_index * 0x1000) as u16;

        // Renderizar cada tile (16x16 tiles de 8x8 pixels)
        for tile_y in 0..16 {
            for tile_x in 0..16 {
                let tile_index = tile_y * 16 + tile_x;
                let tile_addr = pattern_table_addr + (tile_index as u16) * 16;

                // Renderizar um tile 8x8
                for row in 0..8 {
                    // Buscar os bits para esta linha do tile
                    let plane0 = self.read_ppu_memory(tile_addr + row);
                    let plane1 = self.read_ppu_memory(tile_addr + row + 8);

                    // Processar cada pixel na linha
                    for col in 0..8 {
                        let bit0 = (plane0 >> (7 - col)) & 1;
                        let bit1 = (plane1 >> (7 - col)) & 1;
                        let pixel = (bit1 << 1) | bit0;

                        // Calcular o endereço da paleta
                        let palette_addr = 0x3f00 + ((palette as u16) * 4 + (pixel as u16));
                        let color_index = self.read_ppu_memory(palette_addr);

                        // Converter para RGB
                        let (r, g, b) = self.convert_palette_to_rgb(color_index);

                        // Calcular posição no buffer de saída
                        let x = tile_x * 8 + col;
                        let y = tile_y * 8 + row;
                        let pos = (y * 128 + x) * 3;

                        // Armazenar o pixel no buffer
                        if pos + 2 < (buffer.len() as u16) {
                            buffer[pos as usize] = r;
                            buffer[(pos + 1) as usize] = g;
                            buffer[(pos + 2) as usize] = b;
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
                        let attr_addr = nt_addr + 0x3c0 + attr_y * 8 + attr_x;
                        let attr = self.read_ppu_memory(attr_addr as u16);

                        // Determina qual quadrante do atributo usar
                        let quad_x = (tile_x % 4) / 2;
                        let quad_y = (tile_y % 4) / 2;
                        let quad = quad_y * 2 + quad_x;
                        let palette_index = (attr >> (quad * 2)) & 0x03;

                        // Busca os dados do tile da pattern table
                        let pattern_table = (self.ctrl & 0x10) >> 4;
                        let tile_addr = (pattern_table as u16) * 0x1000 + (tile_index as u16) * 16;

                        // Renderiza o tile 8x8
                        for row in 0..8 {
                            let plane0 = self.read_ppu_memory(tile_addr + (row as u16));
                            let plane1 = self.read_ppu_memory(tile_addr + (row as u16) + 8);

                            for col in 0..8 {
                                let bit0 = (plane0 >> (7 - col)) & 1;
                                let bit1 = (plane1 >> (7 - col)) & 1;
                                let pixel = (bit1 << 1) | bit0;

                                // Seleciona a cor da paleta
                                let palette_addr =
                                    0x3f00 + ((palette_index as u16) * 4 + (pixel as u16));
                                let color_index = self.read_ppu_memory(palette_addr);

                                // Converte para RGB
                                let (r, g, b) = self.convert_palette_to_rgb(color_index);

                                // Calcula a posição no buffer
                                let x = nt_x * 256 + tile_x * 8 + col;
                                let y = nt_y * 240 + tile_y * 8 + row;
                                let pos = (y * 512 + x) * 3;

                                if pos + 2 < buffer.len() {
                                    buffer[pos] = r;
                                    buffer[pos + 1] = g;
                                    buffer[pos + 2] = b;
                                }
                            }
                        }
                    }
                }
            }
        }

        buffer
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
