#[derive(Debug)]
pub enum Instruction {
    BRK, STA, JMP,
    ORA, LDX, LDY,
    LDA, STX, STY,
    SEC, EOR, ADC,
    SBC, NOP, PHA,
    BEQ, BMI, JSR,
    CMP, BNE, RTS,
    SEI, CLD, TXS,
    BPL, BCS, CLC,
    BCC, BIT, BVS,
    BVC, SED, PHP,
    PLA, AND, PLP,
    CLV, CPY, CPX,
    INY, INX, DEY,
    DEX,
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

#[derive(Default)]
pub struct Nmos6502;

impl Nmos6502 {
    pub fn decode(opcode: u8) -> Option<(Instruction, AddressingMode)> {
        match opcode {
            0x00 => Some((Instruction::BRK, AddressingMode::IMP)),
            0x01 => Some((Instruction::ORA, AddressingMode::IZX)),
            0x02 => None,
            0x03 => None,
            0x04 => None,
            0x05 => Some((Instruction::ORA, AddressingMode::ZP0)),
            0x07 => None,
            0x08 => Some((Instruction::PHP, AddressingMode::IMP)),
            0x09 => Some((Instruction::ORA, AddressingMode::IMM)),
            0x10 => Some((Instruction::BPL, AddressingMode::REL)),
            0x13 => None,
            0x18 => Some((Instruction::CLC, AddressingMode::IMP)),
            0x20 => Some((Instruction::JSR, AddressingMode::ABS)),
            0x24 => Some((Instruction::BIT, AddressingMode::ZP0)),
            0x28 => Some((Instruction::PLP, AddressingMode::IMP)),
            0x29 => Some((Instruction::AND, AddressingMode::IMM)),
            0x30 => Some((Instruction::BMI, AddressingMode::REL)),
            0x33 => None,
            0x38 => Some((Instruction::SEC, AddressingMode::IMP)),
            0x48 => Some((Instruction::PHA, AddressingMode::IMP)),
            0x49 => Some((Instruction::EOR, AddressingMode::IMM)),
            0x4C => Some((Instruction::JMP, AddressingMode::ABS)),
            0x50 => Some((Instruction::BVC, AddressingMode::REL)),
            0x60 => Some((Instruction::RTS, AddressingMode::IMP)),
            0x61 => Some((Instruction::ADC, AddressingMode::IZX)),
            0x65 => Some((Instruction::ADC, AddressingMode::ZP0)),
            0x68 => Some((Instruction::PLA, AddressingMode::IMP)),
            0x69 => Some((Instruction::ADC, AddressingMode::IMM)),
            0x6D => Some((Instruction::ADC, AddressingMode::ABS)),
            0x70 => Some((Instruction::BVS, AddressingMode::REL)),
            0x71 => Some((Instruction::ADC, AddressingMode::IZY)),
            0x75 => Some((Instruction::ADC, AddressingMode::ZPX)),
            0x78 => Some((Instruction::SEI, AddressingMode::IMP)),
            0x79 => Some((Instruction::ADC, AddressingMode::ABY)),
            0x7D => Some((Instruction::ADC, AddressingMode::ABX)),
            0x84 => Some((Instruction::STY, AddressingMode::ZP0)),
            0x85 => Some((Instruction::STA, AddressingMode::ZP0)),
            0x86 => Some((Instruction::STX, AddressingMode::ZP0)),
            0x88 => Some((Instruction::DEY, AddressingMode::IMP)),
            0x90 => Some((Instruction::BCC, AddressingMode::REL)),
            0x9A => Some((Instruction::TXS, AddressingMode::IMP)),
            0xA0 => Some((Instruction::LDY, AddressingMode::IMM)),
            0xA2 => Some((Instruction::LDX, AddressingMode::IMM)),
            0xA4 => Some((Instruction::LDY, AddressingMode::ZP0)),
            0xA5 => Some((Instruction::LDA, AddressingMode::ZP0)),
            0xA6 => Some((Instruction::LDX, AddressingMode::ZP0)),
            0xA9 => Some((Instruction::LDA, AddressingMode::IMM)),
            0xAD => Some((Instruction::LDA, AddressingMode::ABS)),
            0xB0 => Some((Instruction::BCS, AddressingMode::REL)),
            0xB8 => Some((Instruction::CLV, AddressingMode::IMP)),
            0xC0 => Some((Instruction::CPY, AddressingMode::IMM)),
            0xC8 => Some((Instruction::INY, AddressingMode::IMP)),
            0xC9 => Some((Instruction::CMP, AddressingMode::IMM)),
            0xCA => Some((Instruction::DEX, AddressingMode::IMP)),
            0xD0 => Some((Instruction::BNE, AddressingMode::REL)),
            0xD8 => Some((Instruction::CLD, AddressingMode::IMP)),
            0xE0 => Some((Instruction::CPX, AddressingMode::IMM)),
            0xE5 => Some((Instruction::SBC, AddressingMode::ZP0)),
            0xE8 => Some((Instruction::INX, AddressingMode::IMP)),
            0xE9 => Some((Instruction::SBC, AddressingMode::IMM)),
            0xEA => Some((Instruction::NOP, AddressingMode::IMP)),
            0xF0 => Some((Instruction::BEQ, AddressingMode::REL)),
            0xF8 => Some((Instruction::SED, AddressingMode::IMP)),
            0xFF => None,
            x => panic!("FAIL decode {:X}", x),
        }
    }
}
