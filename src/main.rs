use iced::widget::text;
use iced::Element;
use rs6502::bus::Bus;
use rs6502::cartridge::Cartridge;
use rs6502::cpu::CPU;

fn main() {// -> iced::Result {
    let cart = Cartridge::new(String::from("nestest.nes"));
    let bus = Bus::new(cart);
    let mut cpu = CPU::new(bus);

    // cpu.memory.set_bytes(0x0000, &zero_page_data);
    // cpu.memory.set_bytes(0xE000, &[0]);
    // cpu.memory.set_bytes(0x8000, &data);
    // cpu.registers.pc = 0xE000;

    //let r = iced::run("title", update, view);

    cpu.reset();
    cpu.run();

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

fn update(cpu: &mut CPU, message: Message) {
    cpu.run();
}

fn view(cpu: &CPU) -> Element<Message> {
    text(cpu.registers.x)
        .size(20)
        .into()
}

#[derive(Debug, Clone)]
enum Message {
}
