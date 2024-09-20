pub mod instruction;
pub mod memory;
pub mod registers;
pub mod cartridge;
pub mod mapper;
pub mod bus;
pub mod cpu;

pub trait Variant {
    fn decode(opcode: u8) -> Option<(
        crate::instruction::Instruction,
        crate::instruction::AddressingMode,
    )>;
}
