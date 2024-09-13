const ADDR_LO_BARE: u16 = 0x0000;
const ADDR_HI_BARE: u16 = 0xFFFF;

const MEMORY_SIZE: usize = (ADDR_HI_BARE - ADDR_LO_BARE) as usize + 1usize;

pub struct Memory {
    bytes: [u8; MEMORY_SIZE],
}

impl Default for Memory {
    fn default() -> Self {
        Self::new()
    }
}

pub trait Bus {
    fn get_byte(&mut self, address: u16) -> u8;
    fn set_byte(&mut self, address: u16, value: u8);
}

impl Memory {
    pub const fn new() -> Memory {
        Memory {
            bytes: [0; MEMORY_SIZE],
        }
    }
}

impl Bus for Memory {
    fn get_byte(&mut self, address: u16) -> u8 {
        self.bytes[address as usize]
    }

    fn set_byte(&mut self, address: u16, value: u8) {
        self.bytes[address as usize] = value;
    }
}
