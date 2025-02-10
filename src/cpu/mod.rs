pub mod cpu {
    use std::collections::HashMap;

    // Flags 0b NV1B DIZC
    const CARRY: u8 = 0b0000_0001;
    const ZERO: u8 = 0b0000_0010;
    const INTERRUPT_DISABLE: u8 = 0b0000_0100;
    const DECIMAL: u8 = 0b0000_1000;
    const BREAK: u8 = 0b0001_0000;
    const UNUSED: u8 = 0b0010_0000;
    const OVERFLOW: u8 = 0b0100_0000;
    const NEGATIVE: u8 = 0b1000_0000;

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
        fn bytes(&self) -> u8 {
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

    struct Registers {
        acc: u8, //A
        index_x: u8, //X
        index_y: u8, //Y
        stack_pointer: u8, //S
        status_register: u8, //P -> NV1B DIZC (Flags)
        program_counter: u16, //PC
    }

    type Opcode = u8;
    type OpcodeFunction = (fn(&mut Cpu, AddressingModes), AddressingModes);
    type InstructionMap = HashMap<Opcode, OpcodeFunction>; // Opcode, Instrution

    pub struct Cpu {
        registers: Registers,
        instructions: InstructionMap,
        memory: [u8; 65536],
    }

    impl Cpu {
        fn new() -> Self {
            Cpu {
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
            }
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

        fn fetch(&mut self) -> u8 {
            let opcode = self.memory[self.registers.program_counter as usize];
            self.registers.program_counter = self.registers.program_counter.wrapping_add(1);
            opcode
        }

        fn decode(&mut self) {
            let opcode = self.fetch();
        }

        fn execute(&self, _opcode: u8) {}

        fn execute_mode(&self, mode: AddressingModes, value: u16) -> u8 {
            match mode {
                AddressingModes::Immediate => value as u8,
                AddressingModes::ZeroPage => self.memory[value as usize],
                AddressingModes::ZeroPageIndexedX => {
                    let val = (value as u8).wrapping_add(self.registers.index_x);
                    self.memory[val as usize]
                }
                AddressingModes::ZeroPageIndexedY => {
                    let val = (value as u8).wrapping_add(self.registers.index_y);
                    self.memory[val as usize]
                }
                AddressingModes::Absolute => self.memory[value as usize],
                AddressingModes::AbsoluteIndexedX => {
                    let val = value.wrapping_add(self.registers.index_x as u16);
                    self.memory[val as usize]
                }
                AddressingModes::AbsoluteIndexedY => {
                    let val = value.wrapping_add(self.registers.index_y as u16);
                    self.memory[val as usize]
                }
                // Corrigir com wrapping_add
                AddressingModes::IndexedIndirect => {
                    let val_1 = ((value as u8) + self.registers.index_x) % 255;
                    let peek_1 = self.memory[val_1 as usize];

                    let val_2 = ((value as u8) + self.registers.index_x + 1) % 255;
                    let peek_2 = self.memory[val_2 as usize];

                    let val = (peek_1 + peek_2) * 255;

                    self.memory[val as usize]
                }
                AddressingModes::IndirectIndexed => {
                    let peek_1 = self.memory[value as usize];

                    let val_1 = (((value as u8) + 1) % 255) * 255 + self.registers.index_y;
                    let peek_2 = self.memory[val_1 as usize];

                    let val = peek_1 + peek_2;
                    self.memory[val as usize]
                }
                _ => 0,
            }
        }
    }

    impl Cpu {
        // Access
        fn lda(&mut self, mode: AddressingModes) {
            let value = 0;
            /*
               Esse value tem que vir da memoria
               LDA #$45
               <LDA #$> <45>

               <LDA #$> -> OPCODE XYZ -> Função + Modo
               <45> -> OPERANDO ; O modo define como esse operando vai ser tratado

               Ex:
               Byte 1 = self.memory[0] -> Tem o Opcode = Função + Modo
               Byte 2 = self.memory[1] -> Tem o Operando
               Byte 3 = self.memory[2] -> Pode ser o proximo opcode ou Mais um operando
            */

            let value_for_acc = self.execute_mode(mode, value);
            self.registers.acc = value_for_acc;

            /* Tem que melhorar esses sets de flags */
            if value_for_acc == 0 {
                self.registers.status_register |= ZERO;
            } else {
                self.registers.status_register &= !ZERO;
            }

            if (value_for_acc & NEGATIVE) == 0 {
                self.registers.status_register |= NEGATIVE;
            } else {
                self.registers.status_register &= !NEGATIVE;
            }
        }

        fn sta(&mut self, mode: AddressingModes) {
            let value = 0;
            let value_for_memory = self.execute_mode(mode, value);
            self.memory[value_for_memory as usize] = self.registers.acc;
        }

        fn ldx(&mut self, mode: AddressingModes) {
            let value = 0;
            let value_for_x = self.execute_mode(mode, value);

            self.registers.index_x = value_for_x;
            // Zero e Negative
        }

        fn stx(&mut self, mode: AddressingModes) {
            let value = 0;
            let value_for_memory = self.execute_mode(mode, value);
            self.memory[value_for_memory as usize] = self.registers.index_x;
        }

        fn ldy(&mut self, mode: AddressingModes) {
            let value = 0;
            let value_for_y = self.execute_mode(mode, value);
            self.registers.index_y = value_for_y;

            // Zero e Negative
        }

        fn sty(&mut self, mode: AddressingModes) {
            let value = 0;
            let value_for_memory = self.execute_mode(mode, value);
            self.memory[value_for_memory as usize] = value_for_memory;
        }

        // Transfer
        fn tax(&mut self, mode: AddressingModes) {
            self.registers.index_x = self.registers.acc;
            // flags zero e negative
        }
        fn txa(&mut self, mode: AddressingModes) {
            self.registers.acc = self.registers.index_x;
            // flags zero e negative
        }
        fn tay(&mut self, mode: AddressingModes) {
            self.registers.index_y = self.registers.acc;
            //flags zero negative
        }
        fn tya(&mut self, mode: AddressingModes) {
            self.registers.acc = self.registers.index_y;
            //flags zero negative
        }

        // Arithmetic
        fn adc(&mut self, mode: AddressingModes) {}
        fn sbc(&mut self, mode: AddressingModes) {}
        fn inc(&mut self, mode: AddressingModes) {}
        fn dec(&mut self, mode: AddressingModes) {}
        fn inx(&mut self, mode: AddressingModes) {}
        fn dex(&mut self, mode: AddressingModes) {}
        fn iny(&mut self, mode: AddressingModes) {}
        fn dey(&mut self, mode: AddressingModes) {}

        // Shift
        fn asl(&mut self, mode: AddressingModes) {}
        fn lsr(&mut self, mode: AddressingModes) {}
        fn rol(&mut self, mode: AddressingModes) {}
        fn ror(&mut self, mode: AddressingModes) {}

        // Bitwise
        fn and(&mut self, mode: AddressingModes) {}
        fn ora(&mut self, mode: AddressingModes) {}
        fn eor(&mut self, mode: AddressingModes) {}
        fn bit(&mut self, mode: AddressingModes) {}

        // Compare
        fn cmp(&mut self, mode: AddressingModes) {}
        fn cpx(&mut self, mode: AddressingModes) {}
        fn cpy(&mut self, mode: AddressingModes) {}

        // Branch
        fn bcc(&mut self, mode: AddressingModes) {}
        fn bcs(&mut self, mode: AddressingModes) {}
        fn beq(&mut self, mode: AddressingModes) {}
        fn bne(&mut self, mode: AddressingModes) {}
        fn bpl(&mut self, mode: AddressingModes) {}
        fn bmi(&mut self, mode: AddressingModes) {}
        fn bvc(&mut self, mode: AddressingModes) {}
        fn bvs(&mut self, mode: AddressingModes) {}

        // Jump
        fn jmp(&mut self, mode: AddressingModes) {}
        fn jsr(&mut self, mode: AddressingModes) {}
        fn rts(&mut self, mode: AddressingModes) {}
        fn brk(&mut self, mode: AddressingModes) {}
        fn rti(&mut self, mode: AddressingModes) {}

        // Stack
        fn pha(&mut self, mode: AddressingModes) {}
        fn pla(&mut self, mode: AddressingModes) {}
        fn php(&mut self, mode: AddressingModes) {}
        fn plp(&mut self, mode: AddressingModes) {}
        fn txs(&mut self, mode: AddressingModes) {}
        fn tsx(&mut self, mode: AddressingModes) {}

        // Flags
        fn clc(&mut self, mode: AddressingModes) {
            // Clear Carry
            self.registers.status_register |= !CARRY;
        }
        fn sec(&mut self, mode: AddressingModes) {
            // Set Carry
            self.registers.status_register ^= CARRY;
        }
        fn cli(&mut self, mode: AddressingModes) {
            // Clear Interrupt Disable
        }
        fn sei(&mut self, mode: AddressingModes) {
            // Set Interrupt Disable
        }
        fn cld(&mut self, mode: AddressingModes) {
            // Clear Decimal
        }
        fn sed(&mut self, mode: AddressingModes) {
            // Set Decimal
        }
        fn clv(&mut self, mode: AddressingModes) {
            //Clear Overflow
        }

        // Other
        fn nop(&mut self, mode: AddressingModes) {}
    }
}
