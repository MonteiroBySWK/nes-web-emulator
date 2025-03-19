use crate::{ apu::APU, input::Controller, ppu::PPU, rom::ROM };

pub struct BUS {
    pub ppu: PPU,
    ram: [u8; 2048],
    pub rom: ROM,
    apu: APU,
    pub controller:Controller,
}

impl BUS {
    pub fn new(ppu: PPU, rom: ROM, apu: APU) -> Self {
        BUS {
            ppu,
            ram: [0; 2048],
            rom,
            apu,
            controller: Controller::new(),
        }
    }

    pub fn read(&mut self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1FFF => self.ram[addr as usize & 0x07FF],
            0x2000..=0x3FFF => self.ppu.read_register(addr & 0x7),
            0x4016 => {
                // Controller read
                let mut result = 0u8;
                if self.controller.a { result |= 0x01; }
                if self.controller.b { result |= 0x02; }
                if self.controller.select { result |= 0x04; }
                if self.controller.start { result |= 0x08; }
                if self.controller.up { result |= 0x10; }
                if self.controller.down { result |= 0x20; }
                if self.controller.left { result |= 0x40; }
                if self.controller.right { result |= 0x80; }
                result
            },
            0x8000..=0xFFFF => self.rom.read(addr), // Mapper handles the memory mapping
            _ => 0
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000..=0x1FFF => self.ram[addr as usize & 0x07FF] = value,
            0x2000..=0x3FFF => self.ppu.write_register(addr & 0x7, value),
            0x4014 => {
                // DMA transfer to PPU OAM
                let base = (value as u16) << 8;
                for i in 0..256 {
                    let data = self.read(base + i);
                    self.ppu.write_register(0x2004, data);
                }
            },
            0x4016 => {
                if value & 1 == 1 {
                    self.controller = Controller::new();
                }
            },
            0x8000..=0xFFFF => self.rom.write(addr, value), // Mapper handles bank switching
            _ => { /* Ignore writes to other addresses */ }
        }
    }
}
