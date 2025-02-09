pub mod cpu {
    use std::ops::Add;

    // Flags
    const CARRY: u8             = 0b0000_0001;
    const ZERO: u8              = 0b0000_0010;
    const INTERRUPT_DISABLE: u8 = 0b0000_0100;
    const DECIMAL: u8           = 0b0000_1000;
    const BREAK: u8             = 0b0001_0000;
    const UNUSED: u8            = 0b0010_0000;
    const OVERFLOW: u8          = 0b0100_0000;
    const NEGATIVE: u8          = 0b1000_0000;

    pub enum AddressingModes {
        ZeroPageIndexedX,   // D,x
        ZeroPageIndexedY,   // D,x
        AbsoluteIndexedX,   // A,x
        AbsoluteIndexedY,   // A,x
        IndexedIndirect,    // (d,x)
        IndirectIndexed,    // (d),y
        
        //Others
        Implicit,
        Accumulator,
        Immediate,
        ZeroPage,
        Absolute,
        Relative,
        Indirect,    
    } 

    struct Instruction {
        opcode: u8,
        mode: AddressingModes,
    }

    struct Registers {
        acc: u8,             //A
        index_x: u8,         //X
        index_y: u8,         //Y
        stack_pointer: u8,   //S
        status_register: u8, //P -> NV1B DIZC (Flags)
        program_couter: u16, //PC
    }

    pub struct Cpu {
        registers: Registers,
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
                    program_couter: 0,
                },
                memory: [0;65536]

            }
        }

        fn fetch() {}

        // Lembra de Baixar a tebela de memoria 
        fn decode(opcode: u8) {
            match opcode {
                0x00 => (),
                0x01 => (),
                _ => (),
            }
        }

        fn execute_mode(&self, mode: AddressingModes, value: u16) -> u8 {
            match mode { 
                AddressingModes::Immediate => {
                    value as u8
                },
                AddressingModes::ZeroPage => {
                    self.memory[value as usize]
                },
                AddressingModes::ZeroPageIndexedX => {
                    let val = ((value as u8) + self.registers.index_x) % 255;
                    self.memory[val as usize]
                },
                AddressingModes::ZeroPageIndexedY => {
                    let val = ((value as u8) + self.registers.index_y) % 255;
                    self.memory[val as usize]
                },
                AddressingModes::Absolute => {
                    self.memory[value as usize]
                },
                AddressingModes::AbsoluteIndexedX => {
                    let val = (value as u8) + self.registers.index_x;
                    self.memory[val as usize]
                },
                AddressingModes::AbsoluteIndexedY => {
                    let val = (value as u8) + self.registers.index_y;
                    self.memory[val as usize]
                },
                AddressingModes::IndexedIndirect => {
                    let val_1 = ((value as u8) + self.registers.index_x) % 255;
                    let peek_1 = self.memory[val_1 as usize];

                    let val_2 = ((value as u8) + self.registers.index_x + 1) % 255;
                    let peek_2 = self.memory[val_2 as usize];

                    let val = (peek_1 + peek_2)*255;

                    self.memory[val as usize]
                },
                AddressingModes::IndirectIndexed => {
                    let peek_1 = self.memory[value as usize];

                    let val_1 = (((value as u8) + 1) % 255) * 255 + self.registers.index_y;
                    let peek_2 = self.memory[val_1 as usize];

                    let val = peek_1 + peek_2;
                    self.memory[val as usize] 
                }
                _ => {0}
            }
        }
    }
    
    impl Cpu {
        // Access
        fn lda(&mut self, value: u16, mode: AddressingModes) {
            let value_for_acc = self.execute_mode(mode, value); 
            self.registers.acc = value_for_acc;
            
            /* Tem que melhorar esses sets de flags */
            if value_for_acc == 0 {
                self.registers.status_register |= ZERO;
            } else {
                self.registers.status_register &= !ZERO;
            }

            if value_for_acc & NEGATIVE == 0 {
                self.registers.status_register |= NEGATIVE;
            } else {
                self.registers.status_register &= !NEGATIVE;
            }

       }

        fn sta(&mut self, value: u16, mode:AddressingModes) {
            let value_for_memory = self.execute_mode(mode, value);

            self.memory[value_for_memory as usize] = self.registers.acc;
        }

        fn ldx(&mut self, value: u16, mode: AddressingModes) {
            let value_for_x = self.execute_mode(mode, value);
         
            self.registers.index_x = value_for_x;
            // Zero e Negative
        }

        fn stx(&mut self, value: u16, mode: AddressingModes) {
            let value_for_memory = self.execute_mode(mode, value);

            self.memory[value_for_memory as usize] = self.registers.index_x;
        }
        
        fn ldy(&mut self, value: u16, mode: AddressingModes) {
            let value_for_y = self.execute_mode(mode, value);

            self.registers.index_y = value_for_y;
            // Zero e Negative
        }

        fn sty(&mut self, value: u16, mode: AddressingModes) {
            let value_for_memory = self.execute_mode(mode, value);
            self.memory[value_for_memory as usize] = value_for_memory;
        }

        // Transfer
        fn tax(&mut self) {
            self.registers.index_x = self.registers.acc;   
            // flags zero e negative
        }
        fn txa(&mut self) {
            self.registers.acc = self.registers.index_x;
            // flags zero e negative
        }
        fn tay(&mut self) {
            self.registers.index_y = self.registers.acc;
            //flags zero negative
        }
        fn tya(&mut self) {
            self.registers.acc = self.registers.index_y;
            //flags zero negative
        }

        // Arithmetic
        fn adc() {}
        fn sbc() {}
        fn inc() {}
        fn dec() {}
        fn inx() {}
        fn dex() {}
        fn iny() {}
        fn dey() {}

        // Shift
        fn asl() {}
        fn lsr() {}
        fn rol() {}
        fn ror() {}

        // Bitwise
        fn and() {}
        fn ora() {}
        fn eor() {}
        fn bit() {}

        // Compare
        fn cmp() {}
        fn cpx() {}
        fn cpy() {}

        // Branch
        fn bcc() {}
        fn bcs() {}
        fn beq() {}
        fn bne() {}
        fn bpl() {}
        fn bmi() {}
        fn bvc() {}
        fn bvs() {}

        // Jump
        fn jmp() {}
        fn jsr() {}
        fn rts() {}
        fn brk() {}
        fn rti() {}

        // Stack
        fn pha() {}
        fn pla() {}
        fn php() {}
        fn plp() {}
        fn txs() {}
        fn tsx() {}

        // Flags
        fn clc(&mut self) { 
            // Clear Carry
            self.registers.status_register |= !CARRY;
        }
        fn sec(&mut self) {
            // Set Carry 
            self.registers.status_register ^= CARRY;
        }
        fn cli(&mut self) {
            // Clear Interrupt Disable
        }
        fn sei(&mut self) {
            // Set Interrupt Disable
        }
        fn cld(&mut self) {
            // Clear Decimal
        }
        fn sed(&mut self) {
            // Set Decimal
        }
        fn clv(&mut self) {
            //Clear Overflow
        }

        // Other
        fn nop() {}
    }
}
