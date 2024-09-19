use std::fs::read;

use rs6502::memory::{Bus, Memory};
use rs6502::instruction::Nmos6502;

mod cpu;

fn main() {
    let zero_page_data = vec![
        0x00, 0x02, // ADC ZeroPage target
        0x00, 0x04, // ADC ZeroPageX target
        0x00, 0x00, 0x00, 0x00, 0x10, // ADC IndexedIndirectX address
        0x80, // ADC IndexedIndirectX address
        0x00, 0x00, 0x00, 0x00, 0x00, 0x08, // ADC IndirectIndexedY address
        0x80, // ADC IndirectIndexedY address
    ];

    // let program = match read("6502_functional_test.bin") {
    //     Ok(data) => data,
    //     Err(err) => panic!("{}", err)
    // };

    let program = [
        0xA9, // LDA Immediate
        0x01, //     Immediate operand
        0x69, // ADC Immediate
        0x07, //     Immediate operand
        0x65, // ADC ZeroPage
        0x01, //     ZeroPage operand
        0xA2, // LDX Immediate
        0x01, //     Immediate operand
        0x75, // ADC ZeroPageX
        0x02, //     ZeroPageX operand
        0x6D, // ADC Absolute
        0x01, //     Absolute operand
        0x80, //     Absolute operand
        0xA2, // LDX immediate
        0x08, //     Immediate operand
        0x7D, // ADC AbsoluteX
        0x00, //     AbsoluteX operand
        0x80, //     AbsoluteX operand
        0xA0, // LDY immediate
        0x04, //     Immediate operand
        0x79, // ADC AbsoluteY
        0x00, //     AbsoluteY operand
        0x80, //     AbsoluteY operand
        0xA2, // LDX immediate
        0x05, //     Immediate operand
        0x61, // ADC IndexedIndirectX
        0x03, //     IndexedIndirectX operand
        0xA0, // LDY immediate
        0x10, //     Immediate operand
        0x71, // ADC IndirectIndexedY
        0x0F, //     IndirectIndexedY operand
        0xEA, // NOP :)
        0xFF, // Something invalid -- the end!
    ];

    let data = [
        0x00, 0x09, // ADC Absolute target
        0x00, 0x00, 0x40, // ADC AbsoluteY target
        0x00, 0x00, 0x00, 0x11, // ADC AbsoluteX target
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x12, // ADC IndexedIndirectX target
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x06, // ADC IndirectIndexedY target
    ];

    let mut cpu = cpu::CPU::new(Memory::new(), Nmos6502);

    cpu.memory.set_bytes(0x0000, &zero_page_data);
    cpu.memory.set_bytes(0x4000, &program);
    cpu.memory.set_bytes(0x8000, &data);
    cpu.registers.pc = 0x4000;

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
