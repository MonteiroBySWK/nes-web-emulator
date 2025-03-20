use super::*;

impl PPU {
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

    fn mirror_nametable_address(&self, addr: u16) -> u16 {
        let addr = addr & 0x2fff;
        let nametable_index = (addr - 0x2000) / 0x400;
        let offset = (addr - 0x2000) % 0x400;

        let mirrored_nametable = match self.mirroring {
            Mirroring::Horizontal => {
                match nametable_index {
                    0 => 0,
                    1 => 0,
                    2 => 2,
                    3 => 2,
                    _ => 0,
                }
            }
            Mirroring::Vertical => {
                match nametable_index {
                    0 => 0,
                    1 => 1,
                    2 => 0,
                    3 => 1,
                    _ => 0,
                }
            }
            Mirroring::FourScreen => nametable_index,
            Mirroring::OneScreenLo => 0,
            Mirroring::OneScreenHi => 1,
        };

        0x2000 + mirrored_nametable * 0x400 + offset
    }
}
