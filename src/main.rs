use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use rs6502::bus::Bus;
use rs6502::cartridge::Cartridge;
use rs6502::cpu::CPU;
use rs6502::memory::Memory;
use rs6502::ppu::PPU;
use rs6502::registers::{Registers, Status};
use olc_pixel_game_engine as olc;

struct Emulator {
    cpu: CPU,
    ppu: Rc<RefCell<PPU>>,
    emulation_run: bool,
    residual_time: f32,
    system_clock_counter: i32,
    selected_palette: u8,
    map_asm: HashMap<u16, String>,
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
        olc::draw_string(x, y + 20, format!("A:  ${:02X}", self.cpu.registers.a).as_str(), olc::WHITE).unwrap();
        olc::draw_string(x, y + 30, format!("X:  ${:02X}", self.cpu.registers.x).as_str(), olc::WHITE).unwrap();
        olc::draw_string(x, y + 40, format!("Y:  ${:02X}", self.cpu.registers.y).as_str(), olc::WHITE).unwrap();
        olc::draw_string(x, y + 50, format!("SP: ${:02X}", self.cpu.registers.stkp.0).as_str(), olc::WHITE).unwrap();
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

    fn draw_code(&self, x: i32, y: i32, lines: i32) {
        let mut pc = self.cpu.registers.pc.clone();
        let mut line_y = (lines >> 1) * 10 + y;

        match self.map_asm.get(&pc) {
            Some(line) => {
                olc::draw_string(x, line_y, line, olc::CYAN).unwrap();
            }
            None => ()
        }

        while line_y < (lines * 10) + y {
            pc = pc.wrapping_add(1);

            match self.map_asm.get(&pc) {
                Some(line) => {
                    line_y += 10;
                    olc::draw_string(x, line_y, line, olc::WHITE).unwrap();
                }
                None => ()
            }
        }

        pc = self.cpu.registers.pc.clone();
        line_y = (lines >> 1) * 10 + y;
        while line_y > y {
            pc = pc.wrapping_sub(1);

            match self.map_asm.get(&pc) {
                Some(line) => {
                    line_y -= 10;
                    olc::draw_string(x, line_y, line, olc::WHITE).unwrap();
                }
                None => ()
            }
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
        self.ppu.borrow_mut().clock();

        if self.system_clock_counter % 3 == 0 {
            self.cpu.clock();
        }

        self.system_clock_counter += 1;
    }

    pub fn reset(&mut self) {
        self.cpu.reset();
        self.ppu.borrow_mut().reset();
        self.system_clock_counter = 0;
    }
}

impl olc::Application for Emulator {
    fn on_user_create(&mut self) -> Result<(), olc::Error> {
        self.map_asm = self.cpu.disassemble(0x0000, 0xFFFF);

        self.reset();
        //self.cpu.registers.pc = 0xC000;

        //println!("{}", self.ppu.borrow().get_pattern_table(0).get_pixel(100, 100));

        Ok(())
    }

    fn on_user_update(&mut self, elapsed_time: f32) -> Result<(), olc::Error> {
        olc::clear(olc::DARK_BLUE);

        if self.emulation_run {
            if self.residual_time > 0.0 {
                self.residual_time -= elapsed_time;
            } else {
                self.residual_time += (1.0 / 60.0) - elapsed_time;
                loop {
                    self.clock();
                    if self.ppu.borrow().frame_complete {
                        break;
                    }
                }
                self.ppu.borrow_mut().frame_complete = false;
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
                    if self.ppu.borrow().frame_complete {
                        break;
                    }
                }

                loop {
                    self.clock();
                    if self.cpu.complete() {
                        break;
                    }
                }

                self.ppu.borrow_mut().frame_complete = false;
            }
        }

        if olc::get_key(olc::Key::SPACE).pressed { self.emulation_run = !self.emulation_run }
        if olc::get_key(olc::Key::R).pressed { self.reset(); }
        if olc::get_key(olc::Key::P).pressed {
            self.selected_palette += 1;
            self.selected_palette &= 0x07;
        }

        self.draw_cpu(516, 2);
        self.draw_code(516, 72, 26);
        //self.draw_ram(516, 100, &mut 0x0000, 16, 16);
        //self.draw_ram(516, 300, &mut 0x8000, 16, 16);

        let swatch_size = 6;
        for p in 0..8 {
            for s in 0..4 {
                olc::fill_rect(516 + p * (swatch_size * 5) + s * swatch_size, 340, swatch_size, swatch_size, self.ppu.borrow().get_color_from_palette_ram(p.try_into().unwrap(), s.try_into().unwrap()));
            }
        }
        olc::draw_rect(516 + i32::from(self.selected_palette) * (swatch_size * 5) - 1, 339, swatch_size * 4, swatch_size, olc::WHITE);

        self.ppu.borrow_mut().build_pattern_table(0, self.selected_palette);
        self.ppu.borrow_mut().build_pattern_table(1, self.selected_palette);

        olc::draw_sprite(516, 348, self.ppu.borrow().get_pattern_table(0));
        olc::draw_sprite(648, 348, self.ppu.borrow().get_pattern_table(1));

        olc::draw_sprite_ext(0, 0, &self.ppu.borrow().spr_screen, 2, olc_pixel_game_engine::SpriteFlip::NONE);

        Ok(())
    }

    fn on_user_destroy(&mut self) -> Result<(), olc::Error> {
        Ok(())
    }
}

fn main() {
    let cartridge = Rc::new(RefCell::new(Cartridge::new(String::from("nestest.nes"))));

    let ppu = Rc::new(RefCell::new(PPU::new(cartridge.clone())));

    let bus = Bus {
        cartridge: cartridge.clone(),
        memory: Memory::new(),
        ppu: ppu.clone(),
    };
    let cpu = CPU {
        registers: Registers::new(),
        bus,
        cycles: 0,
        clock_count: 0,
    };

    let mut emulator = Emulator {
        cpu,
        ppu: ppu.clone(),
        emulation_run: false,
        residual_time: 0f32,
        system_clock_counter: 0,
        selected_palette: 0,
        map_asm: HashMap::new(),
    };
    olc::start("nes", &mut emulator, 780, 480, 2, 2).unwrap();
}
