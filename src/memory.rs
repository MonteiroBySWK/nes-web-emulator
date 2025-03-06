use crate::rom::ROM;

pub struct Memory {
    pub ram: [u8; 2048], 
    pub vram: [u8; 8190], 
    pub rom: ROM,
}

impl Memory {
    pub fn read(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x07ff => self.ram[address as usize],
            0x2000..=0x3fff => self.vram[(address % 0x2000) as usize],
            0x8000..=0xffff => self.rom.read(address),
            _ => 0,
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x07ff => self.ram[address as usize] = value,
            0x2000..=0x3fff => self.vram[(address % 0x2000) as usize] = value,
            _ => {},
        }
    } 
}
