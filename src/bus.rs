use std::cell::RefCell;
use std::rc::Rc;

use crate::cartridge::Cartridge;
use crate::memory::Memory;
use crate::ppu::PPU;

pub struct Bus {
    pub memory: Memory,
    pub cartridge: Rc<RefCell<Cartridge>>,
    pub ppu: Rc<RefCell<PPU>>,
    pub controller: [u8; 2],
    pub controller_state: [u8; 2],
}

impl Bus {
    pub fn cpu_write(&mut self, addr: u16, data: u8) {
        if self.cartridge.borrow_mut().cpu_write(addr, data) {
            // done
        } else if addr <= 0x1FFF {
            self.memory.set_byte(addr & 0x07FF, data);
        } else if addr >= 0x2000 && addr <= 0x3FFF {
            self.ppu.borrow_mut().cpu_write(addr & 0x0007, data);
        } else if addr >= 0x4016 && addr <= 0x4017 {
            let idx = (addr & 0x0001) as usize;
            self.controller_state[idx] = self.controller[idx];
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
        } else if addr >= 0x4016 && addr <= 0x4017 {
            let idx = (addr & 0x0001) as usize;
            let data = ((self.controller_state[idx] & 0x80) > 0) as u8;
            self.controller_state[idx] <<= 1;

            data
        } else {
            0x00
        }
    }
}
