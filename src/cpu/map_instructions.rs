impl CPU {
    fn map_instructions(&mut self) {
        // ADC
        self.instructions.insert(0x69, (CPU::adc, AddressingModes::Immediate));
        self.instructions.insert(0x65, (CPU::adc, AddressingModes::ZeroPage));
        self.instructions.insert(0x75, (CPU::adc, AddressingModes::ZeroPageIndexedX));
        self.instructions.insert(0x6d, (CPU::adc, AddressingModes::Absolute));
        self.instructions.insert(0x7d, (CPU::adc, AddressingModes::AbsoluteIndexedX));
        self.instructions.insert(0x79, (CPU::adc, AddressingModes::AbsoluteIndexedY));
        self.instructions.insert(0x61, (CPU::adc, AddressingModes::IndexedIndirect));
        self.instructions.insert(0x71, (CPU::adc, AddressingModes::IndirectIndexed));
        // AND
        self.instructions.insert(0x29, (CPU::and, AddressingModes::Immediate));
        self.instructions.insert(0x25, (CPU::and, AddressingModes::ZeroPage));
        self.instructions.insert(0x35, (CPU::and, AddressingModes::ZeroPageIndexedX));
        self.instructions.insert(0x2d, (CPU::and, AddressingModes::Absolute));
        self.instructions.insert(0x3d, (CPU::and, AddressingModes::AbsoluteIndexedX));
        self.instructions.insert(0x39, (CPU::and, AddressingModes::AbsoluteIndexedY));
        self.instructions.insert(0x21, (CPU::and, AddressingModes::IndexedIndirect));
        self.instructions.insert(0x31, (CPU::and, AddressingModes::IndirectIndexed));
        // ASL
        self.instructions.insert(0x0a, (CPU::asl, AddressingModes::Accumulator));
        self.instructions.insert(0x06, (CPU::asl, AddressingModes::ZeroPage));
        self.instructions.insert(0x16, (CPU::asl, AddressingModes::ZeroPageIndexedX));
        self.instructions.insert(0x0e, (CPU::asl, AddressingModes::Absolute));
        self.instructions.insert(0x1e, (CPU::asl, AddressingModes::AbsoluteIndexedX));
        // BCC
        self.instructions.insert(0x90, (CPU::bcc, AddressingModes::Relative));
        // BCS
        self.instructions.insert(0xb0, (CPU::bcs, AddressingModes::Relative));
        // BEQ
        self.instructions.insert(0xf0, (CPU::beq, AddressingModes::Relative));
        // BIT
        self.instructions.insert(0x24, (CPU::bit, AddressingModes::ZeroPage));
        self.instructions.insert(0x2c, (CPU::bit, AddressingModes::Absolute));
        // BMI
        self.instructions.insert(0x30, (CPU::bmi, AddressingModes::Relative));
        // BNE
        self.instructions.insert(0xd0, (CPU::bne, AddressingModes::Relative));
        // BPL
        self.instructions.insert(0x10, (CPU::bpl, AddressingModes::Relative));
        // BVC
        self.instructions.insert(0x50, (CPU::bvc, AddressingModes::Relative));
        // BVS
        self.instructions.insert(0x70, (CPU::bvs, AddressingModes::Relative));
        // BRK
        self.instructions.insert(0x00, (CPU::brk, AddressingModes::Implicit));
        // CLC
        self.instructions.insert(0x18, (CPU::clc, AddressingModes::Implicit));
        // CLD
        self.instructions.insert(0xd8, (CPU::cld, AddressingModes::Implicit));
        // CLI
        self.instructions.insert(0x58, (CPU::cli, AddressingModes::Implicit));
        // CLV
        self.instructions.insert(0xb8, (CPU::clv, AddressingModes::Implicit));
        // CMP
        self.instructions.insert(0xc9, (CPU::cmp, AddressingModes::Immediate));
        self.instructions.insert(0xc5, (CPU::cmp, AddressingModes::ZeroPage));
        self.instructions.insert(0xd5, (CPU::cmp, AddressingModes::ZeroPageIndexedX));
        self.instructions.insert(0xcd, (CPU::cmp, AddressingModes::Absolute));
        self.instructions.insert(0xdd, (CPU::cmp, AddressingModes::AbsoluteIndexedX));
        self.instructions.insert(0xd9, (CPU::cmp, AddressingModes::AbsoluteIndexedY));
        self.instructions.insert(0xc1, (CPU::cmp, AddressingModes::IndexedIndirect));
        self.instructions.insert(0xd1, (CPU::cmp, AddressingModes::IndirectIndexed));
        // CPX
        self.instructions.insert(0xe0, (CPU::cpx, AddressingModes::Immediate));
        self.instructions.insert(0xe4, (CPU::cpx, AddressingModes::ZeroPage));
        self.instructions.insert(0xec, (CPU::cpx, AddressingModes::Absolute));
        // CPY
        self.instructions.insert(0xc0, (CPU::cpy, AddressingModes::Immediate));
        self.instructions.insert(0xc4, (CPU::cpy, AddressingModes::ZeroPage));
        self.instructions.insert(0xcc, (CPU::cpy, AddressingModes::Absolute));
        // DEC
        self.instructions.insert(0xc6, (CPU::dec, AddressingModes::ZeroPage));
        self.instructions.insert(0xd6, (CPU::dec, AddressingModes::ZeroPageIndexedX));
        self.instructions.insert(0xce, (CPU::dec, AddressingModes::Absolute));
        self.instructions.insert(0xde, (CPU::dec, AddressingModes::AbsoluteIndexedX));
        // DEX
        self.instructions.insert(0xca, (CPU::dex, AddressingModes::Implicit));
        // DEY
        self.instructions.insert(0x88, (CPU::dey, AddressingModes::Implicit));
        // EOR
        self.instructions.insert(0x49, (CPU::eor, AddressingModes::Immediate));
        self.instructions.insert(0x45, (CPU::eor, AddressingModes::ZeroPage));
        self.instructions.insert(0x55, (CPU::eor, AddressingModes::ZeroPageIndexedX));
        self.instructions.insert(0x4d, (CPU::eor, AddressingModes::Absolute));
        self.instructions.insert(0x5d, (CPU::eor, AddressingModes::AbsoluteIndexedX));
        self.instructions.insert(0x59, (CPU::eor, AddressingModes::AbsoluteIndexedY));
        self.instructions.insert(0x41, (CPU::eor, AddressingModes::IndexedIndirect));
        self.instructions.insert(0x51, (CPU::eor, AddressingModes::IndirectIndexed));
        // INC
        self.instructions.insert(0xe6, (CPU::inc, AddressingModes::ZeroPage));
        self.instructions.insert(0xf6, (CPU::inc, AddressingModes::ZeroPageIndexedX));
        self.instructions.insert(0xee, (CPU::inc, AddressingModes::Absolute));
        self.instructions.insert(0xfe, (CPU::inc, AddressingModes::AbsoluteIndexedX));
        // INX
        self.instructions.insert(0xe8, (CPU::inx, AddressingModes::Implicit));
        // INY
        self.instructions.insert(0xc8, (CPU::iny, AddressingModes::Implicit));
        // JMP
        self.instructions.insert(0x4c, (CPU::jmp, AddressingModes::Absolute));
        self.instructions.insert(0x6c, (CPU::jmp, AddressingModes::Indirect));
        // JSR
        self.instructions.insert(0x20, (CPU::jsr, AddressingModes::Absolute));
        // LDA
        self.instructions.insert(0xa9, (CPU::lda, AddressingModes::Immediate));
        self.instructions.insert(0xa5, (CPU::lda, AddressingModes::ZeroPage));
        self.instructions.insert(0xb5, (CPU::lda, AddressingModes::ZeroPageIndexedX));
        self.instructions.insert(0xad, (CPU::lda, AddressingModes::Absolute));
        self.instructions.insert(0xbd, (CPU::lda, AddressingModes::AbsoluteIndexedX));
        self.instructions.insert(0xb9, (CPU::lda, AddressingModes::AbsoluteIndexedY));
        self.instructions.insert(0xa1, (CPU::lda, AddressingModes::IndexedIndirect));
        self.instructions.insert(0xb1, (CPU::lda, AddressingModes::IndirectIndexed));
        // LDX
        self.instructions.insert(0xa2, (CPU::ldx, AddressingModes::Immediate));
        self.instructions.insert(0xa6, (CPU::ldx, AddressingModes::ZeroPage));
        self.instructions.insert(0xb6, (CPU::ldx, AddressingModes::ZeroPageIndexedY));
        self.instructions.insert(0xae, (CPU::ldx, AddressingModes::Absolute));
        self.instructions.insert(0xbe, (CPU::ldx, AddressingModes::AbsoluteIndexedY));
        // LDY
        self.instructions.insert(0xa0, (CPU::ldy, AddressingModes::Immediate));
        self.instructions.insert(0xa4, (CPU::ldy, AddressingModes::ZeroPage));
        self.instructions.insert(0xb4, (CPU::ldy, AddressingModes::ZeroPageIndexedX));
        self.instructions.insert(0xac, (CPU::ldy, AddressingModes::Absolute));
        self.instructions.insert(0xbc, (CPU::ldy, AddressingModes::AbsoluteIndexedX));
        // LSR
        self.instructions.insert(0x4a, (CPU::lsr, AddressingModes::Accumulator));
        self.instructions.insert(0x46, (CPU::lsr, AddressingModes::ZeroPage));
        self.instructions.insert(0x56, (CPU::lsr, AddressingModes::ZeroPageIndexedX));
        self.instructions.insert(0x4e, (CPU::lsr, AddressingModes::Absolute));
        self.instructions.insert(0x5e, (CPU::lsr, AddressingModes::AbsoluteIndexedX));
        // NOP
        self.instructions.insert(0xea, (CPU::nop, AddressingModes::Implicit));
        // ORA
        self.instructions.insert(0x09, (CPU::ora, AddressingModes::Immediate));
        self.instructions.insert(0x05, (CPU::ora, AddressingModes::ZeroPage));
        self.instructions.insert(0x15, (CPU::ora, AddressingModes::ZeroPageIndexedX));
        self.instructions.insert(0x0d, (CPU::ora, AddressingModes::Absolute));
        self.instructions.insert(0x1d, (CPU::ora, AddressingModes::AbsoluteIndexedX));
        self.instructions.insert(0x19, (CPU::ora, AddressingModes::AbsoluteIndexedY));
        self.instructions.insert(0x01, (CPU::ora, AddressingModes::IndexedIndirect));
        self.instructions.insert(0x11, (CPU::ora, AddressingModes::IndirectIndexed));
        // PHA
        self.instructions.insert(0x48, (CPU::pha, AddressingModes::Implicit));
        // PHP
        self.instructions.insert(0x08, (CPU::php, AddressingModes::Implicit));
        // PLA
        self.instructions.insert(0x68, (CPU::pla, AddressingModes::Implicit));
        // PLP
        self.instructions.insert(0x28, (CPU::plp, AddressingModes::Implicit));
        // ROL
        self.instructions.insert(0x2a, (CPU::rol, AddressingModes::Accumulator));
        self.instructions.insert(0x26, (CPU::rol, AddressingModes::ZeroPage));
        self.instructions.insert(0x36, (CPU::rol, AddressingModes::ZeroPageIndexedX));
        self.instructions.insert(0x2e, (CPU::rol, AddressingModes::Absolute));
        self.instructions.insert(0x3e, (CPU::rol, AddressingModes::AbsoluteIndexedX));
        // ROR
        self.instructions.insert(0x6a, (CPU::ror, AddressingModes::Accumulator));
        self.instructions.insert(0x66, (CPU::ror, AddressingModes::ZeroPage));
        self.instructions.insert(0x76, (CPU::ror, AddressingModes::ZeroPageIndexedX));
        self.instructions.insert(0x6e, (CPU::ror, AddressingModes::Absolute));
        self.instructions.insert(0x7e, (CPU::ror, AddressingModes::AbsoluteIndexedX));
        // RTI
        self.instructions.insert(0x40, (CPU::rti, AddressingModes::Implicit));
        // RTS
        self.instructions.insert(0x60, (CPU::rts, AddressingModes::Implicit));
        // SBC
        self.instructions.insert(0xe9, (CPU::sbc, AddressingModes::Immediate));
        self.instructions.insert(0xe5, (CPU::sbc, AddressingModes::ZeroPage));
        self.instructions.insert(0xf5, (CPU::sbc, AddressingModes::ZeroPageIndexedX));
        self.instructions.insert(0xed, (CPU::sbc, AddressingModes::Absolute));
        self.instructions.insert(0xfd, (CPU::sbc, AddressingModes::AbsoluteIndexedX));
        self.instructions.insert(0xf9, (CPU::sbc, AddressingModes::AbsoluteIndexedY));
        self.instructions.insert(0xe1, (CPU::sbc, AddressingModes::IndexedIndirect));
        self.instructions.insert(0xf1, (CPU::sbc, AddressingModes::IndirectIndexed));
        // SEC
        self.instructions.insert(0x38, (CPU::sec, AddressingModes::Implicit));
        // SED
        self.instructions.insert(0xf8, (CPU::sed, AddressingModes::Implicit));
        // SEI
        self.instructions.insert(0x78, (CPU::sei, AddressingModes::Implicit));
        // STA
        self.instructions.insert(0x85, (CPU::sta, AddressingModes::ZeroPage));
        self.instructions.insert(0x95, (CPU::sta, AddressingModes::ZeroPageIndexedX));
        self.instructions.insert(0x8d, (CPU::sta, AddressingModes::Absolute));
        self.instructions.insert(0x9d, (CPU::sta, AddressingModes::AbsoluteIndexedX));
        self.instructions.insert(0x99, (CPU::sta, AddressingModes::AbsoluteIndexedY));
        self.instructions.insert(0x81, (CPU::sta, AddressingModes::IndexedIndirect));
        self.instructions.insert(0x91, (CPU::sta, AddressingModes::IndirectIndexed));
        // STX
        self.instructions.insert(0x86, (CPU::stx, AddressingModes::ZeroPage));
        self.instructions.insert(0x96, (CPU::stx, AddressingModes::ZeroPageIndexedY));
        self.instructions.insert(0x8e, (CPU::stx, AddressingModes::Absolute));
        // STY
        self.instructions.insert(0x84, (CPU::sty, AddressingModes::ZeroPage));
        self.instructions.insert(0x94, (CPU::sty, AddressingModes::ZeroPageIndexedX));
        self.instructions.insert(0x8c, (CPU::sty, AddressingModes::Absolute));
        // TAX
        self.instructions.insert(0xaa, (CPU::tax, AddressingModes::Implicit));
        // TAY
        self.instructions.insert(0xa8, (CPU::tay, AddressingModes::Implicit));
        // TSX
        self.instructions.insert(0xba, (CPU::tsx, AddressingModes::Implicit));
        // TXA
        self.instructions.insert(0x8a, (CPU::txa, AddressingModes::Implicit));
        // TXS
        self.instructions.insert(0x9a, (CPU::txs, AddressingModes::Implicit));
        // TYA
        self.instructions.insert(0x98, (CPU::tya, AddressingModes::Implicit));

        // Insert a NOP instruction: opcode 0xEA
        self.instructions.insert(0xea, (
            |_cpu, _mode| {
                // NOP does nothing
            },
            AddressingModes::Implicit,
        ));

        // Optionally, handle BRK (opcode 0x00) as a basic interrupt simulation
        self.instructions.insert(0x00, (
            |cpu, _mode| {
                cpu.push_u16(cpu.registers.program_counter);
                let status = cpu.registers.status_register & !BREAK;
                cpu.push(status);
                cpu.registers.program_counter = cpu.read_u16(0xfffe);
                cpu.registers.status_register |= INTERRUPT_DISABLE;
            },
            AddressingModes::Implicit,
        ));
    }
}
