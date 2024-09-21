use rs6502::bus::Bus;
use rs6502::cartridge::Cartridge;
use rs6502::cpu::CPU;

fn main() {
    let cart = Cartridge::new(String::from("nestest.nes"));
    let bus = Bus::new(cart);
    let mut cpu = CPU::new(bus);

    cpu.reset();
    cpu.registers.pc = 0xC000;
    cpu.run();

    println!("=============================================");
    //println!("ZP: [{}, {}]", zero_page_data[0], zero_page_data[1]);
    println!("A: 0x{:X}", cpu.registers.a);
    println!("X: 0x{:X}", cpu.registers.x);
    println!("Y: 0x{:X}", cpu.registers.y);

    println!("PC: 0x{:X}", cpu.registers.pc);
    println!("SP: {:X}", cpu.registers.stkp.0);
    println!("Status: {:?}", cpu.registers.status);

    //let r = iced::run("title", update, view);
    // r
}
