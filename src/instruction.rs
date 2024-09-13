pub enum Instruction {

}

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

pub struct Nmos6502;

impl crate::Variant for Nmos6502 {
    fn decode(opcode: u8) -> Option<(
            Instruction,
            AddressingMode,
        )> {
            match opcode {
                _ => None,
            } 
    }
}
