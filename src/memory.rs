pub struct Memory {
    bytes: [u8; 2048],
}

impl Memory {
    pub const fn new() -> Memory {
        Memory {
            bytes: [0; 2048],
        }
    }

    pub fn get_byte(&self, address: u16) -> u8 {
        self.bytes[address as usize]
    }

    pub fn set_byte(&mut self, address: u16, value: u8) {
        self.bytes[address as usize] = value;
    }
}
