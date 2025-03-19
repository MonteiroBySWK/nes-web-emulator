use apu::APU;
use cpu::CPU;
use std::time::{Duration, Instant};

mod mapper;
mod cpu;
mod rom;
mod input;
mod ppu;
mod apu;
mod bus;

fn main() {
    let rom_path = "nestest.nes";

    let rom = match rom::ROM::new(rom_path) {
        Ok(rom) => rom,
        Err(e) => panic!("Error loading ROM: {}", e),
    };
    
    let ppu = ppu::PPU::new();
    let apu = APU::new();
    let bus = bus::BUS::new(ppu, rom, apu);

    let mut cpu: CPU = CPU::new(bus);
    cpu.reset();

    let frame_time = Duration::from_nanos(16_666_667);
    let mut last_frame = Instant::now();

    loop {
        cpu.clock();

        // Frame timing
        let elapsed = last_frame.elapsed();
        if elapsed < frame_time {
            std::thread::sleep(frame_time - elapsed);
        }
        last_frame = Instant::now();
    }
}
