use std::cell::RefCell;
use std::rc::Rc;

use crate::cartridge::Cartridge;
use crate::memory::Memory;
use crate::ppu::PPU;

pub struct Bus {
    pub memory: Memory,
    pub cartridge: Rc<RefCell<Cartridge>>,
    pub ppu: Rc<RefCell<PPU>>,
}

impl Bus {
    // fn cpu_write_bytes(&mut self, start: u16, values: &[u8]) {
    //     for i in 0..values.len() as u16 {
    //         self.cpu_write(start + i, values[i as usize]);
    //     }
    // }

    pub fn cpu_write(&mut self, addr: u16, data: u8) {
        if self.cartridge.borrow_mut().cpu_write(addr, data) {
            // done
        } else if addr <= 0x1FFF {
            self.memory.set_byte(addr & 0x07FF, data);
        } else if addr >= 0x2000 && addr <= 0x3FFF {
            println!("bus: writing to PPU. addr: {:X}, data: {:X}", addr & 0x0007, data);
            self.ppu.borrow_mut().cpu_write(addr & 0x0007, data);
        }
    }

    pub fn cpu_read(&mut self, addr: u16, read_only: bool) -> u8 {
        let (was_read, data) = self.cartridge.borrow().cpu_read(addr);
        if was_read {
            data
        } else if addr <= 0x1FFF {
            self.memory.get_byte(addr & 0x07FF)
        } else if addr >= 0x2000 && addr <= 0x3FFF {
            self.ppu.borrow_mut().cpu_read(addr & 0x0007, read_only)
        } else {
            0x00
        }
    }
}
