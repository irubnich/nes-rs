#[derive(Debug)]
pub struct Mapper {
    prg_banks: u8,
    chr_banks: u8,
}

impl Mapper {
    pub fn new(prg_banks: u8, chr_banks: u8) -> Mapper {
        Mapper {
            prg_banks,
            chr_banks,
        }
    }

    pub fn cpu_map_read(&self, addr: u16) -> (bool, u32) {
        if addr >= 0x8000 && addr <= 0xBFFF {
            let and_with = if self.prg_banks > 1 { 0x7FFF } else { 0x3FFF };
            let mapped_addr = addr & and_with;
            return (true, mapped_addr.into());
        } else if addr >= 0xC000 {
            // todo not fully right
            return (true, (addr - 0xC000).into());
        }

        (false, 0)
    }

    pub fn cpu_map_write(&self, addr: u16) -> (bool, u32) {
        if addr >= 0x8000 && addr <= 0xBFFF {
            let and_with = if self.prg_banks > 1 { 0x7FFF } else { 0x3FFF };
            let mapped_addr = addr & and_with;
            return (true, mapped_addr.into());
        }

        (false, 0)
    }

    pub fn ppu_map_read(&self, addr: u16) -> (bool, u32) {
        if addr <= 0x1FFF {
            return (true, addr.into());
        }

        (false, 0)
    }

    pub fn ppu_map_write(&self, addr: u16) -> (bool, u32) {
        if addr <= 0x1FFF && self.chr_banks == 0 {
            return (true, addr.into());
        }

        (false, 0)
    }
}
