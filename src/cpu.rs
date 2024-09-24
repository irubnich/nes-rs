use crate::bus::Bus;
use crate::registers::{Registers, StackPointer, Status, StatusArgs};
use crate::instruction::{DecodedInstr, Nmos6502, AddressingMode, OpInput, Instruction};

fn address_from_bytes(lo: u8, hi: u8) -> u16 {
    u16::from(hi) << 8 | u16::from(lo)
}

pub struct CPU {
    pub registers: Registers,
    pub bus: Bus,
    pub cycles: u8,
    pub clock_count: u32,
}

impl CPU {
    pub fn get_byte(&mut self, address: u16) -> u8 {
        self.bus.cpu_read(address)
    }

    pub fn set_byte(&mut self, address: u16, value: u8) {
        self.bus.cpu_write(address, value);
    }

    pub fn reset(&mut self) {
        let lo = self.get_byte(0xFFFC);
        let hi = self.get_byte(0xFFFD);
        self.registers.pc = u16::from_be_bytes([hi, lo]);

        self.registers.a = 0;
        self.registers.x = 0;
        self.registers.y = 0;
        self.registers.stkp = StackPointer(self.registers.stkp.0.wrapping_sub(3));
        self.registers.status = Status::empty();
        self.registers.status.insert(Status::PS_DISABLE_INTERRUPTS);

        self.cycles = 8;
        self.clock_count = 0;
    }

    fn read_address(&mut self, addr: u16) -> [u8; 2] {
            let lo = self.get_byte(addr);
            let hi = self.get_byte(addr.wrapping_add(1));
            [lo, hi]
    }

    pub fn fetch_next_and_decode(&mut self) -> Option<DecodedInstr> {
        let x: u8 = self.get_byte(self.registers.pc);

        match Nmos6502::decode(x) {
            Some((instr, am, cycles)) => {
                let extra_bytes = am.extra_bytes();
                let num_bytes = extra_bytes + 1;

                let data_start = self.registers.pc.wrapping_add(1);

                let slice = if extra_bytes == 0 {
                    [0, 0]
                } else if extra_bytes == 1 {
                    [self.get_byte(data_start), 0]
                } else if extra_bytes == 2 {
                    [
                        self.get_byte(data_start),
                        self.get_byte(data_start.wrapping_add(1)),
                    ]
                } else {
                    panic!()
                };

                let x = self.registers.x;
                let y = self.registers.y;

                let (am_out, cycles, extra_cycle) = match am {
                    AddressingMode::ACC => (OpInput::UseImplied, cycles, false),
                    AddressingMode::IMP => (OpInput::UseImplied, cycles, false),
                    AddressingMode::IMM => (OpInput::UseImmediate(slice[0]), cycles, false),
                    AddressingMode::ZP0 => (OpInput::UseAddress(u16::from(slice[0])), cycles, false),
                    AddressingMode::ZPX => (OpInput::UseAddress(u16::from(slice[0].wrapping_add(x))), cycles, false),
                    AddressingMode::ZPY => (OpInput::UseAddress(u16::from(slice[0].wrapping_add(y))), cycles, false),
                    AddressingMode::REL => {
                        let offset = slice[0];
                        let sign_extend = if offset & 0x80 == 0x80 { 0xFFu8 } else { 0x0 };
                        let rel = u16::from_le_bytes([offset, sign_extend]);
                        (OpInput::UseRelative(rel), cycles, false)
                    },
                    AddressingMode::ABS => (OpInput::UseAddress(address_from_bytes(slice[0], slice[1])), cycles, false),
                    AddressingMode::ABX => {
                        let x = slice[0];
                        let addr = address_from_bytes(slice[0], slice[1]).wrapping_add(x.into());
                        let ex = if (addr & 0xFF00) != ((slice[1] as u16) << 8).into() {
                            true
                        } else {
                            false
                        };

                        (OpInput::UseAddress(addr), cycles, ex)
                    }
                    AddressingMode::ABY => {
                        let addr = address_from_bytes(slice[0], slice[1]).wrapping_add(y.into());
                        let ex = if (addr & 0xFF00) != ((slice[1] as u16) << 8).into() {
                            true
                        } else {
                            false
                        };

                        (OpInput::UseAddress(addr), cycles, ex)
                    }
                    AddressingMode::IND => {
                        let slice = self.read_address(address_from_bytes(slice[0], slice[1]));
                        (OpInput::UseAddress(address_from_bytes(slice[0], slice[1])), cycles, false)
                    },
                    AddressingMode::IZX => {
                        let start = slice[0].wrapping_add(x);
                        let slice = self.read_address(u16::from(start));
                        (OpInput::UseAddress(address_from_bytes(slice[0], slice[1])), cycles, false)
                    }
                    AddressingMode::IZY => {
                        let start = slice[0];
                        let slice = self.read_address(u16::from(start));

                        let addr = address_from_bytes(slice[0], slice[1]).wrapping_add(y.into());
                        let ex = if (addr & 0xFF00) != ((slice[1] as u16) << 8).into() {
                            true
                        } else {
                            false
                        };

                        (OpInput::UseAddress(addr), cycles, ex)
                    },
                    AddressingMode::BuggyIndirect => {
                        let pointer = address_from_bytes(slice[0], slice[1]);
                        let low_byte_of_target = self.get_byte(pointer);
                        let low_byte_of_incremented_pointer = pointer.to_le_bytes()[0].wrapping_add(1);
                        let incremented_pointer = u16::from_le_bytes([low_byte_of_incremented_pointer, pointer.to_le_bytes()[1]]);

                        let high_byte_of_target = self.get_byte(incremented_pointer);
                        (OpInput::UseAddress(address_from_bytes(low_byte_of_target, high_byte_of_target)), cycles, false)
                    }
                };

                self.registers.pc = self.registers.pc.wrapping_add(num_bytes);

                Some((instr, am_out, cycles, extra_cycle))
            }
            _ => None,
        }
    }

    pub fn execute_instruction(&mut self, decoded_instr: DecodedInstr) -> u8 {
        match decoded_instr {
            (Instruction::LDA, OpInput::UseAddress(addr), cycles, extra_cycle) => {
                let val = self.get_byte(addr);
                self.load_a(val);
                cycles + if extra_cycle { 1 } else { 0 }
            },
            (Instruction::SEC, OpInput::UseImplied, cycles, _) => {
                self.registers.status.or(Status::PS_CARRY);
                cycles
            },
            (Instruction::SBC, OpInput::UseAddress(addr), cycles, extra_cycle) => {
                let val = self.get_byte(addr);
                self.subtract_with_carry(val);
                cycles + if extra_cycle { 1 } else { 0 }
            }
            (Instruction::BEQ, OpInput::UseRelative(rel), cycles, _) => {
                let addr = self.registers.pc.wrapping_add(rel);
                self.branch_if_equal(addr);
                cycles
            },
            (Instruction::BMI, OpInput::UseRelative(rel), cycles, _) => {
                let addr = self.registers.pc.wrapping_add(rel);
                self.branch_if_minus(addr);
                cycles
            },
            (Instruction::STA, OpInput::UseAddress(addr), cycles, _) => {
                self.set_byte(addr, self.registers.a);
                cycles
            },
            (Instruction::JMP, OpInput::UseAddress(addr), cycles, _) => {
                self.jump(addr);
                cycles
            },
            (Instruction::LDX, OpInput::UseAddress(addr), cycles, extra_cycle) => {
                let val = self.get_byte(addr);
                self.load_x(val);
                cycles + if extra_cycle { 1 } else { 0 }
            },
            (Instruction::LDY, OpInput::UseAddress(addr), cycles, extra_cycle) => {
                let val = self.get_byte(addr);
                self.load_y(val);
                cycles + if extra_cycle { 1 } else { 0 }
            },
            (Instruction::STX, OpInput::UseAddress(addr), cycles, _) => {
                self.set_byte(addr, self.registers.x);
                cycles
            },
            (Instruction::STY, OpInput::UseAddress(addr), cycles, _) => {
                self.set_byte(addr, self.registers.y);
                cycles
            },
            (Instruction::BRK, OpInput::UseImplied, cycles, _) => {
                self.registers.pc += 1;
                let hilo = self.registers.pc.to_be_bytes();
                self.push_on_stack(hilo[0]);
                self.push_on_stack(hilo[1]);
                self.registers.status.insert(Status::PS_BRK);
                self.push_on_stack(self.registers.status.bits());
                self.registers.status.insert(Status::PS_DISABLE_INTERRUPTS);
                self.registers.pc = u16::from_be_bytes([self.get_byte(0xFFFE), self.get_byte(0xFFFF)]);
                cycles
            },
            (Instruction::EOR, OpInput::UseAddress(addr), cycles, extra_cycle) => {
                let val = self.get_byte(addr);
                self.xor(val);
                cycles + if extra_cycle { 1 } else { 0 }
            }
            (Instruction::ORA, OpInput::UseAddress(addr), cycles, extra_cycle) => {
                let val = self.get_byte(addr);
                self.inclusive_or(val);
                cycles + if extra_cycle { 1 } else { 0 }
            }
            (Instruction::LDA, OpInput::UseImmediate(val), cycles, extra_cycle) => {
                self.load_a(val);
                cycles + if extra_cycle { 1 } else { 0 }
            }
            (Instruction::ADC, OpInput::UseImmediate(val), cycles, extra_cycle) => {
                self.add_with_carry(val);
                cycles + if extra_cycle { 1 } else { 0 }
            }
            (Instruction::ADC, OpInput::UseAddress(addr), cycles, extra_cycle) => {
                let val = self.get_byte(addr);
                self.add_with_carry(val);
                cycles + if extra_cycle { 1 } else { 0 }
            }
            (Instruction::LDX, OpInput::UseImmediate(val), cycles, extra_cycle) => {
                self.load_x(val);
                cycles + if extra_cycle { 1 } else { 0 }
            }
            (Instruction::LDY, OpInput::UseImmediate(val), cycles, extra_cycle) => {
                self.load_y(val);
                cycles + if extra_cycle { 1 } else { 0 }
            }
            (Instruction::PHA, OpInput::UseImplied, cycles, _) => {
                self.push_on_stack(self.registers.a);
                cycles
            }
            (Instruction::JSR, OpInput::UseAddress(addr), cycles, _) => {
                for b in self.registers.pc.wrapping_sub(1).to_be_bytes() {
                    self.push_on_stack(b);
                }
                self.jump(addr);
                cycles
            }
            (Instruction::CMP, OpInput::UseImmediate(val), cycles, extra_cycle) => {
                self.compare_with_a_register(val);
                cycles + if extra_cycle { 1 } else { 0 }
            }
            (Instruction::CMP, OpInput::UseAddress(addr), cycles, extra_cycle) => {
                let val = self.get_byte(addr);
                self.compare_with_a_register(val);
                cycles + if extra_cycle { 1 } else { 0 }
            }
            (Instruction::BNE, OpInput::UseRelative(rel), cycles, _) => {
                let addr = self.registers.pc.wrapping_add(rel);
                self.branch_if_not_equal(addr);
                cycles
            }
            (Instruction::RTS, OpInput::UseImplied, cycles, _) => {
                self.pull_from_stack();
                let pcl: u8 = self.pull_from_stack();
                let pch: u8 = self.fetch_from_stack();
                self.registers.pc = ((u16::from(pch) << 8) | u16::from(pcl)).wrapping_add(1);
                cycles
            }
            (Instruction::SEI, OpInput::UseImplied, cycles, _) => {
                self.registers.status.or(Status::PS_DISABLE_INTERRUPTS);
                cycles
            }
            (Instruction::CLD, OpInput::UseImplied, cycles, _) => {
                self.registers.status.and(!Status::PS_DECIMAL_MODE);
                cycles
            }
            (Instruction::TXS, OpInput::UseImplied, cycles, _) => {
                self.registers.stkp = StackPointer(self.registers.x);
                cycles
            }
            (Instruction::BPL, OpInput::UseRelative(rel), cycles, _) => {
                let addr = self.registers.pc.wrapping_add(rel);
                self.branch_if_positive(addr);
                cycles
            }
            (Instruction::BCS, OpInput::UseRelative(rel), cycles, _) => {
                let addr = self.registers.pc.wrapping_add(rel);
                self.branch_if_carry_set(addr);
                cycles
            }
            (Instruction::CLC, OpInput::UseImplied, cycles, _) => {
                self.registers.status.and(!Status::PS_CARRY);
                cycles
            }
            (Instruction::BCC, OpInput::UseRelative(rel), cycles, _) => {
                let addr = self.registers.pc.wrapping_add(rel);
                self.branch_if_carry_clear(addr);
                cycles
            }
            (Instruction::BIT, OpInput::UseAddress(addr), cycles, _) => {
                let a = self.registers.a;
                let m = self.get_byte(addr);
                let res = a & m;

                let is_zero = 0 == res;
                let is_negative = 0 != (0x80 & m);
                let v = 0 != (0x40 & m);

                self.registers.status.set_with_mask(
                    Status::PS_ZERO | Status::PS_NEGATIVE | Status::PS_OVERFLOW,
                    Status::new(StatusArgs {
                        z: is_zero,
                        n: is_negative,
                        v: v,
                        ..StatusArgs::none()
                    })
                );
                cycles
            }
            (Instruction::BVS, OpInput::UseRelative(rel), cycles, _) => {
                let addr = self.registers.pc.wrapping_add(rel);
                self.branch_if_overflow_set(addr);
                cycles
            }
            (Instruction::BVC, OpInput::UseRelative(rel), cycles, _) => {
                let addr = self.registers.pc.wrapping_add(rel);
                self.branch_if_overflow_clear(addr);
                cycles
            }
            (Instruction::SED, OpInput::UseImplied, cycles, _) => {
                self.registers.status.or(Status::PS_DECIMAL_MODE);
                cycles
            }
            (Instruction::PHP, OpInput::UseImplied, cycles, _) => {
                let val = self.registers.status.bits() | 0x30;
                self.push_on_stack(val);
                cycles
            }
            (Instruction::PLA, OpInput::UseImplied, cycles, _) => {
                self.pull_from_stack();
                let val = self.fetch_from_stack();
                self.registers.a = val;
                self.registers.status.set_with_mask(
                    Status::PS_ZERO | Status::PS_NEGATIVE,
                    Status::new(StatusArgs {
                        z: val == 0,
                        n: self.registers.a > 127,
                        ..StatusArgs::none()
                    })
                );
                cycles
            }
            (Instruction::AND, OpInput::UseImmediate(val), cycles, extra_cycle) => {
                self.and(val);
                cycles + if extra_cycle { 1 } else { 0 }
            }
            (Instruction::PLP, OpInput::UseImplied, cycles, _) => {
                self.pull_from_stack();
                let val = self.fetch_from_stack();
                self.registers.status = Status::from_bits_truncate(val);
                cycles
            }
            (Instruction::ORA, OpInput::UseImmediate(val), cycles, extra_cycle) => {
                self.inclusive_or(val);
                cycles + if extra_cycle { 1 } else { 0 }
            }
            (Instruction::CLV, OpInput::UseImplied, cycles, _) => {
                self.registers.status.and(!Status::PS_OVERFLOW);
                cycles
            }
            (Instruction::EOR, OpInput::UseImmediate(val), cycles, extra_cycle) => {
                self.exclusive_or(val);
                cycles + if extra_cycle { 1 } else { 0 }
            }
            (Instruction::CPY, OpInput::UseImmediate(val), cycles, _) => {
                self.compare_with_y_register(val);
                cycles
            }
            (Instruction::CPY, OpInput::UseAddress(addr), cycles, _) => {
                let val = self.get_byte(addr);
                self.compare_with_y_register(val);
                cycles
            }
            (Instruction::CPX, OpInput::UseImmediate(val), cycles, _) => {
                self.compare_with_x_register(val);
                cycles
            }
            (Instruction::CPX, OpInput::UseAddress(addr), cycles, _) => {
                let val = self.get_byte(addr);
                self.compare_with_x_register(val);
                cycles
            }
            (Instruction::SBC, OpInput::UseImmediate(val), cycles, extra_cycle) => {
                self.subtract_with_carry(val);
                cycles + if extra_cycle { 1 } else { 0 }
            }
            (Instruction::INY, OpInput::UseImplied, cycles, _) => {
                CPU::increment(&mut self.registers.y, &mut self.registers.status);
                cycles
            }
            (Instruction::INX, OpInput::UseImplied, cycles, _) => {
                CPU::increment(&mut self.registers.x, &mut self.registers.status);
                cycles
            }
            (Instruction::DEY, OpInput::UseImplied, cycles, _) => {
                CPU::decrement(&mut self.registers.y, &mut self.registers.status);
                cycles
            }
            (Instruction::DEX, OpInput::UseImplied, cycles, _) => {
                CPU::decrement(&mut self.registers.x, &mut self.registers.status);
                cycles
            }
            (Instruction::TAY, OpInput::UseImplied, cycles, _) => {
                self.load_y(self.registers.a);
                cycles
            }
            (Instruction::TAX, OpInput::UseImplied, cycles, _) => {
                self.load_x(self.registers.a);
                cycles
            }
            (Instruction::TYA, OpInput::UseImplied, cycles, _) => {
                self.load_a(self.registers.y);
                cycles
            }
            (Instruction::TXA, OpInput::UseImplied, cycles, _) => {
                self.load_a(self.registers.x);
                cycles
            }
            (Instruction::TSX, OpInput::UseImplied, cycles, _) => {
                let StackPointer(val) = self.registers.stkp;
                self.load_x(val);
                cycles
            }
            (Instruction::RTI, OpInput::UseImplied, cycles, _) => {
                self.pull_from_stack();
                let val = self.pull_from_stack();
                self.registers.status = Status::from_bits_truncate(val);
                let pcl = self.pull_from_stack();
                let pch = self.fetch_from_stack();
                self.registers.pc = address_from_bytes(pcl, pch);
                cycles
            }
            (Instruction::LSR, OpInput::UseImplied, cycles, _) => {
                let mut val = self.registers.a;
                CPU::shift_right_with_flags(&mut val, &mut self.registers.status);
                self.registers.a = val;
                cycles
            }
            (Instruction::LSR, OpInput::UseAddress(addr), cycles, _) => {
                let mut operand = self.get_byte(addr);
                CPU::shift_right_with_flags(&mut operand, &mut self.registers.status);
                self.set_byte(addr, operand);
                cycles
            }
            (Instruction::ASL, OpInput::UseImplied, cycles, _) => {
                let mut val = self.registers.a;
                CPU::shift_left_with_flags(&mut val, &mut self.registers.status);
                self.registers.a = val;
                cycles
            }
            (Instruction::ASL, OpInput::UseAddress(addr), cycles, _) => {
                let mut operand = self.get_byte(addr);
                CPU::shift_left_with_flags(&mut operand, &mut self.registers.status);
                self.set_byte(addr, operand);
                cycles
            }
            (Instruction::ROR, OpInput::UseImplied, cycles, _) => {
                let mut val = self.registers.a;
                CPU::rotate_right_with_flags(&mut val, &mut self.registers.status);
                self.registers.a = val;
                cycles
            }
            (Instruction::ROR, OpInput::UseAddress(addr), cycles, _) => {
                let mut operand = self.get_byte(addr);
                CPU::rotate_right_with_flags(&mut operand, &mut self.registers.status);
                self.set_byte(addr, operand);
                cycles
            }
            (Instruction::ROL, OpInput::UseImplied, cycles, _) => {
                let mut val = self.registers.a;
                CPU::rotate_left_with_flags(&mut val, &mut self.registers.status);
                self.registers.a = val;
                cycles
            }
            (Instruction::ROL, OpInput::UseAddress(addr), cycles, _) => {
                let mut operand = self.get_byte(addr);
                CPU::rotate_left_with_flags(&mut operand, &mut self.registers.status);
                self.set_byte(addr, operand);
                cycles
            }
            (Instruction::AND, OpInput::UseAddress(addr), cycles, extra_cycle) => {
                let val = self.get_byte(addr);
                self.and(val);
                cycles + if extra_cycle { 1 } else { 0 }
            }
            (Instruction::INC, OpInput::UseAddress(addr), cycles, _) => {
                let mut operand = self.get_byte(addr);
                CPU::increment(&mut operand, &mut self.registers.status);
                self.set_byte(addr, operand);
                cycles
            }
            (Instruction::DEC, OpInput::UseAddress(addr), cycles, _) => {
                let mut operand = self.get_byte(addr);
                CPU::decrement(&mut operand, &mut self.registers.status);
                self.set_byte(addr, operand);
                cycles
            }

            //
            // unofficial instructions
            //

            (Instruction::LAX, OpInput::UseAddress(_addr), cycles, _) => {
                // unofficial
                cycles
            }
            (Instruction::AAX, OpInput::UseAddress(_addr), cycles, _) => {
                // unofficial
                cycles
            }
            (Instruction::DCP, OpInput::UseAddress(_addr), cycles, _) => {
                // unofficial
                cycles
            }
            (Instruction::ISC, OpInput::UseAddress(_addr), cycles, _) => {
                // unofficial
                cycles
            }
            (Instruction::SLO, OpInput::UseAddress(_addr), cycles, _) => {
                // unofficial
                cycles
            }
            (Instruction::RLA, OpInput::UseAddress(_addr), cycles, _) => {
                // unofficial
                cycles
            }
            (Instruction::SRE, OpInput::UseAddress(_addr), cycles, _) => {
                // unofficial
                cycles
            }
            (Instruction::RRA, OpInput::UseAddress(_addr), cycles, _) => {
                // unofficial
                cycles
            }
            (Instruction::KIL, OpInput::UseImplied, cycles, _) => {
                // unofficial
                cycles
            }
            (Instruction::DOP, OpInput::UseAddress(_addr), cycles, _) => {
                // unofficial
                cycles
            }
            (Instruction::TOP, OpInput::UseAddress(_addr), cycles, _) => {
                // unofficial
                cycles
            }
            (Instruction::AAC, OpInput::UseAddress(_addr), cycles, _) => {
                // unofficial
                cycles
            }
            (Instruction::DOP, OpInput::UseImmediate(_val), cycles, _) => {
                // unofficial
                cycles
            }

            //
            // nop and errors
            //

            (Instruction::NOP, _, cycles, _) => {
                cycles
            }
            (_, _, _, _) => {
                panic!("can't execute {:?} {:?}", decoded_instr.0, decoded_instr.1);
            },
        }
    }

    pub fn clock(&mut self) {
        if self.cycles == 0 {
            let opcode = self.fetch_next_and_decode().unwrap();
            self.cycles = opcode.2;
            self.execute_instruction(opcode);
        }

        self.clock_count += 1;
        self.cycles -= 1;
    }

    pub fn complete(&self) -> bool {
        self.cycles == 0
    }

    pub fn run(&mut self) {
        while let Some(decoded_instr) = self.fetch_next_and_decode() {
            self.execute_instruction(decoded_instr);
            println!();
        }
    }

    const fn value_is_negative(value: u8) -> bool {
        value > 127
    }

    fn set_flags_from_u8(status: &mut Status, value: u8) {
        let is_zero = value == 0;
        let is_negative = Self::value_is_negative(value);

        status.set_with_mask(
            Status::PS_ZERO | Status::PS_NEGATIVE,
            Status::new(StatusArgs {
                z: is_zero,
                n: is_negative,
                ..StatusArgs::none()
            }),
        );
    }

    fn set_u8_with_flags(mem: &mut u8, status: &mut Status, value: u8) {
        *mem = value;
        CPU::set_flags_from_u8(status, value);
    }

    fn shift_right_with_flags(p_val: &mut u8, status: &mut Status) {
        let mask = 1;
        let is_bit_0_set = (*p_val & mask) == mask;
        *p_val >>= 1;
        status.set_with_mask(
            Status::PS_CARRY,
            Status::new(StatusArgs {
                c: is_bit_0_set,
                ..StatusArgs::none()
            })
        );
        CPU::set_flags_from_u8(status, *p_val);
    }

    fn shift_left_with_flags(p_val: &mut u8, status: &mut Status) {
        let mask = 1 << 7;
        let is_bit_7_set = (*p_val & mask) == mask;
        let shifted = (*p_val & !(1 << 7)) << 1;
        *p_val = shifted;
        status.set_with_mask(
            Status::PS_CARRY,
            Status::new(StatusArgs {
                c: is_bit_7_set,
                ..StatusArgs::none()
            })
        );
        CPU::set_flags_from_u8(status, *p_val);
    }

    fn rotate_right_with_flags(p_val: &mut u8, status: &mut Status) {
        let is_carry_set = status.contains(Status::PS_CARRY);
        let mask = 1;
        let is_bit_0_set = (*p_val & mask) == mask;
        let shifted = *p_val >> 1;
        *p_val = shifted + if is_carry_set { 1 << 7 } else { 0 };
        status.set_with_mask(
            Status::PS_CARRY,
            Status::new(StatusArgs {
                c: is_bit_0_set,
                ..StatusArgs::none()
            })
        );
        CPU::set_flags_from_u8(status, *p_val);
    }

    fn rotate_left_with_flags(p_val: &mut u8, status: &mut Status) {
        let is_carry_set = status.contains(Status::PS_CARRY);
        let mask = 1 << 7;
        let is_bit_7_set = (*p_val & mask) == mask;
        let shifted = (*p_val & !(1 << 7)) << 1;
        *p_val = shifted + u8::from(is_carry_set);
        status.set_with_mask(
            Status::PS_CARRY,
            Status::new(StatusArgs {
                c: is_bit_7_set,
                ..StatusArgs::none()
            })
        );
        CPU::set_flags_from_u8(status, *p_val);
    }

    fn load_a(&mut self, value: u8) {
        CPU::set_u8_with_flags(
            &mut self.registers.a,
            &mut self.registers.status,
            value,
        );
    }

    fn load_x(&mut self, value: u8) {
        CPU::set_u8_with_flags(
            &mut self.registers.x,
            &mut self.registers.status,
            value,
        );
    }

    fn load_y(&mut self, value: u8) {
        CPU::set_u8_with_flags(
            &mut self.registers.y,
            &mut self.registers.status,
            value,
        );
    }

    fn jump(&mut self, addr: u16) {
        self.registers.pc = addr;
    }

    fn branch_if_equal(&mut self, addr: u16) {
        if self.registers.status.contains(Status::PS_ZERO) {
            self.registers.pc = addr;
        }
    }

    fn branch_if_not_equal(&mut self, addr: u16) {
        if !self.registers.status.contains(Status::PS_ZERO) {
            self.registers.pc = addr;
        }
    }

    fn branch_if_minus(&mut self, addr: u16) {
        if self.registers.status.contains(Status::PS_NEGATIVE) {
            self.registers.pc = addr;
        }
    }

    fn branch_if_positive(&mut self, addr: u16) {
        if !self.registers.status.contains(Status::PS_NEGATIVE) {
            self.registers.pc = addr;
        }
    }

    fn branch_if_carry_set(&mut self, addr: u16) {
        if self.registers.status.contains(Status::PS_CARRY) {
            self.registers.pc = addr;
        }
    }

    fn branch_if_carry_clear(&mut self, addr: u16) {
        if !self.registers.status.contains(Status::PS_CARRY) {
            self.registers.pc = addr;
        }
    }

    fn branch_if_overflow_set(&mut self, addr: u16) {
        if self.registers.status.contains(Status::PS_OVERFLOW) {
            self.registers.pc = addr;
        }
    }

    fn branch_if_overflow_clear(&mut self, addr: u16) {
        if !self.registers.status.contains(Status::PS_OVERFLOW) {
            self.registers.pc = addr;
        }
    }

    fn add_with_carry(&mut self, value: u8) {
        const fn decimal_adjust(result: u8) -> u8 {
            let bcd1: u8 = if (result & 0x0F) > 0x09 { 0x06 } else { 0x00 };
            let bcd2: u8 = if (result.wrapping_add(bcd1) & 0xF0) > 0x90 {
                0x60
            } else {
                0x00
            };

            result.wrapping_add(bcd1).wrapping_add(bcd2)
        }

        let a_before: u8 = self.registers.a;
        let c_before: u8 = u8::from(self.registers.status.contains(Status::PS_CARRY));
        let a_after: u8 = a_before.wrapping_add(c_before).wrapping_add(value);

        let result: u8 = if self.registers.status.contains(Status::PS_DECIMAL_MODE) {
            decimal_adjust(a_after)
        } else {
            a_after
        };

        let did_carry = (result < a_before) || (a_after == 0 && c_before == 0x01) || (value == 0xFF && c_before == 0x01);
        let did_overflow = (a_before > 127 && value > 127 && a_after < 128) || (a_before < 128 && value < 128 && a_after > 127);

        let mask = Status::PS_CARRY | Status::PS_OVERFLOW;

        self.registers.status.set_with_mask(
            mask,
            Status::new(StatusArgs {
                c: did_carry,
                v: did_overflow,
                ..StatusArgs::none()
            }),
        );

        self.load_a(result);
    }

    fn subtract_with_carry(&mut self, value: u8) {
        let nc: u8 = u8::from(!self.registers.status.contains(Status::PS_CARRY));

        let a_before = self.registers.a;
        let a_after = a_before.wrapping_sub(value).wrapping_sub(nc);
        let over = (nc == 0 && value > 127) && a_before < 128 && a_after > 127;
        let under = (a_before > 127) && (0u8.wrapping_sub(value).wrapping_sub(nc) > 127) && a_after < 128;
        let did_overflow = over || under;

        let mask = Status::PS_CARRY | Status::PS_OVERFLOW;

        let bcd1: u8 = if (a_before & 0x0f).wrapping_sub(nc) < (value & 0x0f) {
            0x06
        } else {
            0x00
        };

        let bcd2: u8 = if (a_after.wrapping_sub(bcd1) & 0xf0) > 0x90 {
            0x60
        } else {
            0x00
        };

        let result: u8 = if self.registers.status.contains(Status::PS_DECIMAL_MODE) {
            a_after.wrapping_sub(bcd1).wrapping_sub(bcd2)
        } else {
            a_after
        };

        let did_carry = result > a_before;

        self.registers.status.set_with_mask(
            mask,
            Status::new(StatusArgs {
                c: did_carry,
                v: did_overflow,
                ..StatusArgs::none()
            }),
        );

        self.load_a(result);
    }

    fn push_on_stack(&mut self, val: u8) {
        let addr = self.registers.stkp.to_u16();
        self.set_byte(addr, val);
        self.registers.stkp.decrement();
    }

    fn pull_from_stack(&mut self) -> u8 {
        let addr = self.registers.stkp.to_u16();
        let out = self.get_byte(addr);
        self.registers.stkp.increment();
        out
    }

    fn fetch_from_stack(&mut self) -> u8 {
        self.get_byte(self.registers.stkp.to_u16())
    }

    fn xor(&mut self, val: u8) {
        let a_after = self.registers.a ^ val;
        self.load_a(a_after);
    }

    fn inclusive_or(&mut self, val: u8) {
        let a_after = self.registers.a | val;
        self.load_a(a_after);
    }

    fn exclusive_or(&mut self, val: u8) {
        let a_after = self.registers.a ^ val;
        self.load_a(a_after);
    }

    fn compare(&mut self, r: u8, val: u8) {
        if r >= val {
            self.registers.status.insert(Status::PS_CARRY);
        } else {
            self.registers.status.remove(Status::PS_CARRY);
        }

        if r == val {
            self.registers.status.insert(Status::PS_ZERO);
        } else {
            self.registers.status.remove(Status::PS_ZERO);
        }

        let diff = r.wrapping_sub(val);
        if Self::value_is_negative(diff) {
            self.registers.status.insert(Status::PS_NEGATIVE);
        } else {
            self.registers.status.remove(Status::PS_NEGATIVE);
        }
    }

    fn compare_with_a_register(&mut self, val: u8) {
        self.compare(self.registers.a, val);
    }

    fn compare_with_y_register(&mut self, val: u8) {
        self.compare(self.registers.y, val);
    }

    fn compare_with_x_register(&mut self, val: u8) {
        self.compare(self.registers.x, val);
    }

    fn and(&mut self, value: u8) {
        let a_after = self.registers.a & value;
        self.load_a(a_after);
    }

    fn increment(val: &mut u8, flags: &mut Status) {
        let value_new = val.wrapping_add(1);
        *val = value_new;

        let is_zero = value_new == 0;

        flags.set_with_mask(
            Status::PS_NEGATIVE | Status::PS_ZERO,
            Status::new(StatusArgs {
                n: Self::value_is_negative(value_new),
                z: is_zero,
                ..StatusArgs::none()
            })
        );
    }

    fn decrement(val: &mut u8, flags: &mut Status) {
        let value_new = val.wrapping_sub(1);
        *val = value_new;

        let is_zero = value_new == 0;
        let is_negative = Self::value_is_negative(value_new);

        flags.set_with_mask(
            Status::PS_NEGATIVE | Status::PS_ZERO,
            Status::new(StatusArgs {
                n: is_negative,
                z: is_zero,
                ..StatusArgs::none()
            })
        );
    }
}
