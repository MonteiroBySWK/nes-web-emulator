mod cpu;
mod rom;

mod memory;
mod ppu;
mod apu;
mod emulator;

use emulator::run;

fn main() {
    let rom_path = "Zelda.nes";

    run(rom_path);
}
