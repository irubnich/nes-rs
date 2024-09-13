#[derive(Debug)]
pub enum Instruction {
    BRK,
    ORA,
    LDA,
}

#[derive(Debug)]
pub enum OpInput {
    UseImplied,
    UseImmediate(u8),
    UseRelative(u16),
    UseAddress(u16),
}

#[derive(Copy, Clone)]
pub enum AddressingMode {
    IMP,
    IMM,
    ZP0,
    ZPX,
    ZPY,
    REL,
    ABS,
    ABX,
    ABY,
    IND,
    IZX,
    IZY,
}

impl AddressingMode {
    pub const fn extra_bytes(self) -> u16 {
        match self {
            AddressingMode::IMP => 0,
            AddressingMode::IMM => 1,
            AddressingMode::ZP0 => 1,
            AddressingMode::ZPX => 1,
            AddressingMode::ZPY => 1,
            AddressingMode::REL => 1,
            AddressingMode::ABS => 2,
            AddressingMode::ABX => 2,
            AddressingMode::ABY => 2,
            AddressingMode::IND => 2,
            AddressingMode::IZX => 1,
            AddressingMode::IZY => 1,
        }
    }
}

pub type DecodedInstr = (Instruction, OpInput);

pub struct Nmos6502;

impl crate::Variant for Nmos6502 {
    fn decode(opcode: u8) -> Option<(Instruction, AddressingMode)> {
        match opcode {
            0x00 => Some((Instruction::BRK, AddressingMode::IMP)),
            0x01 => Some((Instruction::ORA, AddressingMode::IZX)),
            0x02 => None,
            0x03 => None,
            0x04 => None,
            0x05 => Some((Instruction::ORA, AddressingMode::ZP0)),
            0xA5 => Some((Instruction::LDA, AddressingMode::ZP0)),
            0xFF => None,
            _ => None,
        }
    }
}
