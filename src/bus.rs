use crate::cpu::CPU;
use crate::memory::Memory;
use crate::instruction::Nmos6502;
use crate::cartridge::Cartridge;

pub struct Bus {
    cpu: CPU<Memory, Nmos6502>,
    cpu_ram: [u8; 2048],
    cartridge: Cartridge,
}

impl Bus {
    pub fn new(cpu: CPU<Memory, Nmos6502>, cartridge: Cartridge) -> Bus {
        Bus {
            cpu,
            cpu_ram: [0; 2048],
            cartridge,
        }
    }

    pub fn cpu_write(&mut self, addr: u16, data: u8) {
        if addr <= 0x1FFF {
            self.cpu_ram[(addr & 0x07FF) as usize] = data;
        }
    }

    pub fn cpu_read(&self, addr: u16) -> u8 {
        if addr <= 0x1FFF {
            return self.cpu_ram[(addr & 0x07FF) as usize];
        }

        return 0x00;
    }

    pub fn reset(&mut self) {
        self.cpu.reset();
    }
}
