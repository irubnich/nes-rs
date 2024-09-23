extern crate olc_pixel_game_engine;

use rs6502::bus::Bus;
use rs6502::cartridge::Cartridge;
use rs6502::cpu::CPU;
use rs6502::registers::Status;
use crate::olc_pixel_game_engine as olc;

struct Emulator {
    cpu: CPU
}
impl Emulator {
    fn draw_cpu(&self, x: i32, y: i32) {
        olc::draw_string(x, y, "STATUS", olc::WHITE).unwrap();
        olc::draw_string(x + 80, y, "N", self.get_color(Status::PS_NEGATIVE)).unwrap();
        olc::draw_string(x + 64, y, "V", self.get_color(Status::PS_OVERFLOW)).unwrap();
        olc::draw_string(x + 96, y, "-", self.get_color(Status::PS_UNUSED)).unwrap();
        olc::draw_string(x + 112, y, "B", self.get_color(Status::PS_BRK)).unwrap();
        olc::draw_string(x + 128, y, "D", self.get_color(Status::PS_DECIMAL_MODE)).unwrap();
        olc::draw_string(x + 144, y, "I", self.get_color(Status::PS_DISABLE_INTERRUPTS)).unwrap();
        olc::draw_string(x + 160, y, "Z", self.get_color(Status::PS_ZERO)).unwrap();
        olc::draw_string(x + 178, y, "C", self.get_color(Status::PS_CARRY)).unwrap();

        olc::draw_string(x, y + 10, format!("PC: ${:4X}", self.cpu.registers.pc).as_str(), olc::WHITE).unwrap();
        olc::draw_string(x, y + 20, format!("A: ${:4X}", self.cpu.registers.a).as_str(), olc::WHITE).unwrap();
        olc::draw_string(x, y + 30, format!("X: ${:4X}", self.cpu.registers.x).as_str(), olc::WHITE).unwrap();
        olc::draw_string(x, y + 40, format!("Y: ${:4X}", self.cpu.registers.y).as_str(), olc::WHITE).unwrap();
        olc::draw_string(x, y + 50, format!("SP: ${:4X}", self.cpu.registers.stkp.0).as_str(), olc::WHITE).unwrap();
    }

    pub fn get_color(&self, s: Status) -> olc::Pixel {
        if self.cpu.registers.status.contains(s) {
            olc::GREEN
        } else {
            olc::RED
        }
    }
}

impl olc::Application for Emulator {
    fn on_user_create(&mut self) -> Result<(), olc_pixel_game_engine::Error> {
        self.cpu.reset();
        self.cpu.registers.pc = 0xC000;

        Ok(())
    }

    fn on_user_update(&mut self, elapsed_time: f32) -> Result<(), olc_pixel_game_engine::Error> {
        olc::clear(olc::BLACK);

        //cpu.run();

        self.draw_cpu(516, 2);

        Ok(())
    }

    fn on_user_destroy(&mut self) -> Result<(), olc_pixel_game_engine::Error> {
        Ok(())
    }
}

fn main() {
    let cart = Cartridge::new(String::from("nestest.nes"));
    let bus = Bus::new(cart);
    let cpu = CPU::new(bus);

    let mut emulator = Emulator {
        cpu,
    };
    olc::start("nes", &mut emulator, 780, 480, 2, 2).unwrap();
}
