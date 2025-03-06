use crate::{ apu::APU, cpu::CPU, memory::Memory, ppu::PPU, rom::ROM };

pub fn run(rom_path: &str) {
    let rom = ROM::new(rom_path).expect("ROM carregada");

    let mut memory = Memory {
        ram: [0; 2048],
        vram: [0; 8190],
        rom: rom,
    };

    let mut cpu = CPU::new(memory.rom.prg_rom.clone(), &mut memory);
    let ppu = PPU::new();
    let apu = APU::new();
    
    loop {
        cpu.execute();
    }
}
