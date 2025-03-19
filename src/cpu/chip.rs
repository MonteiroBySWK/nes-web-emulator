use std::collections::HashMap;
use crate::bus::BUS;

// Flags 0b NV1B DIZC
const CARRY: u8 = 0b0000_0001;
const ZERO: u8 = 0b0000_0010;
const INTERRUPT_DISABLE: u8 = 0b0000_0100;
const DECIMAL: u8 = 0b0000_1000;
const BREAK: u8 = 0b0001_0000;
const UNUSED: u8 = 0b0010_0000;
const OVERFLOW: u8 = 0b0100_0000;
const NEGATIVE: u8 = 0b1000_0000;

#[derive(Clone, Copy, PartialEq)]
pub enum AddressingModes {
    ZeroPageIndexedX, // 2 bytes
    ZeroPageIndexedY, // 2 bytes
    AbsoluteIndexedX, // 3 bytes
    AbsoluteIndexedY, // 3 bytes
    IndexedIndirect, // 2 bytes
    IndirectIndexed, // 2 bytes

    //Others
    Implicit, // 1 byte
    Accumulator, // 1 byte
    Immediate, // 2 bytes
    ZeroPage, // 2 bytes
    Absolute, // 3 bytes
    Relative, // 2 bytes
    Indirect, // 3 bytes
}

pub enum FlagUpdate {
    Zero(u8),
    Negative(u8),
    Carry(bool),
    Overflow(bool),
    InterruptDisable(bool),
    Decimal(bool),
    Break(bool),
}

pub struct Registers {
    pub acc: u8, //A
    pub index_x: u8, //X
    pub index_y: u8, //Y
    pub stack_pointer: u8, //S
    pub status_register: u8, //P -> NV1B DIZC (Flags)
    pub program_counter: u16, //PC
}

type Opcode = u8;
type Instruction = fn(&mut CPU, AddressingModes);
type OpcodeFunction = (Instruction, AddressingModes);
type InstructionMap = HashMap<Opcode, OpcodeFunction>; // Opcode, Instrution

pub struct CPU {
    pub registers: Registers,
    instructions: InstructionMap,
    pub bus: BUS,
    // Campo para controle dos ciclos da instrução atual
    pub remaining_cycles: u32,
}

impl CPU {

    pub fn new(bus: BUS) -> Self {
        CPU {
            registers: Registers {
                acc: 0,
                index_x: 0,
                index_y: 0,
                stack_pointer: 0,
                status_register: UNUSED,
                program_counter: 0,
            },
            instructions: HashMap::new(),
            bus,
            remaining_cycles: 0,
        }
    }

    fn fetch(&mut self) -> u8 {
        let opcode: u8 = self.read(self.registers.program_counter);
        self.registers.program_counter = self.registers.program_counter.wrapping_add(1);
        opcode
    }

    fn decode(&mut self) -> Option<OpcodeFunction> {
        let opcode: u8 = self.fetch();

        if self.instructions.is_empty() {
            self.map_instructions();
        }

        println!("Fetched Opcode: 0x{:X} - PC: {}", opcode, self.registers.program_counter);
        self.instructions.get(&opcode).copied()
    }

    fn execute(&mut self) {
        let op_function: Option<OpcodeFunction> = self.decode();
        if let Some((func, mode)) = op_function {
            // Log para debug
            web_sys::console::log_1(&format!(
                "Executing instruction at PC: {:04X}, A: {:02X}, X: {:02X}, Y: {:02X}",
                self.registers.program_counter,
                self.registers.acc,
                self.registers.index_x,
                self.registers.index_y
            ).into());
            
            func(self, mode);
        } else {
            println!("Opcode not found");
        }
    }

    pub fn reset(&mut self) {
        self.registers.acc = 0;
        self.registers.index_x = 0;
        self.registers.index_y = 0;
        self.registers.stack_pointer = 0xFD;
        self.registers.status_register = UNUSED | INTERRUPT_DISABLE;
        
        // Lê o endereço de reset do vetor em 0xFFFC
        let reset_vector = self.read_u16(0xFFFC);
        web_sys::console::log_1(&format!(
            "Reset vector: {:04X}, ROM data at vector: {:02X} {:02X}", 
            reset_vector,
            self.read(reset_vector),
            self.read(reset_vector + 1)
        ).into());
        
        self.registers.program_counter = reset_vector;
        self.remaining_cycles = 8;
    }

    pub fn clock(&mut self) -> bool {
        let mut frame_complete = false;

        // Se não há ciclos pendentes, busca e executa uma nova instrução
        if self.remaining_cycles == 0 {
            let pc_before = self.registers.program_counter;
            self.execute();
            
            // Debug log
            web_sys::console::log_1(&format!(
                "CPU executed at {:04X} -> {:04X}, A:{:02X} X:{:02X} Y:{:02X} P:{:02X}", 
                pc_before,
                self.registers.program_counter,
                self.registers.acc,
                self.registers.index_x,
                self.registers.index_y,
                self.registers.status_register
            ).into());
            
            // Cada instrução deve definir seus próprios ciclos
            // Por enquanto, usando um valor padrão mais realista
            self.remaining_cycles = 4;
        }

        // Consome um ciclo
        if self.remaining_cycles > 0 {
            self.remaining_cycles -= 1;
        }

        // Avança a PPU (3:1 ratio)
        for _ in 0..3 {
            let step_result = self.bus.ppu.step();
            
            if step_result.vblank_nmi {
                self.trigger_nmi();
            }
            
            if step_result.new_frame {
                frame_complete = true;
            }
        }

        frame_complete
    }

    fn read(&mut self, address: u16) -> u8 {
        self.bus.read(address)
    }

    fn write(&mut self, address: u16, value: u8) {
        self.bus.write(address, value);
    }

    fn turn_in_u16(&self, low: u8, high: u8) -> u16 {
        ((high as u16) << 8) | (low as u16)
    }

    fn execute_mode(&mut self, mode: AddressingModes) -> u8 {
        match mode {
            AddressingModes::Immediate => { self.fetch() }
            AddressingModes::ZeroPage => {
                let address: u8 = self.fetch();

                self.read(address as u16)
            }
            AddressingModes::ZeroPageIndexedX => {
                let base: u8 = self.fetch();
                let indexed_address: u8 = base.wrapping_add(self.registers.index_x);

                self.read(indexed_address as u16)
            }
            AddressingModes::ZeroPageIndexedY => {
                let base: u8 = self.fetch();
                let indexed_address: u8 = base.wrapping_add(self.registers.index_y);

                self.read(indexed_address as u16)
            }
            AddressingModes::Absolute => {
                let low: u8 = self.fetch();
                let high: u8 = self.fetch();
                let address: u16 = self.turn_in_u16(low, high);

                self.read(address)
            }
            AddressingModes::AbsoluteIndexedX => {
                let low: u8 = self.fetch();
                let high: u8 = self.fetch();
                let address: u16 = self.turn_in_u16(low, high);
                let indexed_address: u16 = address.wrapping_add(self.registers.index_x as u16);

                self.read(indexed_address)
            }
            AddressingModes::AbsoluteIndexedY => {
                let low: u8 = self.fetch();
                let high: u8 = self.fetch();
                let address: u16 = self.turn_in_u16(low, high);
                let indexed_address: u16 = address.wrapping_add(self.registers.index_y as u16);

                self.read(indexed_address)
            }
            AddressingModes::IndexedIndirect => {
                let base: u8 = self.fetch();
                let ptr: u8 = base.wrapping_add(self.registers.index_x);
                let low: u8 = self.read(ptr as u16);
                let high: u8 = self.read(ptr.wrapping_add(1) as u16);
                let indirect_address: u16 = self.turn_in_u16(low, high);

                self.read(indirect_address)
            }
            AddressingModes::IndirectIndexed => {
                let base: u8 = self.fetch();
                let low: u8 = self.read(base as u16);
                let high: u8 = self.read(base.wrapping_add(1) as u16);
                let indirect_address: u16 = self.turn_in_u16(low, high);
                let indexed_address: u16 = indirect_address.wrapping_add(
                    self.registers.index_y as u16
                );

                self.read(indexed_address)
            }
            AddressingModes::Indirect => {
                let low_ptr: u8 = self.fetch();
                let high_ptr: u8 = self.fetch();
                let ptr: u16 = self.turn_in_u16(low_ptr, high_ptr);
                let low = self.read(ptr);
                let high = if (ptr & 0x00ff) == 0x00ff {
                    self.read(ptr & 0xff00)
                } else {
                    self.read(ptr + 1)
                };
                let effective_address = self.turn_in_u16(low, high);

                self.read(effective_address)
            }
            AddressingModes::Relative => {
                let offset = self.fetch() as i8;
                let address = self.registers.program_counter.wrapping_add(offset as u16);

                self.read(address)
            }
            _ => 0,
        }
    }

    fn get_operand_address(&mut self, mode: AddressingModes) -> u16 {
        match mode {
            AddressingModes::Immediate => {
                let addr = self.registers.program_counter;
                self.registers.program_counter = self.registers.program_counter.wrapping_add(1);
                addr
            }
            AddressingModes::ZeroPage => {
                let addr = self.fetch() as u16;
                addr
            }
            AddressingModes::ZeroPageIndexedX => {
                let base = self.fetch();
                let addr = base.wrapping_add(self.registers.index_x) as u16;
                addr
            }
            AddressingModes::ZeroPageIndexedY => {
                let base = self.fetch();
                let addr = base.wrapping_add(self.registers.index_y) as u16;
                addr
            }
            AddressingModes::Absolute => {
                let low = self.fetch();
                let high = self.fetch();
                self.turn_in_u16(low, high)
            }
            AddressingModes::AbsoluteIndexedX => {
                let low = self.fetch();
                let high = self.fetch();
                let base = self.turn_in_u16(low, high);
                base.wrapping_add(self.registers.index_x as u16)
            }
            AddressingModes::AbsoluteIndexedY => {
                let low = self.fetch();
                let high = self.fetch();
                let base = self.turn_in_u16(low, high);
                base.wrapping_add(self.registers.index_y as u16)
            }
            AddressingModes::IndexedIndirect => {
                // Agora, utilizando bus.read em vez de acesso direto a prg
                let base = self.fetch();
                let ptr = base.wrapping_add(self.registers.index_x);
                let low = self.read(ptr as u16);
                let high = self.read(ptr.wrapping_add(1) as u16);
                self.turn_in_u16(low, high)
            }
            AddressingModes::IndirectIndexed => {
                let base = self.fetch();
                let low = self.read(base as u16);
                let high = self.read(base.wrapping_add(1) as u16);
                let addr = self.turn_in_u16(low, high);
                addr.wrapping_add(self.registers.index_y as u16)
            }
            AddressingModes::Indirect => {
                let low = self.fetch();
                let high = self.fetch();
                let addr = self.turn_in_u16(low, high);
                // Simula o bug de hardware do 6502 quando o endereço está em limite de página
                let low_addr = self.read(addr);
                let high_addr = if (addr & 0x00ff) == 0x00ff {
                    self.read(addr & 0xff00)
                } else {
                    self.read(addr + 1)
                };
                self.turn_in_u16(low_addr, high_addr)
            }
            AddressingModes::Relative => {
                let offset = self.fetch() as i8;
                self.registers.program_counter.wrapping_add(offset as u16)
            }
            AddressingModes::Accumulator => 0,
            AddressingModes::Implicit => 0,
        }
    }

    fn branch_if(&mut self, condition: bool) {
        let offset = self.fetch() as i8;
        if condition {
            let old_pc = self.registers.program_counter;
            self.registers.program_counter = old_pc.wrapping_add(offset as u16);
        } 
    }

    fn read_u16(&mut self, addr: u16) -> u16 {
        let low = self.read(addr) as u16;
        let high = self.read(addr + 1) as u16;
        (high << 8) | low
    }

    fn push_u16(&mut self, value: u16) {
        let high = (value >> 8) as u8;
        let low = (value & 0xff) as u8;
        self.push(high);
        self.push(low);
    }

    fn push(&mut self, value: u8) {
        self.write(0x0100 | (self.registers.stack_pointer as u16), value);
        self.registers.stack_pointer = self.registers.stack_pointer.wrapping_sub(1);
    }

    fn pull(&mut self) -> u8 {
        self.registers.stack_pointer = self.registers.stack_pointer.wrapping_add(1);
        self.read(0x0100 | (self.registers.stack_pointer as u16))
    }

    fn pull_u16(&mut self) -> u16 {
        let low = self.pull() as u16;
        let high = self.pull() as u16;
        (high << 8) | low
    }

    fn update_flags(&mut self, updates: &[FlagUpdate]) {
        for update in updates {
            match *update {
                FlagUpdate::Zero(val) => {
                    if val == 0 {
                        self.registers.status_register |= ZERO;
                    } else {
                        self.registers.status_register &= !ZERO;
                    }
                }
                FlagUpdate::Negative(val) => {
                    if (val & 0x80) != 0 {
                        self.registers.status_register |= NEGATIVE;
                    } else {
                        self.registers.status_register &= !NEGATIVE;
                    }
                }
                FlagUpdate::Carry(flag) => {
                    if flag {
                        self.registers.status_register |= CARRY;
                    } else {
                        self.registers.status_register &= !CARRY;
                    }
                }
                FlagUpdate::Overflow(flag) => {
                    if flag {
                        self.registers.status_register |= OVERFLOW;
                    } else {
                        self.registers.status_register &= !OVERFLOW;
                    }
                }
                FlagUpdate::InterruptDisable(flag) => {
                    if flag {
                        self.registers.status_register |= INTERRUPT_DISABLE;
                    } else {
                        self.registers.status_register &= !INTERRUPT_DISABLE;
                    }
                }
                FlagUpdate::Decimal(flag) => {
                    if flag {
                        self.registers.status_register |= DECIMAL;
                    } else {
                        self.registers.status_register &= !DECIMAL;
                    }
                }
                FlagUpdate::Break(flag) => {
                    if flag {
                        self.registers.status_register |= BREAK;
                    } else {
                        self.registers.status_register &= !BREAK;
                    }
                }
            }
        }
    }

    pub fn trigger_nmi(&mut self) {
        // Push do PC
        self.push_u16(self.registers.program_counter);
        // Push do status (limpando o bit Break para consistência)
        let status = self.registers.status_register & !BREAK;
        self.push(status);
        // Carrega o vetor NMI (endereço 0xfffa)
        self.registers.program_counter = self.read_u16(0xfffa);
        // Desabilita interruptos
        self.registers.status_register |= INTERRUPT_DISABLE;
    }

    // Nova função para disparar IRQ, se não estiver desabilitado
    pub fn trigger_irq(&mut self) {
        // IRQ só é executado se o flag de interrupção não estiver desabilitado
        if (self.registers.status_register & INTERRUPT_DISABLE) == 0 {
            // Push do PC
            self.push_u16(self.registers.program_counter);
            // Push do status (limpando o bit Break)
            let status = self.registers.status_register & !BREAK;
            self.push(status);
            // Carrega o vetor IRQ (endereço 0xfffe)
            self.registers.program_counter = self.read_u16(0xfffe);
            // Desabilita interruptos
            self.registers.status_register |= INTERRUPT_DISABLE;
        }
    }
}
