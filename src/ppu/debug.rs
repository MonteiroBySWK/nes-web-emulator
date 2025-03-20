use super::*;
use super::colors::convert_color;

impl PPU {
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

                        let (r, g, b) = convert_color(color_index);

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
                                let (r, g, b) = convert_color(color_index);

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
}