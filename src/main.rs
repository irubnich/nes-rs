use rs6502::memory::Memory;
use rs6502::instruction::Nmos6502;

mod cpu;

fn main() {
    let cpu = cpu::CPU::new(Memory::new(), Nmos6502);
    println!("Hello, world!");
}
