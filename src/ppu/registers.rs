use super::*;

impl PPU {
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
}
