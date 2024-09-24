use rs6502::bus::Bus;
use rs6502::cartridge::Cartridge;
use rs6502::cpu::CPU;
use rs6502::memory::Memory;
use rs6502::ppu::PPU;
use rs6502::registers::{Registers, Status};
use olc_pixel_game_engine as olc;

struct Emulator {
    cpu: CPU,
    ppu: PPU,
    emulation_run: bool,
    residual_time: f32,
    system_clock_counter: i32,
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

        olc::draw_string(x, y + 10, format!("PC: ${:04X}", self.cpu.registers.pc).as_str(), olc::WHITE).unwrap();
        olc::draw_string(x, y + 20, format!("A:  ${:04X}", self.cpu.registers.a).as_str(), olc::WHITE).unwrap();
        olc::draw_string(x, y + 30, format!("X:  ${:04X}", self.cpu.registers.x).as_str(), olc::WHITE).unwrap();
        olc::draw_string(x, y + 40, format!("Y:  ${:04X}", self.cpu.registers.y).as_str(), olc::WHITE).unwrap();
        olc::draw_string(x, y + 50, format!("SP: ${:04X}", self.cpu.registers.stkp.0).as_str(), olc::WHITE).unwrap();
        olc::draw_string(x, y + 60, format!("CY: {}", self.cpu.clock_count).as_str(), olc::WHITE).unwrap();
    }

    fn draw_ram(&mut self, x: i32, y: i32, addr: &mut u16, rows: i32, cols: i32) {
        let ram_x = x;
        let mut ram_y = y;
        for _ in 0..rows {
            let mut offset = format!("${:04X}:", addr);
            for _ in 0..cols {
                offset = format!("{} {:02X}", offset, self.cpu.get_byte(*addr));
                *addr = *addr + 1;
            }
            olc::draw_string(ram_x, ram_y, &offset, olc::WHITE).unwrap();
            ram_y += 10;
        }
    }

    pub fn get_color(&self, s: Status) -> olc::Pixel {
        if self.cpu.registers.status.contains(s) {
            olc::GREEN
        } else {
            olc::RED
        }
    }

    pub fn clock(&mut self) {
        self.ppu.clock();

        if self.system_clock_counter % 3 == 0 {
            self.cpu.clock();
        }

        self.system_clock_counter += 1;
    }

    pub fn reset(&mut self) {
        self.cpu.reset();
        self.system_clock_counter = 0;
    }
}

impl olc::Application for Emulator {
    fn on_user_create(&mut self) -> Result<(), olc::Error> {
        self.reset();
        self.cpu.registers.pc = 0xC000;

        Ok(())
    }

    fn on_user_update(&mut self, elapsed_time: f32) -> Result<(), olc::Error> {
        olc::clear(olc::BLUE);

        if self.emulation_run {
            if self.residual_time > 0.0 {
                self.residual_time -= elapsed_time;
            } else {
                self.residual_time += (1.0 / 60.0) - elapsed_time;
                loop {
                    self.clock();
                    if self.ppu.frame_complete {
                        break;
                    }
                }
                self.ppu.frame_complete = false;
            }
        } else {
            if olc::get_key(olc::Key::C).pressed {
                loop {
                    self.clock();
                    if self.cpu.complete() {
                        break;
                    }
                }
                loop {
                    self.clock();
                    if !self.cpu.complete() {
                        break;
                    }
                }
            }
            if olc::get_key(olc::Key::F).pressed {
                loop {
                    self.clock();
                    if self.ppu.frame_complete {
                        break;
                    }
                }

                loop {
                    self.clock();
                    if self.cpu.complete() {
                        break;
                    }
                }

                self.ppu.frame_complete = false;
            }
        }

        if olc::get_key(olc::Key::SPACE).pressed { self.emulation_run = !self.emulation_run }
        if olc::get_key(olc::Key::R).pressed { self.reset(); }

        self.draw_cpu(516, 2);
        self.draw_ram(516, 100, &mut 0x0000, 16, 16);
        self.draw_ram(516, 300, &mut 0x8000, 16, 16);
        olc::draw_sprite_ext(0, 0, &self.ppu.spr_screen, 2, olc_pixel_game_engine::SpriteFlip::NONE);

        Ok(())
    }

    fn on_user_destroy(&mut self) -> Result<(), olc::Error> {
        Ok(())
    }
}

fn main() {
    let ppu = PPU::new();
    let cartridge = Cartridge::new(String::from("nestest.nes"));
    let bus = Bus {
        cartridge,
        memory: Memory::new()
    };
    let cpu = CPU {
        registers: Registers::new(),
        bus,
        cycles: 0,
        clock_count: 0,
    };

    let mut emulator = Emulator {
        cpu,
        ppu,
        emulation_run: false,
        residual_time: 0f32,
        system_clock_counter: 0,
    };
    olc::start("nes", &mut emulator, 780, 480, 2, 2).unwrap();
}
