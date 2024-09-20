use rs6502::cpu::CPU;
use iced::widget::text;
use iced::Element;
use rs6502::bus::Bus;
use rs6502::cartridge::Cartridge;
use rs6502::memory::Memory;
use rs6502::instruction::Nmos6502;

mod cpu;

fn main() {// -> iced::Result {
    let mut cpu = cpu::CPU::new(Memory::new(), Nmos6502);
    let cart = Cartridge::new(String::from("01-basics.nes"));
    let bus = Bus::new(cpu, cart);

    // cpu.memory.set_bytes(0x0000, &zero_page_data);
    // cpu.memory.set_bytes(0xE000, &[0]);
    // cpu.memory.set_bytes(0x8000, &data);
    // cpu.registers.pc = 0xE000;

    //let r = iced::run("title", update, view);

    cpu.reset();
    //cpu.run();

    println!("=============================================");
    //println!("ZP: [{}, {}]", zero_page_data[0], zero_page_data[1]);
    println!("A: 0x{:X}", cpu.registers.a);
    println!("X: 0x{:X}", cpu.registers.x);
    println!("Y: 0x{:X}", cpu.registers.y);

    println!("PC: 0x{:X}", cpu.registers.pc);
    println!("SP: {:?}", cpu.registers.stkp);
    println!("Status: {:?}", cpu.registers.status);

    // r
}

fn update(cpu: &mut CPU<Memory, Nmos6502>, message: Message) {
    cpu.run();
}

fn view(cpu: &CPU<Memory, Nmos6502>) -> Element<Message> {
    text(cpu.registers.x)
        .size(20)
        .into()
}

#[derive(Debug, Clone)]
enum Message {
}
