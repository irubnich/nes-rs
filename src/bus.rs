use crate::cartridge::Cartridge;

pub struct Bus {
    cpu_ram: [u8; 2048],
    cartridge: Cartridge,
}

impl Bus {
    pub fn new(cartridge: Cartridge) -> Bus {
        Bus {
            cpu_ram: [0; 2048],
            cartridge,
        }
    }

    fn cpu_write_bytes(&mut self, start: u16, values: &[u8]) {
        for i in 0..values.len() as u16 {
            self.cpu_write(start + i, values[i as usize]);
        }
    }

    pub fn cpu_write(&mut self, addr: u16, data: u8) {
        if self.cartridge.cpu_write(addr, data) {
            // done
        } else if addr <= 0x1FFF {
            self.cpu_ram[(addr & 0x07FF) as usize] = data;
        }
    }

    pub fn cpu_read(&self, addr: u16) -> u8 {
        let data = self.cartridge.cpu_read(addr);
        if data.0 {
            return data.1;
        } else if addr <= 0x1FFF {
            return self.cpu_ram[(addr & 0x07FF) as usize];
        }

        return 0x00;
    }

    pub fn reset(&mut self) {
        //self.cpu.reset();
    }
}
