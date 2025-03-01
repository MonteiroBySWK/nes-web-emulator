use std::{fs, num::ParseIntError};
mod cpu;
use cpu::Cpu;

fn main() {
    let mut chip = Cpu::new();
    chip.load_rom("Zelda.nes").unwrap();
    loop {
     chip.execute();
    }
}
