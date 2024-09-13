use rs6502::Variant;
use rs6502::memory::Bus;
use rs6502::registers::Registers;
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

                self.registers.pc = self.registers.pc.wrapping_add(num_bytes);

                Some((instr, am_out))
            }
            _ => None,
        }
    }

    pub fn execute_instruction(&mut self, decoded_instr: DecodedInstr) {
        match decoded_instr {
            (Instruction::BRK, OpInput::UseImmediate(val)) => {

            },
            _ => panic!()
        }
    }
}
