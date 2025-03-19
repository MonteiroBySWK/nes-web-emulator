impl CPU {
    // Access
    fn lda(&mut self, mode: AddressingModes) {
        let value_for_acc: u8 = self.execute_mode(mode);
        self.registers.acc = value_for_acc;
        self.update_flags(&[FlagUpdate::Zero(value_for_acc), FlagUpdate::Negative(value_for_acc)]);
        
        // Adicionar ciclos com base no modo de endereçamento
        self.remaining_cycles += match mode {
            AddressingModes::Immediate => 2,
            AddressingModes::ZeroPage => 3,
            AddressingModes::ZeroPageIndexedX => 4,
            AddressingModes::Absolute => 4,
            AddressingModes::AbsoluteIndexedX => 4, // +1 se cruzar página
            AddressingModes::AbsoluteIndexedY => 4, // +1 se cruzar página
            AddressingModes::IndexedIndirect => 6,
            AddressingModes::IndirectIndexed => 5, // +1 se cruzar página
            _ => 2,
        };
    }

    fn sta(&mut self, mode: AddressingModes) {
        let address = self.get_operand_address(mode);
        self.write(address, self.registers.acc);
        
        // Adicionar ciclos com base no modo de endereçamento
        self.remaining_cycles += match mode {
            AddressingModes::ZeroPage => 3,
            AddressingModes::ZeroPageIndexedX => 4,
            AddressingModes::Absolute => 4,
            AddressingModes::AbsoluteIndexedX => 5,
            AddressingModes::AbsoluteIndexedY => 5,
            AddressingModes::IndexedIndirect => 6,
            AddressingModes::IndirectIndexed => 6,
            _ => 2,
        };
    }

    fn ldx(&mut self, mode: AddressingModes) {
        let value_for_x: u8 = self.execute_mode(mode);
        self.registers.index_x = value_for_x;
        self.update_flags(&[FlagUpdate::Zero(value_for_x), FlagUpdate::Negative(value_for_x)]);
        
        // Adicionar ciclos com base no modo de endereçamento
        self.remaining_cycles += match mode {
            AddressingModes::Immediate => 2,
            AddressingModes::ZeroPage => 3,
            AddressingModes::ZeroPageIndexedY => 4,
            AddressingModes::Absolute => 4,
            AddressingModes::AbsoluteIndexedY => 4, // +1 se cruzar página
            _ => 2,
        };
    }

    fn stx(&mut self, mode: AddressingModes) {
        let address = self.get_operand_address(mode);
        self.write(address, self.registers.index_x);
        
        // Adicionar ciclos com base no modo de endereçamento
        self.remaining_cycles += match mode {
            AddressingModes::ZeroPage => 3,
            AddressingModes::ZeroPageIndexedY => 4,
            AddressingModes::Absolute => 4,
            _ => 2,
        };
    }

    fn ldy(&mut self, mode: AddressingModes) {
        let value_for_y: u8 = self.execute_mode(mode);
        self.registers.index_y = value_for_y;
        self.update_flags(&[FlagUpdate::Zero(value_for_y), FlagUpdate::Negative(value_for_y)]);
        
        // Adicionar ciclos com base no modo de endereçamento
        self.remaining_cycles += match mode {
            AddressingModes::Immediate => 2,
            AddressingModes::ZeroPage => 3,
            AddressingModes::ZeroPageIndexedX => 4,
            AddressingModes::Absolute => 4,
            AddressingModes::AbsoluteIndexedX => 4, // +1 se cruzar página
            _ => 2,
        };
    }

    fn sty(&mut self, mode: AddressingModes) {
        let address = self.get_operand_address(mode);
        self.write(address, self.registers.index_y);
        
        // Adicionar ciclos com base no modo de endereçamento
        self.remaining_cycles += match mode {
            AddressingModes::ZeroPage => 3,
            AddressingModes::ZeroPageIndexedX => 4,
            AddressingModes::Absolute => 4,
            _ => 2,
        };
    }

    // Transfer
    fn tax(&mut self, _mode: AddressingModes) {
        self.registers.index_x = self.registers.acc;
        self.update_flags(
            &[
                FlagUpdate::Zero(self.registers.index_x),
                FlagUpdate::Negative(self.registers.index_x),
            ]
        );
        
        // Adicionar ciclos (constante para TAX)
        self.remaining_cycles += 2;
    }

    fn txa(&mut self, _mode: AddressingModes) {
        self.registers.acc = self.registers.index_x;
        self.update_flags(
            &[FlagUpdate::Zero(self.registers.acc), FlagUpdate::Negative(self.registers.acc)]
        );
        
        // Adicionar ciclos (constante para TXA)
        self.remaining_cycles += 2;
    }

    fn tay(&mut self, _mode: AddressingModes) {
        self.registers.index_y = self.registers.acc;
        self.update_flags(
            &[
                FlagUpdate::Zero(self.registers.index_y),
                FlagUpdate::Negative(self.registers.index_y),
            ]
        );
        
        // Adicionar ciclos (constante para TAY)
        self.remaining_cycles += 2;
    }

    fn tya(&mut self, _mode: AddressingModes) {
        self.registers.acc = self.registers.index_y;
        self.update_flags(
            &[FlagUpdate::Zero(self.registers.acc), FlagUpdate::Negative(self.registers.acc)]
        );
        
        // Adicionar ciclos (constante para TYA)
        self.remaining_cycles += 2;
    }

    // Arithmetic
    fn adc(&mut self, mode: AddressingModes) {
        let value = self.execute_mode(mode);
        let carry = if (self.registers.status_register & CARRY) != 0 { 1 } else { 0 };
        let result = (self.registers.acc as u16) + (value as u16) + (carry as u16);
        let overflow =
            (!(self.registers.acc ^ value) & (self.registers.acc ^ (result as u8)) & 0x80) != 0;
        self.registers.acc = result as u8;
        self.update_flags(
            &[
                FlagUpdate::Zero(self.registers.acc),
                FlagUpdate::Negative(self.registers.acc),
                FlagUpdate::Carry(result > 0xff),
                FlagUpdate::Overflow(overflow),
            ]
        );
        
        // Adicionar ciclos com base no modo de endereçamento
        self.remaining_cycles += match mode {
            AddressingModes::Immediate => 2,
            AddressingModes::ZeroPage => 3,
            AddressingModes::ZeroPageIndexedX => 4,
            AddressingModes::Absolute => 4,
            AddressingModes::AbsoluteIndexedX => 4, // +1 se cruzar página
            AddressingModes::AbsoluteIndexedY => 4, // +1 se cruzar página
            AddressingModes::IndexedIndirect => 6,
            AddressingModes::IndirectIndexed => 5, // +1 se cruzar página
            _ => 2,
        };
    }

    fn sbc(&mut self, mode: AddressingModes) {
        let value = self.execute_mode(mode);
        let carry = if (self.registers.status_register & CARRY) != 0 { 0 } else { 1 };
        let value_comp = !value;
        let result = (self.registers.acc as u16) + (value_comp as u16) + ((1 - carry) as u16);
        let overflow =
            ((self.registers.acc ^ value) & (self.registers.acc ^ (result as u8)) & 0x80) != 0;
        self.registers.acc = result as u8;
        self.update_flags(
            &[
                FlagUpdate::Zero(self.registers.acc),
                FlagUpdate::Negative(self.registers.acc),
                FlagUpdate::Carry(result > 0xff),
                FlagUpdate::Overflow(overflow),
            ]
        );
        
        // Adicionar ciclos com base no modo de endereçamento (mesmo que ADC)
        self.remaining_cycles += match mode {
            AddressingModes::Immediate => 2,
            AddressingModes::ZeroPage => 3,
            AddressingModes::ZeroPageIndexedX => 4,
            AddressingModes::Absolute => 4,
            AddressingModes::AbsoluteIndexedX => 4, // +1 se cruzar página
            AddressingModes::AbsoluteIndexedY => 4, // +1 se cruzar página
            AddressingModes::IndexedIndirect => 6,
            AddressingModes::IndirectIndexed => 5, // +1 se cruzar página
            _ => 2,
        };
    }

    fn inc(&mut self, mode: AddressingModes) {
        let address = self.get_operand_address(mode);
        let value = self.read(address);
        let result = value.wrapping_add(1);
        self.write(address, result);
        self.update_flags(&[FlagUpdate::Zero(result), FlagUpdate::Negative(result)]);
        
        // Adicionar ciclos com base no modo de endereçamento
        self.remaining_cycles += match mode {
            AddressingModes::ZeroPage => 5,
            AddressingModes::ZeroPageIndexedX => 6,
            AddressingModes::Absolute => 6,
            AddressingModes::AbsoluteIndexedX => 7,
            _ => 2,
        };
    }

    fn dec(&mut self, mode: AddressingModes) {
        let address = self.get_operand_address(mode);
        let value = self.read(address);
        let result = value.wrapping_sub(1);
        self.write(address, result);
        self.update_flags(&[FlagUpdate::Zero(result), FlagUpdate::Negative(result)]);
        
        // Adicionar ciclos com base no modo de endereçamento
        self.remaining_cycles += match mode {
            AddressingModes::ZeroPage => 5,
            AddressingModes::ZeroPageIndexedX => 6,
            AddressingModes::Absolute => 6,
            AddressingModes::AbsoluteIndexedX => 7,
            _ => 2,
        };
    }

    fn inx(&mut self, _mode: AddressingModes) {
        self.registers.index_x = self.registers.index_x.wrapping_add(1);
        self.update_flags(
            &[
                FlagUpdate::Zero(self.registers.index_x),
                FlagUpdate::Negative(self.registers.index_x),
            ]
        );
        
        // Adicionar ciclos (constante para INX)
        self.remaining_cycles += 2;
    }

    fn dex(&mut self, _mode: AddressingModes) {
        self.registers.index_x = self.registers.index_x.wrapping_sub(1);
        self.update_flags(
            &[
                FlagUpdate::Zero(self.registers.index_x),
                FlagUpdate::Negative(self.registers.index_x),
            ]
        );
        
        // Adicionar ciclos (constante para DEX)
        self.remaining_cycles += 2;
    }

    fn iny(&mut self, _mode: AddressingModes) {
        self.registers.index_y = self.registers.index_y.wrapping_add(1);
        self.update_flags(
            &[
                FlagUpdate::Zero(self.registers.index_y),
                FlagUpdate::Negative(self.registers.index_y),
            ]
        );
        
        // Adicionar ciclos (constante para INY)
        self.remaining_cycles += 2;
    }

    fn dey(&mut self, _mode: AddressingModes) {
        self.registers.index_y = self.registers.index_y.wrapping_sub(1);
        self.update_flags(
            &[
                FlagUpdate::Zero(self.registers.index_y),
                FlagUpdate::Negative(self.registers.index_y),
            ]
        );
        
        // Adicionar ciclos (constante para DEY)
        self.remaining_cycles += 2;
    }

    // Shift
    fn asl(&mut self, mode: AddressingModes) {
        if mode == AddressingModes::Accumulator {
            let carry = (self.registers.acc & 0x80) != 0;
            self.registers.acc <<= 1;
            self.update_flags(
                &[
                    FlagUpdate::Zero(self.registers.acc),
                    FlagUpdate::Negative(self.registers.acc),
                    FlagUpdate::Carry(carry),
                ]
            );
            
            // Acumulador: 2 ciclos
            self.remaining_cycles += 2;
        } else {
            let address = self.get_operand_address(mode);
            let value = self.read(address);
            let carry = (value & 0x80) != 0;
            let result = value << 1;
            self.write(address, result);
            self.update_flags(
                &[FlagUpdate::Zero(result), FlagUpdate::Negative(result), FlagUpdate::Carry(carry)]
            );
            
            // Outros modos: varia com o endereçamento
            self.remaining_cycles += match mode {
                AddressingModes::ZeroPage => 5,
                AddressingModes::ZeroPageIndexedX => 6,
                AddressingModes::Absolute => 6,
                AddressingModes::AbsoluteIndexedX => 7,
                _ => 2,
            };
        }
    }

    fn lsr(&mut self, mode: AddressingModes) {
        if mode == AddressingModes::Accumulator {
            let carry = (self.registers.acc & 0x01) != 0;
            self.registers.acc >>= 1;
            self.update_flags(
                &[
                    FlagUpdate::Zero(self.registers.acc),
                    FlagUpdate::Negative(self.registers.acc),
                    FlagUpdate::Carry(carry),
                ]
            );
            
            // Acumulador: 2 ciclos
            self.remaining_cycles += 2;
        } else {
            let address = self.get_operand_address(mode);
            let value = self.read(address);
            let carry = (value & 0x01) != 0;
            let result = value >> 1;
            self.write(address, result);
            self.update_flags(
                &[FlagUpdate::Zero(result), FlagUpdate::Negative(result), FlagUpdate::Carry(carry)]
            );
            
            // Outros modos: varia com o endereçamento
            self.remaining_cycles += match mode {
                AddressingModes::ZeroPage => 5,
                AddressingModes::ZeroPageIndexedX => 6,
                AddressingModes::Absolute => 6,
                AddressingModes::AbsoluteIndexedX => 7,
                _ => 2,
            };
        }
    }

    fn rol(&mut self, mode: AddressingModes) {
        let old_carry = (self.registers.status_register & CARRY) != 0;
        if mode == AddressingModes::Accumulator {
            let new_carry = (self.registers.acc & 0x80) != 0;
            self.registers.acc = (self.registers.acc << 1) | (if old_carry { 1 } else { 0 });
            self.update_flags(
                &[
                    FlagUpdate::Zero(self.registers.acc),
                    FlagUpdate::Negative(self.registers.acc),
                    FlagUpdate::Carry(new_carry),
                ]
            );
            
            // Acumulador: 2 ciclos
            self.remaining_cycles += 2;
        } else {
            let address = self.get_operand_address(mode);
            let value = self.read(address);
            let new_carry = (value & 0x80) != 0;
            let result = (value << 1) | (if old_carry { 1 } else { 0 });
            self.write(address, result);
            self.update_flags(
                &[
                    FlagUpdate::Zero(result),
                    FlagUpdate::Negative(result),
                    FlagUpdate::Carry(new_carry),
                ]
            );
            
            // Outros modos: varia com o endereçamento
            self.remaining_cycles += match mode {
                AddressingModes::ZeroPage => 5,
                AddressingModes::ZeroPageIndexedX => 6,
                AddressingModes::Absolute => 6,
                AddressingModes::AbsoluteIndexedX => 7,
                _ => 2,
            };
        }
    }

    fn ror(&mut self, mode: AddressingModes) {
        let old_carry = (self.registers.status_register & CARRY) != 0;
        if mode == AddressingModes::Accumulator {
            let new_carry = (self.registers.acc & 0x01) != 0;
            self.registers.acc = (self.registers.acc >> 1) | (if old_carry { 0x80 } else { 0 });
            self.update_flags(
                &[
                    FlagUpdate::Zero(self.registers.acc),
                    FlagUpdate::Negative(self.registers.acc),
                    FlagUpdate::Carry(new_carry),
                ]
            );
            
            // Acumulador: 2 ciclos
            self.remaining_cycles += 2;
        } else {
            let address = self.get_operand_address(mode);
            let value = self.read(address);
            let new_carry = (value & 0x01) != 0;
            let result = (value >> 1) | (if old_carry { 0x80 } else { 0 });
            self.write(address, result);
            self.update_flags(
                &[
                    FlagUpdate::Zero(result),
                    FlagUpdate::Negative(result),
                    FlagUpdate::Carry(new_carry),
                ]
            );
            
            // Outros modos: varia com o endereçamento
            self.remaining_cycles += match mode {
                AddressingModes::ZeroPage => 5,
                AddressingModes::ZeroPageIndexedX => 6,
                AddressingModes::Absolute => 6,
                AddressingModes::AbsoluteIndexedX => 7,
                _ => 2,
            };
        }
    }

    // Bitwise
    fn and(&mut self, mode: AddressingModes) {
        let value_for_acc = self.execute_mode(mode);
        self.registers.acc &= value_for_acc;
        
        // Adicionar ciclos com base no modo de endereçamento
        self.remaining_cycles += match mode {
            AddressingModes::Immediate => 2,
            AddressingModes::ZeroPage => 3,
            AddressingModes::ZeroPageIndexedX => 4,
            AddressingModes::Absolute => 4,
            AddressingModes::AbsoluteIndexedX => 4, // +1 se cruzar página
            AddressingModes::AbsoluteIndexedY => 4, // +1 se cruzar página
            AddressingModes::IndexedIndirect => 6,
            AddressingModes::IndirectIndexed => 5, // +1 se cruzar página
            _ => 2,
        };
    }

    fn ora(&mut self, mode: AddressingModes) {
        let value_for_acc = self.execute_mode(mode);
        self.registers.acc |= value_for_acc;
        
        // Adicionar ciclos com base no modo de endereçamento
        self.remaining_cycles += match mode {
            AddressingModes::Immediate => 2,
            AddressingModes::ZeroPage => 3,
            AddressingModes::ZeroPageIndexedX => 4,
            AddressingModes::Absolute => 4,
            AddressingModes::AbsoluteIndexedX => 4, // +1 se cruzar página
            AddressingModes::AbsoluteIndexedY => 4, // +1 se cruzar página
            AddressingModes::IndexedIndirect => 6,
            AddressingModes::IndirectIndexed => 5, // +1 se cruzar página
            _ => 2,
        };
    }

    fn eor(&mut self, mode: AddressingModes) {
        let value_for_acc = self.execute_mode(mode);
        self.registers.acc ^= value_for_acc;
        
        // Adicionar ciclos com base no modo de endereçamento
        self.remaining_cycles += match mode {
            AddressingModes::Immediate => 2,
            AddressingModes::ZeroPage => 3,
            AddressingModes::ZeroPageIndexedX => 4,
            AddressingModes::Absolute => 4,
            AddressingModes::AbsoluteIndexedX => 4, // +1 se cruzar página
            AddressingModes::AbsoluteIndexedY => 4, // +1 se cruzar página
            AddressingModes::IndexedIndirect => 6,
            AddressingModes::IndirectIndexed => 5, // +1 se cruzar página
            _ => 2,
        };
    }

    fn bit(&mut self, mode: AddressingModes) {
        let value = self.execute_mode(mode);
        let result = self.registers.acc & value;

        self.update_flags(
            &[
                FlagUpdate::Zero(result),
                FlagUpdate::Overflow((value & 0x40) != 0),
                FlagUpdate::Negative(if (value & 0x80) != 0 { 1 } else { 0 }),
            ]
        );
        
        // Adicionar ciclos com base no modo de endereçamento
        self.remaining_cycles += match mode {
            AddressingModes::ZeroPage => 3,
            AddressingModes::Absolute => 4,
            _ => 2,
        };
    }

    // Compare
    fn cmp(&mut self, mode: AddressingModes) {
        let value = self.execute_mode(mode);
        let result = self.registers.acc.wrapping_sub(value);

        self.update_flags(
            &[
                FlagUpdate::Zero(result),
                FlagUpdate::Negative(result),
                FlagUpdate::Carry(self.registers.acc >= value),
            ]
        );
        
        // Adicionar ciclos com base no modo de endereçamento
        self.remaining_cycles += match mode {
            AddressingModes::Immediate => 2,
            AddressingModes::ZeroPage => 3,
            AddressingModes::ZeroPageIndexedX => 4,
            AddressingModes::Absolute => 4,
            AddressingModes::AbsoluteIndexedX => 4, // +1 se cruzar página
            AddressingModes::AbsoluteIndexedY => 4, // +1 se cruzar página
            AddressingModes::IndexedIndirect => 6,
            AddressingModes::IndirectIndexed => 5, // +1 se cruzar página
            _ => 2,
        };
    }

    fn cpx(&mut self, mode: AddressingModes) {
        let value = self.execute_mode(mode);
        let result = self.registers.index_x.wrapping_sub(value);

        self.update_flags(
            &[
                FlagUpdate::Zero(result),
                FlagUpdate::Negative(result),
                FlagUpdate::Carry(self.registers.index_x >= value),
            ]
        );
        
        // Adicionar ciclos com base no modo de endereçamento
        self.remaining_cycles += match mode {
            AddressingModes::Immediate => 2,
            AddressingModes::ZeroPage => 3,
            AddressingModes::Absolute => 4,
            _ => 2,
        };
    }

    fn cpy(&mut self, mode: AddressingModes) {
        let value = self.execute_mode(mode);
        let result = self.registers.index_y.wrapping_sub(value);

        self.update_flags(
            &[
                FlagUpdate::Zero(result),
                FlagUpdate::Negative(result),
                FlagUpdate::Carry(self.registers.index_y >= value),
            ]
        );
        
        // Adicionar ciclos com base no modo de endereçamento
        self.remaining_cycles += match mode {
            AddressingModes::Immediate => 2,
            AddressingModes::ZeroPage => 3,
            AddressingModes::Absolute => 4,
            _ => 2,
        };
    }

    // Branch
    fn bcc(&mut self, _mode: AddressingModes) {
        let carry_clear = (self.registers.status_register & CARRY) == 0;
        self.branch_if(carry_clear);
        
        // Ciclos: 2 (+ 1 se ramificação for tomada, + 1 se cruzar página)
        self.remaining_cycles += 2;
    }

    fn bcs(&mut self, _mode: AddressingModes) {
        let carry_set = (self.registers.status_register & CARRY) != 0;
        self.branch_if(carry_set);
        
        // Ciclos: 2 (+ 1 se ramificação for tomada, + 1 se cruzar página)
        self.remaining_cycles += 2;
    }

    fn beq(&mut self, _mode: AddressingModes) {
        let zero_set = (self.registers.status_register & ZERO) != 0;
        self.branch_if(zero_set);
        
        // Ciclos: 2 (+ 1 se ramificação for tomada, + 1 se cruzar página)
        self.remaining_cycles += 2;
    }

    fn bne(&mut self, _mode: AddressingModes) {
        let zero_clear = (self.registers.status_register & ZERO) == 0;
        self.branch_if(zero_clear);
        
        // Ciclos: 2 (+ 1 se ramificação for tomada, + 1 se cruzar página)
        self.remaining_cycles += 2;
    }

    fn bpl(&mut self, _mode: AddressingModes) {
        let negative_clear = (self.registers.status_register & NEGATIVE) == 0;
        self.branch_if(negative_clear);
        
        // Ciclos: 2 (+ 1 se ramificação for tomada, + 1 se cruzar página)
        self.remaining_cycles += 2;
    }

    fn bmi(&mut self, _mode: AddressingModes) {
        let negative_set = (self.registers.status_register & NEGATIVE) != 0;
        self.branch_if(negative_set);
        
        // Ciclos: 2 (+ 1 se ramificação for tomada, + 1 se cruzar página)
        self.remaining_cycles += 2;
    }

    fn bvc(&mut self, _mode: AddressingModes) {
        let overflow_clear = (self.registers.status_register & OVERFLOW) == 0;
        self.branch_if(overflow_clear);
        
        // Ciclos: 2 (+ 1 se ramificação for tomada, + 1 se cruzar página)
        self.remaining_cycles += 2;
    }

    fn bvs(&mut self, _mode: AddressingModes) {
        let overflow_set = (self.registers.status_register & OVERFLOW) != 0;
        self.branch_if(overflow_set);
        
        // Ciclos: 2 (+ 1 se ramificação for tomada, + 1 se cruzar página)
        self.remaining_cycles += 2;
    }

    // Jump
    fn jmp(&mut self, mode: AddressingModes) {
        let address = self.get_operand_address(mode);
        self.registers.program_counter = address;
        
        // Ciclos com base no modo de endereçamento
        self.remaining_cycles += match mode {
            AddressingModes::Absolute => 3,
            AddressingModes::Indirect => 5,
            _ => 3,
        };
    }

    fn jsr(&mut self, _mode: AddressingModes) {
        // Armazena o endereço de retorno (PC - 1) na pilha
        let return_addr = self.registers.program_counter.wrapping_add(1);
        let high = (return_addr >> 8) as u8;
        let low = (return_addr & 0xff) as u8;

        // Empilha o endereço de retorno (primeiro high, depois low)
        self.write(0x100 + (self.registers.stack_pointer as usize as u16), high);
        self.registers.stack_pointer = self.registers.stack_pointer.wrapping_sub(1);
        self.write(0x100 + (self.registers.stack_pointer as usize as u16), low);
        self.registers.stack_pointer = self.registers.stack_pointer.wrapping_sub(1);

        // Obtém o endereço destino
        let low = self.fetch();
        let high = self.fetch();
        let jump_addr = self.turn_in_u16(low, high);

        // Pula para o endereço
        self.registers.program_counter = jump_addr;
        
        // Ciclos para JSR
        self.remaining_cycles += 6;
    }

    fn rts(&mut self, _mode: AddressingModes) {
        // Desempilha o endereço de retorno (primeiro low, depois high)
        self.registers.stack_pointer = self.registers.stack_pointer.wrapping_add(1);
        let low = self.read(0x100 + (self.registers.stack_pointer as usize as u16));
        self.registers.stack_pointer = self.registers.stack_pointer.wrapping_add(1);
        let high = self.read(0x100 + (self.registers.stack_pointer as usize as u16));

        let return_addr = self.turn_in_u16(low, high);

        // Retorna para o endereço + 1
        self.registers.program_counter = return_addr.wrapping_add(1);

        // Ciclos para RTS
        self.remaining_cycles += 6;
    }

    fn brk(&mut self, _mode: AddressingModes) {
        // ...existing code...

        // Ciclos para BRK
        self.remaining_cycles += 7;
    }

    fn rti(&mut self, _mode: AddressingModes) {
        // ...existing code...

        // Ciclos para RTI
        self.remaining_cycles += 6;
    }

    fn pha(&mut self, _mode: AddressingModes) {
        self.write(0x100 + (self.registers.stack_pointer as usize as u16), self.registers.acc);
        self.registers.stack_pointer = self.registers.stack_pointer.wrapping_sub(1);
        
        // Ciclos para PHA
        self.remaining_cycles += 3;
    }

    fn pla(&mut self, _mode: AddressingModes) {
        // ...existing code...

        // Ciclos para PLA
        self.remaining_cycles += 4;
    }

    fn php(&mut self, _mode: AddressingModes) {
        // ...existing code...

        // Ciclos para PHP
        self.remaining_cycles += 3;
    }

    fn plp(&mut self, _mode: AddressingModes) {
        // ...existing code...

        // Ciclos para PLP
        self.remaining_cycles += 4;
    }

    fn txs(&mut self, _mode: AddressingModes) {
        self.registers.stack_pointer = self.registers.index_x;
        
        // Ciclos para TXS
        self.remaining_cycles += 2;
    }

    fn tsx(&mut self, _mode: AddressingModes) {
        // ...existing code...

        // Ciclos para TSX
        self.remaining_cycles += 2;
    }

    // Flags - todas as operações de flag levam 2 ciclos
    fn clc(&mut self, _mode: AddressingModes) {
        self.registers.status_register &= !CARRY;
        self.remaining_cycles += 2;
    }

    fn sec(&mut self, _mode: AddressingModes) {
        self.registers.status_register |= CARRY;
        self.remaining_cycles += 2;
    }

    fn cli(&mut self, _mode: AddressingModes) {
        self.registers.status_register &= !INTERRUPT_DISABLE;
        self.remaining_cycles += 2;
    }

    fn sei(&mut self, _mode: AddressingModes) {
        self.registers.status_register |= INTERRUPT_DISABLE;
        self.remaining_cycles += 2;
    }

    fn cld(&mut self, _mode: AddressingModes) {
        self.registers.status_register &= !DECIMAL;
        self.remaining_cycles += 2;
    }

    fn sed(&mut self, _mode: AddressingModes) {
        self.registers.status_register |= DECIMAL;
        self.remaining_cycles += 2;
    }

    fn clv(&mut self, _mode: AddressingModes) {
        self.registers.status_register &= !OVERFLOW;
        self.remaining_cycles += 2;
    }

    fn nop(&mut self, _mode: AddressingModes) {
        // NOP leva 2 ciclos
        self.remaining_cycles += 2;
    }
}