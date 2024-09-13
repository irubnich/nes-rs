use rs6502::memory::{Bus, Memory};
use rs6502::instruction::Nmos6502;

mod cpu;

fn main() {
    println!("enter 2 numbers < 128 separated by a space to know their GCD");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();

    let zero_page_data = input
        .split_whitespace()
        .map(|s| s.parse::<u8>().unwrap())
        .collect::<Vec<u8>>();

    let program = [
        0xA5, 0x00, 0xFF, // load input into A
    ];

    let mut cpu = cpu::CPU::new(Memory::new(), Nmos6502);

    cpu.memory.set_bytes(0x00, &zero_page_data);
    cpu.memory.set_bytes(0x10, &program);
    cpu.registers.pc = 0x10;

    cpu.run();

    println!("A: {}", cpu.registers.a);
}
