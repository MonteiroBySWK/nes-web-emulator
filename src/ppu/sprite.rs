use super::*;

impl PPU {
    pub(crate) fn evaluate_sprites(&mut self) {
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

    pub(crate) fn get_sprite_pixel(&self, x: u8) -> (u8, u8, bool) {
        for i in 0..self.sprite_count {
            let pattern = self.sprite_patterns[i];
            let sprite_x = self.sprite_positions[i];
            let priority = self.sprite_priorities[i];
            
            let offset = x as i16 - sprite_x as i16;
            if offset >= 0 && offset < 8 {
                let sprite_pixel = ((pattern >> (7 - offset)) & 0x01) |
                                 (((pattern >> 8) >> (7 - offset)) & 0x01) << 1;

                if sprite_pixel != 0 {
                    let sprite_palette = 4 + ((self.secondary_oam[i * 4 + 2] & 0x03) as u8);
                    let priority = priority == 0;

                    return (sprite_pixel as u8, sprite_palette, priority);
                }
            }
        }
        (0, 0, false)
    }
}