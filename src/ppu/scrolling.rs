use super::*;

impl PPU {
    pub(crate) fn increment_scroll_x(&mut self) {
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

    pub(crate) fn increment_scroll_y(&mut self) {
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

    pub(crate) fn transfer_address_x(&mut self) {
        // Transfere componente X do registro t para v
        if (self.mask & 0x08) != 0 || (self.mask & 0x10) != 0 {
            self.v = (self.v & !0x041f) | (self.t & 0x041f);
        }
    }

    pub(crate) fn transfer_address_y(&mut self) {
        // Transfere componente Y do registro t para v
        if (self.mask & 0x08) != 0 || (self.mask & 0x10) != 0 {
            self.v = (self.v & !0x7be0) | (self.t & 0x7be0);
        }
    }
}