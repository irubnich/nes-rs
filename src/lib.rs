pub mod instruction;
pub mod memory;
pub mod registers;

pub trait Variant {
    fn decode(opcode: u8) -> Option<(
        crate::instruction::Instruction,
        crate::instruction::AddressingMode,
    )>;
}
