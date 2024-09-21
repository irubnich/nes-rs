use crate::registers::{Registers, StackPointer, Status, StatusArgs};
use crate::instruction::{DecodedInstr, Nmos6502, AddressingMode, OpInput, Instruction};
use crate::bus::Bus;

fn address_from_bytes(lo: u8, hi: u8) -> u16 {
    u16::from(hi) << 8 | u16::from(lo)
}

pub struct CPU {
    pub registers: Registers,
    bus: Bus,
}

impl CPU {
    pub fn new(bus: Bus) -> CPU {
        CPU {
            registers: Registers::new(),
            bus,
        }
    }

    fn get_byte(&mut self, address: u16) -> u8 {
        self.bus.cpu_read(address)
    }

    fn set_byte(&mut self, address: u16, value: u8) {
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
    }

    fn read_address(&mut self, addr: u16) -> [u8; 2] {
            let lo = self.get_byte(addr);
            let hi = self.get_byte(addr.wrapping_add(1));
            [lo, hi]
    }

    pub fn fetch_next_and_decode(&mut self) -> Option<DecodedInstr> {
        let x: u8 = self.get_byte(self.registers.pc);

        println!("at PC: {:X}", self.registers.pc);
        println!("decoding opcode {:X}", x);
        match Nmos6502::decode(x) {
            Some((instr, am)) => {
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

                let am_out = match am {
                    AddressingMode::IMP => OpInput::UseImplied,
                    AddressingMode::IMM => OpInput::UseImmediate(slice[0]),
                    AddressingMode::ZP0 => OpInput::UseAddress(u16::from(slice[0])),
                    AddressingMode::ZPX => OpInput::UseAddress(u16::from(slice[0].wrapping_add(x))),
                    AddressingMode::ZPY => OpInput::UseAddress(u16::from(slice[0].wrapping_add(y))),
                    AddressingMode::REL => {
                        let offset = slice[0];
                        let sign_extend = if offset & 0x80 == 0x80 { 0xFFu8 } else { 0x0 };
                        let rel = u16::from_le_bytes([offset, sign_extend]);
                        OpInput::UseRelative(rel)
                    },
                    AddressingMode::ABS => OpInput::UseAddress(address_from_bytes(slice[0], slice[1])),
                    AddressingMode::ABX => OpInput::UseAddress(address_from_bytes(slice[0], slice[1]).wrapping_add(x.into())),
                    AddressingMode::ABY => OpInput::UseAddress(address_from_bytes(slice[0], slice[1]).wrapping_add(y.into())),
                    AddressingMode::IND => {
                        let slice = self.read_address(address_from_bytes(slice[0], slice[1]));
                        OpInput::UseAddress(address_from_bytes(slice[0], slice[1]))
                    },
                    AddressingMode::IZX => {
                        let start = slice[0].wrapping_add(x);
                        let slice = self.read_address(u16::from(start));
                        OpInput::UseAddress(address_from_bytes(slice[0], slice[1]))
                    }
                    AddressingMode::IZY => {
                        let start = slice[0];
                        let slice = self.read_address(u16::from(start));
                        OpInput::UseAddress(address_from_bytes(slice[0], slice[1]).wrapping_add(y.into()))
                    },
                };

                self.registers.pc = self.registers.pc.wrapping_add(num_bytes);

                Some((instr, am_out))
            }
            _ => None,
        }
    }

    pub fn execute_instruction(&mut self, decoded_instr: DecodedInstr) {
        match decoded_instr {
            (Instruction::LDA, OpInput::UseAddress(addr)) => {
                let val = self.get_byte(addr);
                self.load_a(val);
            },
            (Instruction::SEC, OpInput::UseImplied) => {
                self.registers.status.or(Status::PS_CARRY);
            },
            (Instruction::SBC, OpInput::UseAddress(addr)) => {
                let val = self.get_byte(addr);
                self.subtract_with_carry(val);
            }
            (Instruction::BEQ, OpInput::UseRelative(rel)) => {
                let addr = self.registers.pc.wrapping_add(rel);
                self.branch_if_equal(addr);
            },
            (Instruction::BMI, OpInput::UseRelative(rel)) => {
                let addr = self.registers.pc.wrapping_add(rel);
                self.branch_if_minus(addr)
            },
            (Instruction::STA, OpInput::UseAddress(addr)) => {
                self.set_byte(addr, self.registers.a);
            },
            (Instruction::JMP, OpInput::UseAddress(addr)) => {
                self.jump(addr);
            },
            (Instruction::LDX, OpInput::UseAddress(addr)) => {
                let val = self.get_byte(addr);
                self.load_x(val);
            },
            (Instruction::LDY, OpInput::UseAddress(addr)) => {
                let val = self.get_byte(addr);
                self.load_y(val);
            },
            (Instruction::STX, OpInput::UseAddress(addr)) => {
                self.set_byte(addr, self.registers.x);
            },
            (Instruction::STY, OpInput::UseAddress(addr)) => {
                self.set_byte(addr, self.registers.y);
            },
            (Instruction::BRK, OpInput::UseImplied) => {
                // for b in self.registers.pc.wrapping_sub(1).to_be_bytes() {
                //     self.push_on_stack(b);
                // }
                // self.push_on_stack(self.registers.status.bits());
                // let pcl = self.get_byte(0xFFFE);
                // let pch = self.get_byte(0xFFFF);
                // self.jump((u16::from(pch) << 8) | u16::from(pcl));
                // self.registers.status.or(Status::PS_DISABLE_INTERRUPTS);
                // self.registers.pc += 1;
                // self.registers.status.insert(Status::PS_DISABLE_INTERRUPTS);
                // self.set_byte(0x0100 + self.registers.stkp.to_u16(), (self.registers.pc >> 8) & 0x00FF);

                self.registers.pc += 1;
                let hilo = self.registers.pc.to_be_bytes();
                self.push_on_stack(hilo[0]);
                self.push_on_stack(hilo[1]);
                self.registers.status.insert(Status::PS_BRK);
                self.push_on_stack(self.registers.status.bits());
                self.registers.status.insert(Status::PS_DISABLE_INTERRUPTS);
                self.registers.pc = u16::from_be_bytes([self.get_byte(0xFFFE), self.get_byte(0xFFFF)])

            },
            (Instruction::EOR, OpInput::UseAddress(addr)) => {
                let val = self.get_byte(addr);
                self.xor(val);
            }
            (Instruction::ORA, OpInput::UseAddress(addr)) => {
                let val = self.get_byte(addr);
                self.inclusive_or(val);
            }
            (Instruction::LDA, OpInput::UseImmediate(val)) => {
                self.load_a(val);
            }
            (Instruction::ADC, OpInput::UseImmediate(val)) => {
                self.add_with_carry(val);
            }
            (Instruction::ADC, OpInput::UseAddress(addr)) => {
                let val = self.get_byte(addr);
                self.add_with_carry(val);
            }
            (Instruction::LDX, OpInput::UseImmediate(val)) => {
                self.load_x(val);
            }
            (Instruction::LDY, OpInput::UseImmediate(val)) => {
                self.load_y(val);
            }
            (Instruction::PHA, OpInput::UseImplied) => {
                self.push_on_stack(self.registers.a);
            }
            (Instruction::JSR, OpInput::UseAddress(addr)) => {
                for b in self.registers.pc.wrapping_sub(1).to_be_bytes() {
                    self.push_on_stack(b);
                }
                self.jump(addr);
            }
            (Instruction::CMP, OpInput::UseImmediate(val)) => {
                self.compare_with_a_register(val);
            }
            (Instruction::BNE, OpInput::UseRelative(rel)) => {
                let addr = self.registers.pc.wrapping_add(rel);
                self.branch_if_not_equal(addr);
            }
            (Instruction::RTS, OpInput::UseImplied) => {
                self.pull_from_stack();
                let pcl: u8 = self.pull_from_stack();
                let pch: u8 = self.fetch_from_stack();
                self.registers.pc = ((u16::from(pch) << 8) | u16::from(pcl)).wrapping_add(1);
            }

            (Instruction::NOP, OpInput::UseImplied) => {
                // noop
            }
            (_, _) => {
                panic!("can't execute {:?}", decoded_instr);
            },
        }
    }

    pub fn single_step(&mut self) {
        if let Some(decoded_instr) = self.fetch_next_and_decode() {
            self.execute_instruction(decoded_instr);
        }
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
                Z: is_zero,
                N: is_negative,
                ..StatusArgs::none()
            }),
        );
    }

    fn set_u8_with_flags(mem: &mut u8, status: &mut Status, value: u8) {
        *mem = value;
        CPU::set_flags_from_u8(status, value);
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
                C: did_carry,
                V: did_overflow,
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
                C: did_carry,
                V: did_overflow,
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
}
