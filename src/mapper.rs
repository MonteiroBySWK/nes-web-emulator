use crate::rom::Mirroring;

pub trait Mapper {
    fn read_prg(&self, address: u16) -> u8;
    fn write_prg(&mut self, address: u16, value: u8);
    fn read_chr(&self, address: u16) -> u8;
    fn write_chr(&mut self, address: u16, value: u8);
    fn get_mirroring(&self) -> Mirroring;
    fn get_chr_rom(&self) -> &[u8];
}

pub struct Mapper0 {
    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,
    prg_banks: u8,
    chr_banks: u8,
    mirroring: Mirroring,
}

impl Mapper0 {
    pub fn new(
        prg_rom: Vec<u8>,
        chr_rom: Vec<u8>,
        prg_banks: u8,
        chr_banks: u8,
        mirroring: Mirroring
    ) -> Self {
        Self {
            prg_rom,
            chr_rom,
            prg_banks,
            chr_banks,
            mirroring,
        }
    }
}

impl Mapper for Mapper0 {
    fn read_prg(&self, address: u16) -> u8 {
        match address {
            0x8000..=0xBFFF => self.prg_rom[(address as usize - 0x8000) % self.prg_rom.len()],
            0xC000..=0xFFFF => self.prg_rom[(address as usize - 0xC000) % self.prg_rom.len()],
            _ => 0, // Endereços fora da faixa do PRG ROM
        }
    }

    fn write_prg(&mut self, _address: u16, _value: u8) {
        // Mapper 0 doesn't support PRG RAM writes
    }

    fn read_chr(&self, address: u16) -> u8 {
        self.chr_rom[(address & 0x1fff) as usize]
    }

    fn write_chr(&mut self, _address: u16, _value: u8) {
        // Mapper 0 doesn't support CHR writes
    }

    fn get_mirroring(&self) -> Mirroring {
        self.mirroring
    }

    fn get_chr_rom(&self) -> &[u8] {
        &self.chr_rom
    }
}

pub struct Mapper1 {
    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,
    shift_register: u8,
    control: u8,
    chr_bank_0: u8,
    chr_bank_1: u8,
    prg_bank: u8,
    prg_banks: u8,
    chr_banks: u8,
    shift_count: u8,
}

impl Mapper1 {
    pub fn new(prg_rom: Vec<u8>, chr_rom: Vec<u8>, prg_banks: u8, chr_banks: u8) -> Self {
        Self {
            prg_rom,
            chr_rom,
            shift_register: 0x10,
            control: 0x0c,
            chr_bank_0: 0,
            chr_bank_1: 0,
            prg_bank: 0,
            prg_banks,
            chr_banks,
            shift_count: 0,
        }
    }
}

impl Mapper for Mapper1 {
    fn read_prg(&self, address: u16) -> u8 {
        match (self.control >> 2) & 0x3 {
            0 | 1 => {
                // 32KB switching
                let bank = (self.prg_bank & 0xfe) as usize;
                let offset = (address & 0x7fff) as usize;
                let addr = bank * 0x8000 + offset;
                if addr < self.prg_rom.len() {
                    self.prg_rom[addr]
                } else {
                    web_sys::console::log_1(
                        &format!(
                            "PRG Read Out of Bounds (32KB Mode): Address {:04X}, Index {}",
                            address,
                            addr
                        ).into()
                    );
                    0
                }
            }
            2 => {
                // fix first bank ($8000-$BFFF), switch second ($C000-$FFFF)
                if address < 0xc000 {
                    let addr = (address & 0x3fff) as usize;
                    if addr < self.prg_rom.len() {
                        self.prg_rom[addr]
                    } else {
                        web_sys::console::log_1(
                            &format!(
                                "PRG Read Out of Bounds (Fixed First): Address {:04X}, Index {}",
                                address,
                                addr
                            ).into()
                        );
                        0
                    }
                } else {
                    let bank = (self.prg_bank & 0x0f) as usize;
                    let offset = (address & 0x3fff) as usize;
                    let addr = bank * 0x4000 + offset;
                    if addr < self.prg_rom.len() {
                        self.prg_rom[addr]
                    } else {
                        web_sys::console::log_1(
                            &format!(
                                "PRG Read Out of Bounds (Switched Second): Address {:04X}, Index {}",
                                address,
                                addr
                            ).into()
                        );
                        0
                    }
                }
            }
            3 => {
                // fix last bank ($C000-$FFFF), switch first ($8000-$BFFF)
                if address >= 0xc000 {
                    let offset = (address & 0x3fff) as usize;
                    let last_bank_start = ((self.prg_banks as usize) - 1) * 0x4000;
                    let addr = last_bank_start + offset;
                    if addr < self.prg_rom.len() {
                        self.prg_rom[addr]
                    } else {
                        web_sys::console::log_1(
                            &format!(
                                "PRG Read Out of Bounds (Fixed Last): Address {:04X}, Index {}",
                                address,
                                addr
                            ).into()
                        );
                        0
                    }
                } else {
                    let bank = (self.prg_bank & 0x0f) as usize;
                    let offset = (address & 0x3fff) as usize;
                    let addr = bank * 0x4000 + offset;
                    if addr < self.prg_rom.len() {
                        self.prg_rom[addr]
                    } else {
                        web_sys::console::log_1(
                            &format!(
                                "PRG Read Out of Bounds (Switched First): Address {:04X}, Index {}",
                                address,
                                addr
                            ).into()
                        );
                        0
                    }
                }
            }
            _ => unreachable!(),
        }
    }

    fn write_prg(&mut self, address: u16, value: u8) {
        // Reset do registrador de deslocamento se bit 7 está setado
        if (value & 0x80) != 0 {
            self.shift_register = 0x10;
            self.shift_count = 0; // Também resetar o contador
            self.control |= 0x0c;
            return;
        }

        // Deslocamento e adição do novo bit
        self.shift_register >>= 1;
        self.shift_register |= (value & 1) << 4;
        self.shift_count += 1;

        // Quando acumular 5 bits, processar o registro
        if self.shift_count == 5 {
            let target_register = (address >> 13) & 0x3;
            let value = self.shift_register & 0x1f;

            match target_register {
                0 => {
                    // Control
                    self.control = value;
                    // Atualizar espelhamento imediatamente
                }
                1 => {
                    // CHR bank 0
                    self.chr_bank_0 = value;
                }
                2 => {
                    // CHR bank 1
                    self.chr_bank_1 = value;
                }
                3 => {
                    // PRG bank
                    self.prg_bank = value & 0x0f; // Máscara para 4 bits
                }
                _ => unreachable!(),
            }

            // Reset após processamento
            self.shift_register = 0x10;
            self.shift_count = 0;
        }
    }

    fn read_chr(&self, address: u16) -> u8 {
        let addr = if (self.control & 0x10) == 0 {
            // 8KB mode
            (((self.chr_bank_0 as usize) & 0x1e) * 0x1000 + ((address as usize) & 0x1fff)) %
                self.chr_rom.len()
        } else {
            // 4KB mode
            if address <= 0x0fff {
                ((self.chr_bank_0 as usize) * 0x1000 + ((address as usize) & 0x0fff)) %
                    self.chr_rom.len()
            } else {
                ((self.chr_bank_1 as usize) * 0x1000 + ((address as usize) & 0x0fff)) %
                    self.chr_rom.len()
            }
        };
        self.chr_rom[addr]
    }

    fn write_chr(&mut self, address: u16, value: u8) {
        let addr = if (self.control & 0x10) == 0 {
            (((self.chr_bank_0 as usize) & 0x1e) * 0x1000 + ((address as usize) & 0x1fff)) %
                self.chr_rom.len()
        } else {
            if address <= 0x0fff {
                ((self.chr_bank_0 as usize) * 0x1000 + ((address as usize) & 0x0fff)) %
                    self.chr_rom.len()
            } else {
                ((self.chr_bank_1 as usize) * 0x1000 + ((address as usize) & 0x0fff)) %
                    self.chr_rom.len()
            }
        };
        self.chr_rom[addr] = value;
    }

    fn get_mirroring(&self) -> Mirroring {
        match self.control & 0x3 {
            0 => Mirroring::OneScreenLo,
            1 => Mirroring::OneScreenHi,
            2 => Mirroring::Vertical,
            3 => Mirroring::Horizontal,
            _ => unreachable!(),
        }
    }
    fn get_chr_rom(&self) -> &[u8] {
        &self.chr_rom
    }
}
