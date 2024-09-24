use crate::cartridge::Cartridge;
use crate::memory::Memory;

pub struct Bus {
    pub memory: Memory,
    pub cartridge: Cartridge,
}

impl Bus {
    // fn cpu_write_bytes(&mut self, start: u16, values: &[u8]) {
    //     for i in 0..values.len() as u16 {
    //         self.cpu_write(start + i, values[i as usize]);
    //     }
    // }

    pub fn cpu_write(&mut self, addr: u16, data: u8) {
        if self.cartridge.cpu_write(addr, data) {
            // done
        } else if addr <= 0x1FFF {
            self.memory.set_byte(addr & 0x07FF, data);
        }
    }

    pub fn cpu_read(&mut self, addr: u16) -> u8 {
        let (was_read, data) = self.cartridge.cpu_read(addr);
        if was_read {
            data
        } else if addr <= 0x1FFF {
            self.memory.get_byte(addr & 0x07FF)
        } else {
            0x00
        }
    }
}
