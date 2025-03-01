use std::collections::HashMap;
use std::io::Error;
use std::fs;

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
type Instrution = fn(&mut Cpu, AddressingModes);
type OpcodeFunction = (Instrution, AddressingModes);
type InstructionMap = HashMap<Opcode, OpcodeFunction>; // Opcode, Instrution

pub struct Cpu {
    registers: Registers,
    instructions: InstructionMap,
    memory: [u8; 65536],
}

impl Cpu {
    pub fn new() -> Self {
        let mut c: Cpu = Cpu {
            registers: Registers {
                acc: 0,
                index_x: 0,
                index_y: 0,
                stack_pointer: 0,
                status_register: UNUSED,
                program_counter: 0,
            },
            memory: [0; 65536],
            instructions: HashMap::new(),
        };
        c.map_instructions();

        c
    }

    fn map_instructions(&mut self) {
        // ADC
        self.instructions.insert(0x69, (Cpu::adc, AddressingModes::Immediate));
        self.instructions.insert(0x65, (Cpu::adc, AddressingModes::ZeroPage));
        self.instructions.insert(0x75, (Cpu::adc, AddressingModes::ZeroPageIndexedX));
        self.instructions.insert(0x6d, (Cpu::adc, AddressingModes::Absolute));
        self.instructions.insert(0x7d, (Cpu::adc, AddressingModes::AbsoluteIndexedX));
        self.instructions.insert(0x79, (Cpu::adc, AddressingModes::AbsoluteIndexedY));
        self.instructions.insert(0x61, (Cpu::adc, AddressingModes::IndexedIndirect));
        self.instructions.insert(0x71, (Cpu::adc, AddressingModes::IndirectIndexed));
        // AND
        self.instructions.insert(0x29, (Cpu::and, AddressingModes::Immediate));
        self.instructions.insert(0x25, (Cpu::and, AddressingModes::ZeroPage));
        self.instructions.insert(0x35, (Cpu::and, AddressingModes::ZeroPageIndexedX));
        self.instructions.insert(0x2d, (Cpu::and, AddressingModes::Absolute));
        self.instructions.insert(0x3d, (Cpu::and, AddressingModes::AbsoluteIndexedX));
        self.instructions.insert(0x39, (Cpu::and, AddressingModes::AbsoluteIndexedY));
        self.instructions.insert(0x21, (Cpu::and, AddressingModes::IndexedIndirect));
        self.instructions.insert(0x31, (Cpu::and, AddressingModes::IndirectIndexed));
        // ASL
        self.instructions.insert(0x0a, (Cpu::asl, AddressingModes::Accumulator));
        self.instructions.insert(0x06, (Cpu::asl, AddressingModes::ZeroPage));
        self.instructions.insert(0x16, (Cpu::asl, AddressingModes::ZeroPageIndexedX));
        self.instructions.insert(0x0e, (Cpu::asl, AddressingModes::Absolute));
        self.instructions.insert(0x1e, (Cpu::asl, AddressingModes::AbsoluteIndexedX));
        // BCC
        self.instructions.insert(0x90, (Cpu::bcc, AddressingModes::Relative));
        // BCS
        self.instructions.insert(0xb0, (Cpu::bcs, AddressingModes::Relative));
        // BEQ
        self.instructions.insert(0xf0, (Cpu::beq, AddressingModes::Relative));
        // BIT
        self.instructions.insert(0x24, (Cpu::bit, AddressingModes::ZeroPage));
        self.instructions.insert(0x2c, (Cpu::bit, AddressingModes::Absolute));
        // BMI
        self.instructions.insert(0x30, (Cpu::bmi, AddressingModes::Relative));
        // BNE
        self.instructions.insert(0xd0, (Cpu::bne, AddressingModes::Relative));
        // BPL
        self.instructions.insert(0x10, (Cpu::bpl, AddressingModes::Relative));
        // BVC
        self.instructions.insert(0x50, (Cpu::bvc, AddressingModes::Relative));
        // BVS
        self.instructions.insert(0x70, (Cpu::bvs, AddressingModes::Relative));
        // BRK
        self.instructions.insert(0x00, (Cpu::brk, AddressingModes::Implicit));
        // CLC
        self.instructions.insert(0x18, (Cpu::clc, AddressingModes::Implicit));
        // CLD
        self.instructions.insert(0xd8, (Cpu::cld, AddressingModes::Implicit));
        // CLI
        self.instructions.insert(0x58, (Cpu::cli, AddressingModes::Implicit));
        // CLV
        self.instructions.insert(0xb8, (Cpu::clv, AddressingModes::Implicit));
        // CMP
        self.instructions.insert(0xc9, (Cpu::cmp, AddressingModes::Immediate));
        self.instructions.insert(0xc5, (Cpu::cmp, AddressingModes::ZeroPage));
        self.instructions.insert(0xd5, (Cpu::cmp, AddressingModes::ZeroPageIndexedX));
        self.instructions.insert(0xcd, (Cpu::cmp, AddressingModes::Absolute));
        self.instructions.insert(0xdd, (Cpu::cmp, AddressingModes::AbsoluteIndexedX));
        self.instructions.insert(0xd9, (Cpu::cmp, AddressingModes::AbsoluteIndexedY));
        self.instructions.insert(0xc1, (Cpu::cmp, AddressingModes::IndexedIndirect));
        self.instructions.insert(0xd1, (Cpu::cmp, AddressingModes::IndirectIndexed));
        // CPX
        self.instructions.insert(0xe0, (Cpu::cpx, AddressingModes::Immediate));
        self.instructions.insert(0xe4, (Cpu::cpx, AddressingModes::ZeroPage));
        self.instructions.insert(0xec, (Cpu::cpx, AddressingModes::Absolute));
        // CPY
        self.instructions.insert(0xc0, (Cpu::cpy, AddressingModes::Immediate));
        self.instructions.insert(0xc4, (Cpu::cpy, AddressingModes::ZeroPage));
        self.instructions.insert(0xcc, (Cpu::cpy, AddressingModes::Absolute));
        // DEC
        self.instructions.insert(0xc6, (Cpu::dec, AddressingModes::ZeroPage));
        self.instructions.insert(0xd6, (Cpu::dec, AddressingModes::ZeroPageIndexedX));
        self.instructions.insert(0xce, (Cpu::dec, AddressingModes::Absolute));
        self.instructions.insert(0xde, (Cpu::dec, AddressingModes::AbsoluteIndexedX));
        // DEX
        self.instructions.insert(0xca, (Cpu::dex, AddressingModes::Implicit));
        // DEY
        self.instructions.insert(0x88, (Cpu::dey, AddressingModes::Implicit));
        // EOR
        self.instructions.insert(0x49, (Cpu::eor, AddressingModes::Immediate));
        self.instructions.insert(0x45, (Cpu::eor, AddressingModes::ZeroPage));
        self.instructions.insert(0x55, (Cpu::eor, AddressingModes::ZeroPageIndexedX));
        self.instructions.insert(0x4d, (Cpu::eor, AddressingModes::Absolute));
        self.instructions.insert(0x5d, (Cpu::eor, AddressingModes::AbsoluteIndexedX));
        self.instructions.insert(0x59, (Cpu::eor, AddressingModes::AbsoluteIndexedY));
        self.instructions.insert(0x41, (Cpu::eor, AddressingModes::IndexedIndirect));
        self.instructions.insert(0x51, (Cpu::eor, AddressingModes::IndirectIndexed));
        // INC
        self.instructions.insert(0xe6, (Cpu::inc, AddressingModes::ZeroPage));
        self.instructions.insert(0xf6, (Cpu::inc, AddressingModes::ZeroPageIndexedX));
        self.instructions.insert(0xee, (Cpu::inc, AddressingModes::Absolute));
        self.instructions.insert(0xfe, (Cpu::inc, AddressingModes::AbsoluteIndexedX));
        // INX
        self.instructions.insert(0xe8, (Cpu::inx, AddressingModes::Implicit));
        // INY
        self.instructions.insert(0xc8, (Cpu::iny, AddressingModes::Implicit));
        // JMP
        self.instructions.insert(0x4c, (Cpu::jmp, AddressingModes::Absolute));
        self.instructions.insert(0x6c, (Cpu::jmp, AddressingModes::Indirect));
        // JSR
        self.instructions.insert(0x20, (Cpu::jsr, AddressingModes::Absolute));
        // LDA
        self.instructions.insert(0xa9, (Cpu::lda, AddressingModes::Immediate));
        self.instructions.insert(0xa5, (Cpu::lda, AddressingModes::ZeroPage));
        self.instructions.insert(0xb5, (Cpu::lda, AddressingModes::ZeroPageIndexedX));
        self.instructions.insert(0xbd, (Cpu::lda, AddressingModes::AbsoluteIndexedX));
        self.instructions.insert(0xb9, (Cpu::lda, AddressingModes::AbsoluteIndexedY));
        self.instructions.insert(0xa1, (Cpu::lda, AddressingModes::IndexedIndirect));
        self.instructions.insert(0xb1, (Cpu::lda, AddressingModes::IndirectIndexed));
        // LDX
        self.instructions.insert(0xa2, (Cpu::ldx, AddressingModes::Immediate));
        self.instructions.insert(0xa6, (Cpu::ldx, AddressingModes::ZeroPage));
        self.instructions.insert(0xb6, (Cpu::ldx, AddressingModes::ZeroPageIndexedY));
        self.instructions.insert(0xae, (Cpu::ldx, AddressingModes::Absolute));
        self.instructions.insert(0xbe, (Cpu::ldx, AddressingModes::AbsoluteIndexedY));
        // LDY
        self.instructions.insert(0xa0, (Cpu::ldy, AddressingModes::Immediate));
        self.instructions.insert(0xa4, (Cpu::ldy, AddressingModes::ZeroPage));
        self.instructions.insert(0xb4, (Cpu::ldy, AddressingModes::ZeroPageIndexedX));
        self.instructions.insert(0xac, (Cpu::ldy, AddressingModes::Absolute));
        self.instructions.insert(0xbc, (Cpu::ldy, AddressingModes::AbsoluteIndexedX));
        // LSR
        self.instructions.insert(0x4a, (Cpu::lsr, AddressingModes::Accumulator));
        self.instructions.insert(0x46, (Cpu::lsr, AddressingModes::ZeroPage));
        self.instructions.insert(0x56, (Cpu::lsr, AddressingModes::ZeroPageIndexedX));
        self.instructions.insert(0x4e, (Cpu::lsr, AddressingModes::Absolute));
        self.instructions.insert(0x5e, (Cpu::lsr, AddressingModes::AbsoluteIndexedX));
        // NOP
        self.instructions.insert(0xea, (Cpu::nop, AddressingModes::Implicit));
        // ORA
        self.instructions.insert(0x09, (Cpu::ora, AddressingModes::Immediate));
        self.instructions.insert(0x05, (Cpu::ora, AddressingModes::ZeroPage));
        self.instructions.insert(0x15, (Cpu::ora, AddressingModes::ZeroPageIndexedX));
        self.instructions.insert(0x0d, (Cpu::ora, AddressingModes::Absolute));
        self.instructions.insert(0x1d, (Cpu::ora, AddressingModes::AbsoluteIndexedX));
        self.instructions.insert(0x19, (Cpu::ora, AddressingModes::AbsoluteIndexedY));
        self.instructions.insert(0x01, (Cpu::ora, AddressingModes::IndexedIndirect));
        self.instructions.insert(0x11, (Cpu::ora, AddressingModes::IndirectIndexed));
        // PHA
        self.instructions.insert(0x48, (Cpu::pha, AddressingModes::Implicit));
        // PHP
        self.instructions.insert(0x08, (Cpu::php, AddressingModes::Implicit));
        // PLA
        self.instructions.insert(0x68, (Cpu::pla, AddressingModes::Implicit));
        // PLP
        self.instructions.insert(0x28, (Cpu::plp, AddressingModes::Implicit));
        // ROL
        self.instructions.insert(0x2a, (Cpu::rol, AddressingModes::Accumulator));
        self.instructions.insert(0x26, (Cpu::rol, AddressingModes::ZeroPage));
        self.instructions.insert(0x36, (Cpu::rol, AddressingModes::ZeroPageIndexedX));
        self.instructions.insert(0x2e, (Cpu::rol, AddressingModes::Absolute));
        self.instructions.insert(0x3e, (Cpu::rol, AddressingModes::AbsoluteIndexedX));
        // ROR
        self.instructions.insert(0x6a, (Cpu::ror, AddressingModes::Accumulator));
        self.instructions.insert(0x66, (Cpu::ror, AddressingModes::ZeroPage));
        self.instructions.insert(0x76, (Cpu::ror, AddressingModes::ZeroPageIndexedX));
        self.instructions.insert(0x6e, (Cpu::ror, AddressingModes::Absolute));
        self.instructions.insert(0x7e, (Cpu::ror, AddressingModes::AbsoluteIndexedX));
        // RTI
        self.instructions.insert(0x40, (Cpu::rti, AddressingModes::Implicit));
        // RTS
        self.instructions.insert(0x60, (Cpu::rts, AddressingModes::Implicit));
        // SBC
        self.instructions.insert(0xe9, (Cpu::sbc, AddressingModes::Immediate));
        self.instructions.insert(0xe5, (Cpu::sbc, AddressingModes::ZeroPage));
        self.instructions.insert(0xf5, (Cpu::sbc, AddressingModes::ZeroPageIndexedX));
        self.instructions.insert(0xed, (Cpu::sbc, AddressingModes::Absolute));
        self.instructions.insert(0xfd, (Cpu::sbc, AddressingModes::AbsoluteIndexedX));
        self.instructions.insert(0xf9, (Cpu::sbc, AddressingModes::AbsoluteIndexedY));
        self.instructions.insert(0xe1, (Cpu::sbc, AddressingModes::IndexedIndirect));
        self.instructions.insert(0xf1, (Cpu::sbc, AddressingModes::IndirectIndexed));
        // SEC
        self.instructions.insert(0x38, (Cpu::sec, AddressingModes::Implicit));
        // SED
        self.instructions.insert(0xf8, (Cpu::sed, AddressingModes::Implicit));
        // SEI
        self.instructions.insert(0x78, (Cpu::sei, AddressingModes::Implicit));
        // STA
        self.instructions.insert(0x85, (Cpu::sta, AddressingModes::ZeroPage));
        self.instructions.insert(0x95, (Cpu::sta, AddressingModes::ZeroPageIndexedX));
        self.instructions.insert(0x8d, (Cpu::sta, AddressingModes::Absolute));
        self.instructions.insert(0x9d, (Cpu::sta, AddressingModes::AbsoluteIndexedX));
        self.instructions.insert(0x99, (Cpu::sta, AddressingModes::AbsoluteIndexedY));
        self.instructions.insert(0x81, (Cpu::sta, AddressingModes::IndexedIndirect));
        self.instructions.insert(0x91, (Cpu::sta, AddressingModes::IndirectIndexed));
        // STX
        self.instructions.insert(0x86, (Cpu::stx, AddressingModes::ZeroPage));
        self.instructions.insert(0x96, (Cpu::stx, AddressingModes::ZeroPageIndexedY));
        self.instructions.insert(0x8e, (Cpu::stx, AddressingModes::Absolute));
        // STY
        self.instructions.insert(0x84, (Cpu::sty, AddressingModes::ZeroPage));
        self.instructions.insert(0x94, (Cpu::sty, AddressingModes::ZeroPageIndexedX));
        self.instructions.insert(0x8c, (Cpu::sty, AddressingModes::Absolute));
        // TAX
        self.instructions.insert(0xaa, (Cpu::tax, AddressingModes::Implicit));
        // TAY
        self.instructions.insert(0xa8, (Cpu::tay, AddressingModes::Implicit));
        // TSX
        self.instructions.insert(0xba, (Cpu::tsx, AddressingModes::Implicit));
        // TXA
        self.instructions.insert(0x8a, (Cpu::txa, AddressingModes::Implicit));
        // TXS
        self.instructions.insert(0x9a, (Cpu::txs, AddressingModes::Implicit));
        // TYA
        self.instructions.insert(0x98, (Cpu::tya, AddressingModes::Implicit));
    }

    pub fn load_rom(&mut self, file_path: &str) -> Result<(), Error> {
        let contents: Vec<u8> = fs::read(file_path)?;

        if contents[0..4] != [0x4e, 0x45, 0x53, 0x1a] {
            return Err(
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Arquivo NES inválido: cabeçalho incorreto"
                )
            );
        }

        let prg_rom_size: usize = (contents[4] as usize) * 16 * 1024;
        let _chr_rom_size: usize = (contents[5] as usize) * 8 * 1024;
        let has_trainer: bool = (contents[6] & 0b0000_0100) != 0;

        let prg_rom_start: usize = 16 + (if has_trainer { 512 } else { 0 });
        let prg_rom_data: &[u8] = &contents[prg_rom_start..prg_rom_start + prg_rom_size];

        for (i, byte) in prg_rom_data.iter().enumerate() {
            if 0x8000 + i < 65536 {
                self.memory[0x8000 + i] = *byte;
            } else {
                break;
            }
        }

        self.registers.program_counter = self.turn_in_u16(self.memory[0xfffc], self.memory[0xfffd]);

        Ok(())
    }

    fn fetch(&mut self) -> u8 {
        let opcode: u8 = self.memory[self.registers.program_counter as usize];
        self.registers.program_counter = self.registers.program_counter.wrapping_add(1);
        opcode
    }

    fn decode(&mut self) -> Option<OpcodeFunction> {
        let opcode: u8 = self.fetch();
        println!("Fetched Opcode: 0x{:X} - PC: {}", opcode, self.registers.program_counter);
        self.instructions.get(&opcode).copied()
    }

    pub fn execute(&mut self) {
        let op_function: Option<OpcodeFunction> = self.decode();
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
            AddressingModes::Immediate => {
                let value: u8 = self.fetch();
                value
            }
            AddressingModes::ZeroPage => {
                let address: u8 = self.fetch();
                self.memory[address as usize]
            }
            AddressingModes::ZeroPageIndexedX => {
                let address: u8 = self.fetch();
                let indexed_address: u8 = address.wrapping_add(self.registers.index_x);

                self.memory[indexed_address as usize]
            }
            AddressingModes::ZeroPageIndexedY => {
                let address: u8 = self.fetch();
                let indexed_address: u8 = address.wrapping_add(self.registers.index_y);

                self.memory[indexed_address as usize]
            }
            AddressingModes::Absolute => {
                let low: u8 = self.fetch();
                let high: u8 = self.fetch();
                let address: u16 = self.turn_in_u16(low, high);

                self.memory[address as usize]
            }
            AddressingModes::AbsoluteIndexedX => {
                let low: u8 = self.fetch();
                let high: u8 = self.fetch();
                let address: u16 = self.turn_in_u16(low, high);
                let indexed_address: u16 = address.wrapping_add(self.registers.index_x as u16);

                self.memory[indexed_address as usize]
            }
            AddressingModes::AbsoluteIndexedY => {
                let low: u8 = self.fetch();
                let high: u8 = self.fetch();
                let address: u16 = self.turn_in_u16(low, high);
                let indexed_address: u16 = address.wrapping_add(self.registers.index_y as u16);

                self.memory[indexed_address as usize]
            }
            AddressingModes::IndexedIndirect => {
                let address: u8 = self.fetch();
                let indexed_address: u8 = address.wrapping_add(self.registers.index_x);

                let low: u8 = self.memory[indexed_address as usize];
                let high: u8 = self.memory[indexed_address.wrapping_add(1) as usize];
                let indirect_address: u16 = self.turn_in_u16(low, high);

                self.memory[indirect_address as usize]
            }
            AddressingModes::IndirectIndexed => {
                let indirect_base_address: u8 = self.fetch();

                let low: u8 = self.memory[indirect_base_address as usize];
                let high: u8 = self.memory[indirect_base_address.wrapping_add(1) as usize];
                let indirect_address: u16 = self.turn_in_u16(low, high);

                let indexed_address: u16 = indirect_address.wrapping_add(
                    self.registers.index_y as u16
                );

                self.memory[indexed_address as usize]
            }
            AddressingModes::Indirect => {
                let low_ptr: u8 = self.fetch();
                let high_ptr: u8 = self.fetch();
                let ptr: u16 = self.turn_in_u16(low_ptr, high_ptr);

                let low = self.memory[ptr as usize];
                let high = if (ptr & 0x00ff) == 0x00ff {
                    self.memory[(ptr & 0xff00) as usize]
                } else {
                    self.memory[(ptr + 1) as usize]
                };
                let effective_address = self.turn_in_u16(low, high);

                self.memory[effective_address as usize]
            }
            AddressingModes::Relative => {
                let value = self.fetch() as i8;
                let address = self.registers.program_counter.wrapping_add(value as u16);
                self.memory[address as usize]
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
                let base = self.fetch();
                let ptr = base.wrapping_add(self.registers.index_x);
                let low = self.memory[ptr as usize];
                let high = self.memory[ptr.wrapping_add(1) as usize];
                self.turn_in_u16(low, high)
            }
            AddressingModes::IndirectIndexed => {
                let base = self.fetch();
                let low = self.memory[base as usize];
                let high = self.memory[base.wrapping_add(1) as usize];
                let addr = self.turn_in_u16(low, high);
                addr.wrapping_add(self.registers.index_y as u16)
            }
            AddressingModes::Indirect => {
                let low = self.fetch();
                let high = self.fetch();
                let addr = self.turn_in_u16(low, high);

                // Simula o bug de hardware do 6502 quando o endereço está em limite de página
                let low_addr = if (addr & 0x00ff) == 0x00ff {
                    self.memory[addr as usize]
                } else {
                    self.memory[addr as usize]
                };

                let high_addr = if (addr & 0x00ff) == 0x00ff {
                    self.memory[(addr & 0xff00) as usize]
                } else {
                    self.memory[(addr + 1) as usize]
                };

                self.turn_in_u16(low_addr, high_addr)
            }
            AddressingModes::Relative => {
                let offset = self.fetch() as i8;
                let addr = self.registers.program_counter.wrapping_add(offset as u16);
                addr
            }
            AddressingModes::Accumulator => {
                0 // Valor especial para indicar que a operação é no acumulador
            }
            AddressingModes::Implicit => {
                0 // Valor especial para instruções implícitas
            }
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

