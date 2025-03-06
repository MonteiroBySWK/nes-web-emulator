use std::io::Error;
use std::fs;
use std::ptr::addr_eq;

pub struct ROM {
    pub chr_rom: Vec<u8>, // Gráfico 0x0000 - 0x1FFF
    pub prg_rom: Vec<u8>, // Codigo 0x8000 - 0xFFFF
    // mapper: Box<dyn Mapper>,
}

pub trait Mapper {
    fn read_prg(&self, address: u16) -> u8;
    fn write_prg(&mut self, address: u16, value: u8);
    fn read_chr(&self, address: u16) -> u8;
}

impl ROM {
    pub fn new(rom_path: &str) -> Result<ROM, Error> {
        let contents: Vec<u8> = fs::read(rom_path)?;

        if contents[0..4] != [0x4e, 0x45, 0x53, 0x1a] {
            return Err(
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Arquivo NES inválido: cabeçalho incorreto"
                )
            );
        }

        const KB_SIZE: usize = 1024;

        let prg_rom_size: usize = (contents[4] as usize) * 16 * KB_SIZE;
        let chr_rom_size: usize = (contents[5] as usize) * 8 * KB_SIZE;

        // 6 - 10 são flags

        // Flag 6
        let vertical_mirroring: bool = (contents[6] & 0b0000_0001) != 0;
        let battery_backed: bool = (contents[6] & 0b0000_0010) != 0;
        let has_trainer: bool = (contents[6] & 0b0000_0100) != 0;
        let four_screen_vram: bool = (contents[6] & 0b0000_1000) != 0;
        let mapper_number: u8 = (contents[6] >> 4) | (contents[7] & 0xf0);

        // Flag 7
        let vs_unisystem: bool = (contents[7] & 0b0000_0001) != 0;
        let playchoice_10: bool = (contents[7] & 0b0000_0010) != 0;
        let nes_2_0: bool = (contents[7] & 0b0000_1100) == 0b0000_1000;

        // Flag 8
        let prg_ram_size: u8 = contents[8];

        // Flag 9
        let tv_system: bool = (contents[9] & 0b0000_0001) != 0;

        // Flag 10
        // (bits 0-1) TV system
        // (bit 2) PRG RAM ($6000-$7FFF) (0: present; 1: not present)
        // (bit 4) Bus conflicts (0: no conflicts; 1: conflicts)
        let prg_ram_present: bool = (contents[10] & 0b0000_0100) == 0;
        let bus_conflicts: bool = (contents[10] & 0b0001_0000) != 0;

        let chr_rom: Vec<u8>;
        if chr_rom_size == 0 {
            chr_rom = vec![0; 8 * KB_SIZE]; //CHR_RAM
        } else {
            let chr_rom_start: usize = 16 + prg_rom_size + (if has_trainer { 512 } else { 0 });
            chr_rom = contents[chr_rom_start..chr_rom_start + chr_rom_size].to_vec();
        }

        let prg_rom_start: usize = 16 + (if has_trainer { 512 } else { 0 });
        let prg_rom = contents[prg_rom_start..prg_rom_start + prg_rom_size].to_vec();

        // Imprime as variáveis, exceto os vetores
        let debug: bool = true;
        if debug {
            println!("vertical_mirroring: {}", vertical_mirroring);
            println!("battery_backed: {}", battery_backed);
            println!("trainer_present: {}", has_trainer);
            println!("four_screen_vram: {}", four_screen_vram);
            println!("mapper_number: {}", mapper_number);
            println!("vs_unisystem: {}", vs_unisystem);
            println!("playchoice_10: {}", playchoice_10);
            println!("nes_2_0: {}", nes_2_0);
            println!("prg_ram_size: {}", prg_ram_size);
            println!("tv_system: {}", tv_system);
            println!("prg_ram_present: {}", prg_ram_present);
            println!("bus_conflicts: {}", bus_conflicts);
        }
        // For now, defaulting to Mapper0
        // let mapper = Box::new(Mapper0 {});

        Ok(ROM {
            prg_rom,
            chr_rom,
            //  mapper,
        })
    }

    pub fn read(&self, address: u16) -> u8 {
        let addr = address as usize - 0x8000;
        if addr < self.prg_rom.len() {
            self.prg_rom[addr]
        } else {
            0
        }
    }


}

struct Mapper0 {}
impl Mapper0 {}

struct Mapper1 {}
impl Mapper1 {}
