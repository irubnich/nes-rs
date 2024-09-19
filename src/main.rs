use std::fs::read;

use rs6502::memory::{Bus, Memory};
use rs6502::instruction::Nmos6502;

mod cpu;

fn main() {
    let zero_page_data = vec![56, 49];

    let program = match read("euclid.bin") {
        Ok(data) => data,
        Err(err) => panic!("{}", err)
    };

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
