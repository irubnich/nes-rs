use rs6502::Variant;
use rs6502::memory::Bus;
use rs6502::registers::{Registers, Status, StatusArgs};
use rs6502::instruction::{AddressingMode, DecodedInstr, Instruction, OpInput};

fn address_from_bytes(lo: u8, hi: u8) -> u16 {
    u16::from(lo) + (u16::from(hi) << 8usize)
}

pub struct CPU<M, V>
where
    M: Bus,
    V: Variant,
{
    pub registers: Registers,
    pub memory: M,
    variant: core::marker::PhantomData<V>,
}

impl<M: Bus, V: Variant> CPU<M, V> {
    pub fn new(memory: M, _variant: V) -> CPU<M, V> {
        CPU {
            registers: Registers::new(),
            memory,
            variant: core::marker::PhantomData::<V>,
        }
    }

    pub fn reset(&mut self) {
        // noop
    }

    pub fn fetch_next_and_decode(&mut self) -> Option<DecodedInstr> {
        fn read_address<M: Bus>(mem: &mut M, addr: u16) -> [u8; 2] {
            let lo = mem.get_byte(addr);
            let hi = mem.get_byte(addr.wrapping_add(1));
            [lo, hi]
        }
        
        let x: u8 = self.memory.get_byte(self.registers.pc);

        println!("at PC: {:X}", self.registers.pc);
        println!("decoding opcode {:X}", x);
        match V::decode(x) {
            Some((instr, am)) => {
                let extra_bytes = am.extra_bytes();
                let num_bytes = extra_bytes + 1;

                let data_start = self.registers.pc.wrapping_add(1);

                let slice = if extra_bytes == 0 {
                    [0, 0]
                } else if extra_bytes == 1 {
                    [self.memory.get_byte(data_start), 0]
                } else if extra_bytes == 2 {
                    [
                        self.memory.get_byte(data_start),
                        self.memory.get_byte(data_start.wrapping_add(1)),
                    ]
                } else {
                    panic!()
                };

                let x = self.registers.x;
                let y = self.registers.y;

                let memory = &mut self.memory;

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
                        let slice = read_address(memory, address_from_bytes(slice[0], slice[1]));
                        OpInput::UseAddress(address_from_bytes(slice[0], slice[1]))
                    },
                    AddressingMode::IZX => {
                        let start = slice[0].wrapping_add(x);
                        let slice = read_address(memory, u16::from(start));
                        OpInput::UseAddress(address_from_bytes(slice[0], slice[1]))
                    }
                    AddressingMode::IZY => {
                        let start = slice[0];
                        let slice = read_address(memory, u16::from(start));
                        OpInput::UseAddress(address_from_bytes(slice[0], slice[1]).wrapping_add(y.into()))
                    },
                };

                println!("old PC: {:X}", self.registers.pc);
                self.registers.pc = self.registers.pc.wrapping_add(num_bytes);
                println!("new PC: {:X}", self.registers.pc);

                Some((instr, am_out))
            }
            _ => None,
        }
    }

    pub fn execute_instruction(&mut self, decoded_instr: DecodedInstr) {
        match decoded_instr {
            (Instruction::LDA, OpInput::UseAddress(addr)) => {
                let val = self.memory.get_byte(addr);
                println!("LDA addr: {:?} value: {}", addr, val);
                self.load_a(val);
            },
            (Instruction::SEC, OpInput::UseImplied) => {
                self.registers.status.or(Status::PS_CARRY);
            },
            (Instruction::SBC, OpInput::UseAddress(addr)) => {
                let val = self.memory.get_byte(addr);
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
                self.memory.set_byte(addr, self.registers.a);
            },
            (Instruction::JMP, OpInput::UseAddress(addr)) => {
                self.jump(addr);
            },
            (Instruction::LDX, OpInput::UseAddress(addr)) => {
                let val = self.memory.get_byte(addr);
                self.load_x(val);
            },
            (Instruction::LDY, OpInput::UseAddress(addr)) => {
                let val = self.memory.get_byte(addr);
                self.load_y(val);
            },
            (Instruction::STX, OpInput::UseAddress(addr)) => {
                self.memory.set_byte(addr, self.registers.x);
            },
            (Instruction::STY, OpInput::UseAddress(addr)) => {
                self.memory.set_byte(addr, self.registers.y);
            },
            (_, _) => {
                println!("can't execute {:?}", decoded_instr)
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
        CPU::<M, V>::set_flags_from_u8(status, value);
    }

    fn load_a(&mut self, value: u8) {
        CPU::<M, V>::set_u8_with_flags(
            &mut self.registers.a,
            &mut self.registers.status,
            value,
        );
        println!("LOAD A {}", value)
    }

    fn load_x(&mut self, value: u8) {
        CPU::<M, V>::set_u8_with_flags(
            &mut self.registers.x,
            &mut self.registers.status,
            value,
        );
        println!("LOAD A {}", value)
    }

    fn load_y(&mut self, value: u8) {
        CPU::<M, V>::set_u8_with_flags(
            &mut self.registers.y,
            &mut self.registers.status,
            value,
        );
        println!("LOAD A {}", value)
    }

    fn jump(&mut self, addr: u16) {
        self.registers.pc = addr;
    }

    fn branch_if_equal(&mut self, addr: u16) {
        if self.registers.status.contains(Status::PS_ZERO) {
            self.registers.pc = addr;
        }
    }

    fn branch_if_minus(&mut self, addr: u16) {
        if self.registers.status.contains(Status::PS_NEGATIVE) {
            self.registers.pc = addr;
        }
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
}
