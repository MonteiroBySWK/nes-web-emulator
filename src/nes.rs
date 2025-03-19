use crate::cpu::CPU;
pub struct Nes {
    cpu: CPU,
}

impl Nes {
    pub fn new(cpu: CPU) -> Self {
        Nes {
            cpu,
        }
    }

    pub fn tick(&mut self) {
        self.cpu.clock();
    }

    pub fn reset(&mut self) {
        self.cpu.reset();
    }
}