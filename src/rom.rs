use std::io::{Error, ErrorKind};
use std::fs;
use std::path::Path;
use crate::mapper::{Mapper, Mapper0, Mapper1};

pub struct ROM {
    pub header: Vec<u8>,
    pub mapper: Box<dyn Mapper>,
    pub mirroring: Mirroring,
    pub battery_backed: bool,
    pub mapper_number: u8,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Mirroring {
    Horizontal,
    Vertical,
    OneScreenLo,
    OneScreenHi,
    FourScreen,
}

impl ROM {
    pub fn new<P: AsRef<Path>>(rom_path: P) -> Result<ROM, Error> {
        let contents: Vec<u8> = fs::read(rom_path)?;
        Self::from_bytes(&contents)
    }

    pub fn from_bytes(contents: &[u8]) -> Result<ROM, Error> {
        let _header = &contents[0..16];
        // Log ROM size and header for debugging
        web_sys::console::log_1(&format!(
            "Loading ROM - Size: {}, Header: {:?}",
            contents.len(),
            &contents[0..16].iter().map(|b| format!("{:02x}", b)).collect::<Vec<_>>()
        ).into());

        // Verify minimum size and iNES header
        if contents.len() < 16 || &contents[0..4] != b"NES\x1A" {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "Invalid iNES header"
            ));
        }

        let header = contents[0..16].to_vec();
        let prg_rom_size = contents[4] as usize * 16384;
        let chr_rom_size = contents[5] as usize * 8192;

        // Get mapper number and flags
        let mapper_number = (contents[6] >> 4) | (contents[7] & 0xF0);
        let vertical_mirroring = (contents[6] & 0x01) != 0;
        let battery_backed = (contents[6] & 0x02) != 0;
        let has_trainer = (contents[6] & 0x04) != 0;
        let four_screen = (contents[6] & 0x08) != 0;

        // Log detailed ROM info
        web_sys::console::log_1(&format!(
            "ROM Info - PRG: {}KB, CHR: {}KB, Mapper: {}, Flags: v={} b={} t={} f={}",
            prg_rom_size / 1024,
            chr_rom_size / 1024,
            mapper_number,
            vertical_mirroring,
            battery_backed,
            has_trainer,
            four_screen
        ).into());

        // Calculate offsets and verify size
        let header_size = 16 + if has_trainer { 512 } else { 0 };
        web_sys::console::log_1(&format!("Header Size: {}", header_size).into());
        let expected_size = header_size + prg_rom_size + chr_rom_size;
        if contents.len() < expected_size {
            return Err(Error::new(
                ErrorKind::UnexpectedEof,
                format!(
                    "ROM too small. Expected {}, got {}",
                    expected_size,
                    contents.len()
                )
            ));
        }

        // Extract PRG-ROM (with bounds checking)
        let prg_rom_start = header_size;
        let prg_rom_end = prg_rom_start + prg_rom_size;
        web_sys::console::log_1(&format!("PRG ROM Start: {}, End: {}", prg_rom_start, prg_rom_end).into());
        if prg_rom_end > contents.len() {
            return Err(Error::new(
                ErrorKind::UnexpectedEof,
                format!("PRG ROM ends at {}, but ROM size is {}", prg_rom_end, contents.len())
            ));
        }
        let prg_rom = contents[prg_rom_start..prg_rom_end].to_vec();
        web_sys::console::log_1(&format!("PRG ROM Size: {}", prg_rom.len()).into());

        // Create CHR-ROM/RAM
        let chr_rom = if chr_rom_size == 0 {
            web_sys::console::log_1(&"CHR ROM Size is 0, creating CHR RAM".into());
            vec![0; 8192] // 8KB of CHR-RAM
        } else {
            let chr_rom_start = prg_rom_end;
            let chr_rom_end = chr_rom_start + chr_rom_size;
            web_sys::console::log_1(&format!("CHR ROM Start: {}, End: {}", chr_rom_start, chr_rom_end).into());
            if chr_rom_end > contents.len() {
                return Err(Error::new(
                    ErrorKind::UnexpectedEof,
                    format!("CHR ROM ends at {}, but ROM size is {}", chr_rom_end, contents.len())
                ));
            }
            let chr_rom = contents[chr_rom_start..chr_rom_end].to_vec();
            web_sys::console::log_1(&format!("CHR ROM Size: {}", chr_rom.len()).into());
            chr_rom
        };

        // Determine mirroring type
        let mirroring = if four_screen {
            Mirroring::FourScreen
        } else if vertical_mirroring {
            Mirroring::Vertical
        } else {
            Mirroring::Horizontal
        };

        Ok(ROM {
            header: contents[0..16].to_vec(),
            mapper: match mapper_number {
                0 => Box::new(Mapper0::new(
                    prg_rom,
                    chr_rom,
                    contents[4],
                    contents[5],
                    mirroring
                )),
                1 => Box::new(Mapper1::new(
                    prg_rom,
                    chr_rom,
                    contents[4],
                    contents[5]
                )),
                _ => return Err(Error::new(
                    ErrorKind::Unsupported,
                    format!("Unsupported mapper: {}", mapper_number)
                )),
            },
            mirroring,
            battery_backed,
            mapper_number,
        })
    }

    fn parse_header(_header: &[u8]) -> Result<(), &'static str> {
        // Implementation here
        Ok(())
    }

    pub fn read(&self, address: u16) -> u8 {
        self.mapper.read_prg(address)
    }

    pub fn write(&mut self, address: u16, value: u8) {
        self.mapper.write_prg(address, value);
    }

    pub fn read_chr(&self, address: u16) -> u8 {
        self.mapper.read_chr(address)
    }

    pub fn write_chr(&mut self, address: u16, value: u8) {
        self.mapper.write_chr(address, value);
    }

    pub fn get_mirroring(&self) -> Mirroring {
        self.mapper.get_mirroring()
    }
}
