impl CPU {
    // Access
    fn lda(&mut self, mode: AddressingModes) {
        let value_for_acc: u8 = self.execute_mode(mode);
        self.registers.acc = value_for_acc;
        self.update_flags(&[FlagUpdate::Zero(value_for_acc), FlagUpdate::Negative(value_for_acc)]);
    }

    fn sta(&mut self, mode: AddressingModes) {
        let address = self.get_operand_address(mode);
        self.write(address, self.registers.acc);
    }

    fn ldx(&mut self, mode: AddressingModes) {
        let value_for_x: u8 = self.execute_mode(mode);
        self.registers.index_x = value_for_x;
        self.update_flags(&[FlagUpdate::Zero(value_for_x), FlagUpdate::Negative(value_for_x)]);
    }

    fn stx(&mut self, mode: AddressingModes) {
        // Mantendo a lógica original (usa execute_mode para obter o endereço)
        let value_for_ram: u8 = self.execute_mode(mode);
        self.write(value_for_ram as u16, self.registers.index_x);
    }

    fn ldy(&mut self, mode: AddressingModes) {
        let value_for_y: u8 = self.execute_mode(mode);
        self.registers.index_y = value_for_y;
        self.update_flags(&[FlagUpdate::Zero(value_for_y), FlagUpdate::Negative(value_for_y)]);
    }

    fn sty(&mut self, mode: AddressingModes) {
        let address = self.get_operand_address(mode);
        self.write(address, self.registers.index_y);
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
    }

    fn txa(&mut self, _mode: AddressingModes) {
        self.registers.acc = self.registers.index_x;
        self.update_flags(
            &[FlagUpdate::Zero(self.registers.acc), FlagUpdate::Negative(self.registers.acc)]
        );
    }

    fn tay(&mut self, _mode: AddressingModes) {
        self.registers.index_y = self.registers.acc;
        self.update_flags(
            &[
                FlagUpdate::Zero(self.registers.index_y),
                FlagUpdate::Negative(self.registers.index_y),
            ]
        );
    }

    fn tya(&mut self, _mode: AddressingModes) {
        self.registers.acc = self.registers.index_y;
        self.update_flags(
            &[FlagUpdate::Zero(self.registers.acc), FlagUpdate::Negative(self.registers.acc)]
        );
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
    }

    fn inc(&mut self, mode: AddressingModes) {
        let address = self.get_operand_address(mode);
        let value = self.read(address);
        let result = value.wrapping_add(1);
        self.write(address, result);
        self.update_flags(&[FlagUpdate::Zero(result), FlagUpdate::Negative(result)]);
    }

    fn dec(&mut self, mode: AddressingModes) {
        let address = self.get_operand_address(mode);
        let value = self.read(address);
        let result = value.wrapping_sub(1);
        self.write(address, result);
        self.update_flags(&[FlagUpdate::Zero(result), FlagUpdate::Negative(result)]);
    }

    fn inx(&mut self, _mode: AddressingModes) {
        self.registers.index_x = self.registers.index_x.wrapping_add(1);
        self.update_flags(
            &[
                FlagUpdate::Zero(self.registers.index_x),
                FlagUpdate::Negative(self.registers.index_x),
            ]
        );
    }

    fn dex(&mut self, _mode: AddressingModes) {
        self.registers.index_x = self.registers.index_x.wrapping_sub(1);
        self.update_flags(
            &[
                FlagUpdate::Zero(self.registers.index_x),
                FlagUpdate::Negative(self.registers.index_x),
            ]
        );
    }

    fn iny(&mut self, _mode: AddressingModes) {
        self.registers.index_y = self.registers.index_y.wrapping_add(1);
        self.update_flags(
            &[
                FlagUpdate::Zero(self.registers.index_y),
                FlagUpdate::Negative(self.registers.index_y),
            ]
        );
    }

    fn dey(&mut self, _mode: AddressingModes) {
        self.registers.index_y = self.registers.index_y.wrapping_sub(1);
        self.update_flags(
            &[
                FlagUpdate::Zero(self.registers.index_y),
                FlagUpdate::Negative(self.registers.index_y),
            ]
        );
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
        } else {
            let address = self.get_operand_address(mode);
            let value = self.read(address);
            let carry = (value & 0x80) != 0;
            let result = value << 1;
            self.write(address, result);
            self.update_flags(
                &[FlagUpdate::Zero(result), FlagUpdate::Negative(result), FlagUpdate::Carry(carry)]
            );
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
        } else {
            let address = self.get_operand_address(mode);
            let value = self.read(address);
            let carry = (value & 0x01) != 0;
            let result = value >> 1;
            self.write(address, result);
            self.update_flags(
                &[FlagUpdate::Zero(result), FlagUpdate::Negative(result), FlagUpdate::Carry(carry)]
            );
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
        }
    }

    // Bitwise
    fn and(&mut self, mode: AddressingModes) {
        let value_for_acc = self.execute_mode(mode);
        self.registers.acc &= value_for_acc;
    }

    fn ora(&mut self, mode: AddressingModes) {
        let value_for_acc = self.execute_mode(mode);
        self.registers.acc |= value_for_acc;
    }

    fn eor(&mut self, mode: AddressingModes) {
        let value_for_acc = self.execute_mode(mode);
        self.registers.acc ^= value_for_acc;
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
    }

    // Branch
    fn bcc(&mut self, _mode: AddressingModes) {
        let carry_clear = (self.registers.status_register & CARRY) == 0;
        self.branch_if(carry_clear);
    }

    fn bcs(&mut self, _mode: AddressingModes) {
        let carry_set = (self.registers.status_register & CARRY) != 0;
        self.branch_if(carry_set);
    }

    fn beq(&mut self, _mode: AddressingModes) {
        let zero_set = (self.registers.status_register & ZERO) != 0;
        self.branch_if(zero_set);
    }

    fn bne(&mut self, _mode: AddressingModes) {
        let zero_clear = (self.registers.status_register & ZERO) == 0;
        self.branch_if(zero_clear);
    }

    fn bpl(&mut self, _mode: AddressingModes) {
        let negative_clear = (self.registers.status_register & NEGATIVE) == 0;
        self.branch_if(negative_clear);
    }

    fn bmi(&mut self, _mode: AddressingModes) {
        let negative_set = (self.registers.status_register & NEGATIVE) != 0;
        self.branch_if(negative_set);
    }

    fn bvc(&mut self, _mode: AddressingModes) {
        let overflow_clear = (self.registers.status_register & OVERFLOW) == 0;
        self.branch_if(overflow_clear);
    }

    fn bvs(&mut self, _mode: AddressingModes) {
        let overflow_set = (self.registers.status_register & OVERFLOW) != 0;
        self.branch_if(overflow_set);
    }

    // Jump
    fn jmp(&mut self, mode: AddressingModes) {
        let address = self.get_operand_address(mode);
        self.registers.program_counter = address;
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
    }

    fn brk(&mut self, _mode: AddressingModes) {
        // Empilha PC+2 (endereço da próxima instrução após BRK)
        let return_addr = self.registers.program_counter.wrapping_add(2);
        let high = (return_addr >> 8) as u8;
        let low = (return_addr & 0xff) as u8;

        self.write(0x100 + (self.registers.stack_pointer as usize as u16), high);
        self.registers.stack_pointer = self.registers.stack_pointer.wrapping_sub(1);
        self.write(0x100 + (self.registers.stack_pointer as usize as u16), low);
        self.registers.stack_pointer = self.registers.stack_pointer.wrapping_sub(1);

        // Empilha status (com flag B setada)
        let status_with_b = self.registers.status_register | BREAK;
        self.write(0x100 + (self.registers.stack_pointer as usize as u16), status_with_b);
        self.registers.stack_pointer = self.registers.stack_pointer.wrapping_sub(1);

        // Ativa flag de interrupção
        self.update_flags(&[FlagUpdate::InterruptDisable(true)]);

        // Carrega o vetor de interrupção BRK
        let low = self.read(0xfffe);
        let high = self.read(0xffff);
        let irq_addr = self.turn_in_u16(low, high);

        self.registers.program_counter = irq_addr;
    }

    fn rti(&mut self, _mode: AddressingModes) {
        // Desempilha status (ignorando a flag B)
        self.registers.stack_pointer = self.registers.stack_pointer.wrapping_add(1);
        let status = self.read(0x100 + (self.registers.stack_pointer as usize as u16));
        self.registers.status_register = (status & !BREAK) | UNUSED;

        // Desempilha PC
        self.registers.stack_pointer = self.registers.stack_pointer.wrapping_add(1);
        let low = self.read(0x100 + (self.registers.stack_pointer as usize as u16));
        self.registers.stack_pointer = self.registers.stack_pointer.wrapping_add(1);
        let high = self.read(0x100 + (self.registers.stack_pointer as usize as u16));

        let return_addr = self.turn_in_u16(low, high);
        self.registers.program_counter = return_addr;
    }

    // Stack
    fn pha(&mut self, _mode: AddressingModes) {
        self.write(0x100 + (self.registers.stack_pointer as usize as u16), self.registers.acc);
        self.registers.stack_pointer = self.registers.stack_pointer.wrapping_sub(1);
    }

    fn pla(&mut self, _mode: AddressingModes) {
        self.registers.stack_pointer = self.registers.stack_pointer.wrapping_add(1);
        self.registers.acc = self.read(0x100 + (self.registers.stack_pointer as usize as u16));

        self.update_flags(
            &[FlagUpdate::Zero(self.registers.acc), FlagUpdate::Negative(self.registers.acc)]
        );
    }

    fn php(&mut self, _mode: AddressingModes) {
        // PHP sempre seta a flag B antes de empilhar
        let status_with_b = self.registers.status_register | BREAK | UNUSED;
        self.write(0x100 + (self.registers.stack_pointer as usize as u16), status_with_b);
        self.registers.stack_pointer = self.registers.stack_pointer.wrapping_sub(1);
    }

    fn plp(&mut self, _mode: AddressingModes) {
        self.registers.stack_pointer = self.registers.stack_pointer.wrapping_add(1);
        let status = self.read(0x100 + (self.registers.stack_pointer as usize as u16));
        // Quando desempilhamos, as flags B e UNUSED são mantidas como estavam antes
        self.registers.status_register =
            (status & !BREAK) | (self.registers.status_register & BREAK) | UNUSED;
    }

    fn txs(&mut self, _mode: AddressingModes) {
        self.registers.stack_pointer = self.registers.index_x;
        // Nota: TXS não afeta as flags
    }

    fn tsx(&mut self, _mode: AddressingModes) {
        self.registers.index_x = self.registers.stack_pointer;

        self.update_flags(
            &[
                FlagUpdate::Zero(self.registers.index_x),
                FlagUpdate::Negative(self.registers.index_x),
            ]
        );
    }

    // Flags
    fn clc(&mut self, _mode: AddressingModes) {
        self.registers.status_register &= !CARRY;
    }

    fn sec(&mut self, _mode: AddressingModes) {
        self.registers.status_register |= CARRY;
    }

    fn cli(&mut self, _mode: AddressingModes) {
        self.registers.status_register &= !INTERRUPT_DISABLE;
    }

    fn sei(&mut self, _mode: AddressingModes) {
        self.registers.status_register |= INTERRUPT_DISABLE;
    }

    fn cld(&mut self, _mode: AddressingModes) {
        self.registers.status_register &= !DECIMAL;
    }

    fn sed(&mut self, _mode: AddressingModes) {
        self.registers.status_register |= DECIMAL;
    }

    fn clv(&mut self, _mode: AddressingModes) {
        self.registers.status_register &= !OVERFLOW;
    }

    // Other
    fn nop(&mut self, _mode: AddressingModes) {
        // Não faz nada
    }
}
