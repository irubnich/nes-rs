use rs6502::memory::{Bus, Memory};
use rs6502::instruction::Nmos6502;

mod cpu;

fn main() {
    let zero_page_data = vec![12, 30];

    let program = [
        // .algo
        0xA5, 0x00, // load first into A
        0x38, // set carry flag
        0xE5, 0x01, // A = A - second
        0xF0, 0x07, // jump to .end if diff = 0
        0x30, 0x08, // jump to .swap if diff < 0
        0x85, 0x00, // first = A
        0x4C, 0x12, 0x00, // jump to 2
        // .end
        0xA5, 0x00, // A = second
        0xFF,
        // .swap
        0xA6, 0x00, // X = first
        0xA4, 0x01, // Y = second
        0x86, 0x01, // first = X
        0x84, 0x00, // second = Y
        0x4C, 0x10, 0x00, // jump to .algo
    ];

    let mut cpu = cpu::CPU::new(Memory::new(), Nmos6502);

    cpu.memory.set_bytes(0x00, &zero_page_data);
    cpu.memory.set_bytes(0x10, &program);
    cpu.registers.pc = 0x10;

    cpu.run();

    println!("=============================================");
    println!("ZP: [{}, {}]", zero_page_data[0], zero_page_data[1]);
    println!("A: 0x{:X}", cpu.registers.a);
    println!("X: 0x{:X}", cpu.registers.x);
    println!("Y: 0x{:X}", cpu.registers.y);

    println!("PC: 0x{:X}", cpu.registers.pc);
    println!("SP: {:?}", cpu.registers.stkp);
    println!("Status: {:?}", cpu.registers.status);
}
