use std::collections::HashMap;
use crate::memory::Memory;

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

impl AddressingModes {
    fn _bytes(&self) -> u8 {
        match self {
            AddressingModes::ZeroPageIndexedX => 2,
            AddressingModes::ZeroPageIndexedY => 2,
            AddressingModes::AbsoluteIndexedX => 3,
            AddressingModes::AbsoluteIndexedY => 3,
            AddressingModes::IndexedIndirect => 2,
            AddressingModes::IndirectIndexed => 2,
            AddressingModes::Implicit => 1,
            AddressingModes::Accumulator => 1,
            AddressingModes::Immediate => 2,
            AddressingModes::ZeroPage => 2,
            AddressingModes::Absolute => 3,
            AddressingModes::Relative => 2,
            AddressingModes::Indirect => 3,
        }
    }
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

struct Registers {
    acc: u8, //A
    index_x: u8, //X
    index_y: u8, //Y
    stack_pointer: u8, //S
    status_register: u8, //P -> NV1B DIZC (Flags)
    program_counter: u16, //PC
}

type Opcode = u8;
type Instrution<'a> = fn(&mut CPU<'a>, AddressingModes);
type OpcodeFunction<'a> = (Instrution<'a>, AddressingModes);
type InstructionMap<'a> = HashMap<Opcode, OpcodeFunction<'a>>; // Opcode, Instrution

pub struct CPU<'a> {
    registers: Registers,
    instructions: InstructionMap<'a>,
    prg: Vec<u8>,
    memory: &'a mut Memory,
}

impl<'a> CPU<'a> {
    pub fn new(prg_rom: Vec<u8>, memory: &'a mut Memory) -> Self {
        let mut c: CPU = CPU {
            registers: Registers {
                acc: 0,
                index_x: 0,
                index_y: 0,
                stack_pointer: 0,
                status_register: UNUSED,
                program_counter: 0,
            },
            prg: prg_rom,
            instructions: HashMap::new(),
            memory: memory,
        };
        c.map_instructions();

        c
    }

    fn fetch(&mut self) -> u8 {
        let opcode: u8 = self.memory.read(self.registers.program_counter);
        self.registers.program_counter = self.registers.program_counter.wrapping_add(1);
        opcode
    }

    fn decode(&mut self) -> Option<OpcodeFunction<'a>> {
        let opcode: u8 = self.fetch();
        println!("Fetched Opcode: 0x{:X} - PC: {}", opcode, self.registers.program_counter);
        self.instructions.get(&opcode).copied()
    }

    pub fn execute(&mut self) {
        let op_function: Option<OpcodeFunction<'a>> = self.decode();
        if let Some((func, mode)) = op_function {
            func(self, mode);
        } else {
            println!("Opcode not found");
        }
    }

    fn turn_in_u16(&self, low: u8, high: u8) -> u16 {
        ((high as u16) << 8) | (low as u16)
    }

    fn execute_mode(&mut self, mode: AddressingModes) -> u8 {
        match mode {
            AddressingModes::Immediate => { self.fetch() }
            AddressingModes::ZeroPage => {
                let address: u8 = self.fetch();

                self.memory.read(address as u16)
            }
            AddressingModes::ZeroPageIndexedX => {
                let base: u8 = self.fetch();
                let indexed_address: u8 = base.wrapping_add(self.registers.index_x);

                self.memory.read(indexed_address as u16)
            }
            AddressingModes::ZeroPageIndexedY => {
                let base: u8 = self.fetch();
                let indexed_address: u8 = base.wrapping_add(self.registers.index_y);

                self.memory.read(indexed_address as u16)
            }
            AddressingModes::Absolute => {
                let low: u8 = self.fetch();
                let high: u8 = self.fetch();
                let address: u16 = self.turn_in_u16(low, high);

                self.memory.read(address)
            }
            AddressingModes::AbsoluteIndexedX => {
                let low: u8 = self.fetch();
                let high: u8 = self.fetch();
                let address: u16 = self.turn_in_u16(low, high);
                let indexed_address: u16 = address.wrapping_add(self.registers.index_x as u16);

                self.memory.read(indexed_address)
            }
            AddressingModes::AbsoluteIndexedY => {
                let low: u8 = self.fetch();
                let high: u8 = self.fetch();
                let address: u16 = self.turn_in_u16(low, high);
                let indexed_address: u16 = address.wrapping_add(self.registers.index_y as u16);

                self.memory.read(indexed_address)
            }
            AddressingModes::IndexedIndirect => {
                let base: u8 = self.fetch();
                let ptr: u8 = base.wrapping_add(self.registers.index_x);
                let low: u8 = self.memory.read(ptr as u16);
                let high: u8 = self.memory.read(ptr.wrapping_add(1) as u16);
                let indirect_address: u16 = self.turn_in_u16(low, high);

                self.memory.read(indirect_address)
            }
            AddressingModes::IndirectIndexed => {
                let base: u8 = self.fetch();
                let low: u8 = self.memory.read(base as u16);
                let high: u8 = self.memory.read(base.wrapping_add(1) as u16);
                let indirect_address: u16 = self.turn_in_u16(low, high);
                let indexed_address: u16 = indirect_address.wrapping_add(
                    self.registers.index_y as u16
                );

                self.memory.read(indexed_address)
            }
            AddressingModes::Indirect => {
                let low_ptr: u8 = self.fetch();
                let high_ptr: u8 = self.fetch();
                let ptr: u16 = self.turn_in_u16(low_ptr, high_ptr);
                let low = self.memory.read(ptr);
                let high = if (ptr & 0x00ff) == 0x00ff {
                    self.memory.read(ptr & 0xff00)
                } else {
                    self.memory.read(ptr + 1)
                };
                let effective_address = self.turn_in_u16(low, high);

                self.memory.read(effective_address)
            }
            AddressingModes::Relative => {
                let offset = self.fetch() as i8;
                let address = self.registers.program_counter.wrapping_add(offset as u16);

                self.memory.read(address)
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
                // Agora, utilizando memory.read em vez de acesso direto a prg
                let base = self.fetch();
                let ptr = base.wrapping_add(self.registers.index_x);
                let low = self.memory.read(ptr as u16);
                let high = self.memory.read(ptr.wrapping_add(1) as u16);
                self.turn_in_u16(low, high)
            }
            AddressingModes::IndirectIndexed => {
                let base = self.fetch();
                let low = self.memory.read(base as u16);
                let high = self.memory.read(base.wrapping_add(1) as u16);
                let addr = self.turn_in_u16(low, high);
                addr.wrapping_add(self.registers.index_y as u16)
            }
            AddressingModes::Indirect => {
                let low = self.fetch();
                let high = self.fetch();
                let addr = self.turn_in_u16(low, high);
                // Simula o bug de hardware do 6502 quando o endereço está em limite de página
                let low_addr = self.memory.read(addr);
                let high_addr = if (addr & 0x00ff) == 0x00ff {
                    self.memory.read(addr & 0xff00)
                } else {
                    self.memory.read(addr + 1)
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
        if condition {
            let offset = self.fetch() as i8;
            let old_pc = self.registers.program_counter;
            self.registers.program_counter = old_pc.wrapping_add(offset as u16);
        } else {
            // Se não ramificar, ainda precisamos avançar o PC
            self.registers.program_counter = self.registers.program_counter.wrapping_add(1);
        }
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
}
