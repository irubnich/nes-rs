use std::{cell::RefCell, rc::Rc};
use bitflags::bitflags;

use olc_pixel_game_engine as olc;

use crate::cartridge::{Cartridge, Mirror};

#[derive(Debug)]
pub struct PPU {
    pub tbl_name: [[u8; 1024]; 2],
    tbl_pattern: [[u8; 4096]; 2],
    tbl_palette: [u8; 32],

    pal_screen: Vec<olc::Pixel>,

    pub spr_screen: olc::Sprite,
    _spr_name_table: [olc::Sprite; 2],
    pub spr_pattern_table: [olc::Sprite; 2],

    pub frame_complete: bool,
    scanline: i32,
    cycle: i32,

    cart: Rc<RefCell<Cartridge>>,

    status: PPUStatus,
    mask: PPUMask,
    control: PPUControl,
    vram_addr: LoopyRegister,
    tram_addr: LoopyRegister,
    pub nmi: bool,

    address_latch: u8,
    ppu_data_buffer: u8,
    fine_x: u8,

    bg_next_tile_id: u8,
    bg_next_tile_attrib: u8,
    bg_next_tile_lsb: u8,
    bg_next_tile_msb: u8,

    bg_shifter_pattern_lo: u16,
    bg_shifter_pattern_hi: u16,
    bg_shifter_attrib_lo: u16,
    bg_shifter_attrib_hi: u16,

    addr_hi: u8,
}

bitflags! {
    #[derive(Copy, Clone, Debug)]
    pub struct PPUStatus: u8 {
        const SPRITE_OVERFLOW = 0b0010_0000;
        const SPRITE_ZERO_HIT = 0b0100_0000;
        const VERTICAL_BLANK = 0b1000_0000;
    }
}

bitflags! {
    #[derive(Copy, Clone, Debug)]
    pub struct PPUMask: u8 {
        const GRAYSCALE = 0b0000_0001;
        const RENDER_BACKGROUND_LEFT = 0b0000_0010;
        const RENDER_SPRITES_LEFT = 0b0000_0100;
        const RENDER_BACKGROUND = 0b0000_1000;
        const RENDER_SPRITES = 0b0001_0000;
        const ENHANCE_RED = 0b0010_0000;
        const ENHANCE_GREEN = 0b0100_0000;
        const ENHANCE_BLUE = 0b1000_0000;
    }
}

bitflags! {
    #[derive(Copy, Clone, Debug)]
    pub struct PPUControl: u8 {
        const NAMETABLE_X = 0b0000_0001;
        const NAMETABLE_Y = 0b0000_0010;
        const INCREMENT_MODE = 0b0000_0100;
        const PATTERN_SPRITE = 0b0000_1000;
        const PATTERN_BACKGROUND = 0b0001_0000;
        const SPRITE_SIZE = 0b0010_0000;
        const SLAVE_MODE = 0b0100_0000;
        const ENABLE_NMI = 0b1000_0000;
    }
}

#[derive(Debug, Copy, Clone)]
pub struct LoopyRegister {
    pub register: u16,
}

impl LoopyRegister {
    pub fn new() -> LoopyRegister {
        LoopyRegister {
            register: 0x0000,
        }
    }

    pub fn get_fine_y_scroll(&self) -> u8 {
        let mask: u16 = 0b111 << 12;
        let extract = (self.register & mask) >> 12;
        extract.try_into().unwrap()
    }

    pub fn set_fine_y_scroll(&mut self, val: u8) {
        let val: u16 = u16::from(val) & 0b0000_0111;
        let new_register = (val << 12) | self.register;
        self.register = new_register;
    }

    pub fn get_nametable_select_x(&self) -> u8 {
        let mask: u16 = 0b01 << 11;
        let extract = (self.register & mask) >> 11;
        extract.try_into().unwrap()
    }

    pub fn set_nametable_select_x(&mut self, val: u8) {
        let val: u16 = u16::from(val) & 0b01;
        let new_register = (val << 11) | self.register;
        self.register = new_register;
    }

    pub fn get_nametable_select_y(&self) -> u8 {
        let mask: u16 = 0b10 << 10;
        let extract = (self.register & mask) >> 10;
        extract.try_into().unwrap()
    }

    pub fn set_nametable_select_y(&mut self, val: u8) {
        let val: u16 = u16::from(val) & 0b10;
        let new_register = (val << 10) | self.register;
        self.register = new_register;
    }

    pub fn get_coarse_y_scroll(&self) -> u8 {
        let mask: u16 = 0b0001_1111 << 5;
        let extract = (self.register & mask) >> 5;
        extract.try_into().unwrap()
    }

    pub fn set_coarse_y_scroll(&mut self, val: u8) {
        let val: u16 = u16::from(val) & 0b0001_1111;
        let new_register = (val << 5) | self.register;
        self.register = new_register;
    }

    pub fn get_coarse_x_scroll(&self) -> u8 {
        let mask: u16 = 0b0001_1111;
        let extract = self.register & mask;
        extract.try_into().unwrap()
    }

    pub fn set_coarse_x_scroll(&mut self, val: u8) {
        let val: u16 = u16::from(val) & 0b0001_1111;
        let new_register = val | self.register;
        self.register = new_register;
    }
}

impl PPU {
    pub fn new(cart: Rc<RefCell<Cartridge>>) -> PPU {
        let mut pal_screen = Vec::new();
        pal_screen.resize(0x40, olc::Pixel::rgb(0, 0, 0));

        pal_screen[0x00] = olc::Pixel::rgb(84, 84, 84);
        pal_screen[0x01] = olc::Pixel::rgb(0, 30, 116);
        pal_screen[0x02] = olc::Pixel::rgb(8, 16, 144);
        pal_screen[0x03] = olc::Pixel::rgb(48, 0, 136);
        pal_screen[0x04] = olc::Pixel::rgb(68, 0, 100);
        pal_screen[0x05] = olc::Pixel::rgb(92, 0, 48);
        pal_screen[0x06] = olc::Pixel::rgb(84, 4, 0);
        pal_screen[0x07] = olc::Pixel::rgb(60, 24, 0);
        pal_screen[0x08] = olc::Pixel::rgb(32, 42, 0);
        pal_screen[0x09] = olc::Pixel::rgb(8, 58, 0);
        pal_screen[0x0A] = olc::Pixel::rgb(0, 64, 0);
        pal_screen[0x0B] = olc::Pixel::rgb(0, 60, 0);
        pal_screen[0x0C] = olc::Pixel::rgb(0, 50, 60);
        pal_screen[0x0D] = olc::Pixel::rgb(0, 0, 0);
        pal_screen[0x0E] = olc::Pixel::rgb(0, 0, 0);
        pal_screen[0x0F] = olc::Pixel::rgb(0, 0, 0);

        pal_screen[0x10] = olc::Pixel::rgb(152, 150, 152);
        pal_screen[0x11] = olc::Pixel::rgb(8, 76, 196);
        pal_screen[0x12] = olc::Pixel::rgb(48, 50, 236);
        pal_screen[0x13] = olc::Pixel::rgb(92, 30, 228);
        pal_screen[0x14] = olc::Pixel::rgb(136, 20, 176);
        pal_screen[0x15] = olc::Pixel::rgb(160, 20, 100);
        pal_screen[0x16] = olc::Pixel::rgb(152, 34, 32);
        pal_screen[0x17] = olc::Pixel::rgb(120, 60, 0);
        pal_screen[0x18] = olc::Pixel::rgb(84, 90, 0);
        pal_screen[0x19] = olc::Pixel::rgb(40, 114, 0);
        pal_screen[0x1A] = olc::Pixel::rgb(8, 124, 0);
        pal_screen[0x1B] = olc::Pixel::rgb(0, 118, 40);
        pal_screen[0x1C] = olc::Pixel::rgb(0, 102, 120);
        pal_screen[0x1D] = olc::Pixel::rgb(0, 0, 0);
        pal_screen[0x1E] = olc::Pixel::rgb(0, 0, 0);
        pal_screen[0x1F] = olc::Pixel::rgb(0, 0, 0);

        pal_screen[0x20] = olc::Pixel::rgb(236, 238, 236);
        pal_screen[0x21] = olc::Pixel::rgb(76, 154, 236);
        pal_screen[0x22] = olc::Pixel::rgb(120, 124, 236);
        pal_screen[0x23] = olc::Pixel::rgb(176, 98, 236);
        pal_screen[0x24] = olc::Pixel::rgb(228, 84, 236);
        pal_screen[0x25] = olc::Pixel::rgb(236, 88, 180);
        pal_screen[0x26] = olc::Pixel::rgb(236, 106, 100);
        pal_screen[0x27] = olc::Pixel::rgb(212, 136, 32);
        pal_screen[0x28] = olc::Pixel::rgb(160, 170, 0);
        pal_screen[0x29] = olc::Pixel::rgb(116, 196, 0);
        pal_screen[0x2A] = olc::Pixel::rgb(76, 208, 32);
        pal_screen[0x2B] = olc::Pixel::rgb(56, 204, 108);
        pal_screen[0x2C] = olc::Pixel::rgb(56, 180, 204);
        pal_screen[0x2D] = olc::Pixel::rgb(60, 60, 60);
        pal_screen[0x2E] = olc::Pixel::rgb(0, 0, 0);
        pal_screen[0x2F] = olc::Pixel::rgb(0, 0, 0);

        pal_screen[0x30] = olc::Pixel::rgb(236, 238, 236);
        pal_screen[0x31] = olc::Pixel::rgb(168, 204, 236);
        pal_screen[0x32] = olc::Pixel::rgb(188, 188, 236);
        pal_screen[0x33] = olc::Pixel::rgb(212, 178, 236);
        pal_screen[0x34] = olc::Pixel::rgb(236, 174, 236);
        pal_screen[0x35] = olc::Pixel::rgb(236, 174, 212);
        pal_screen[0x36] = olc::Pixel::rgb(236, 180, 176);
        pal_screen[0x37] = olc::Pixel::rgb(228, 196, 144);
        pal_screen[0x38] = olc::Pixel::rgb(204, 210, 120);
        pal_screen[0x39] = olc::Pixel::rgb(180, 222, 120);
        pal_screen[0x3A] = olc::Pixel::rgb(168, 226, 144);
        pal_screen[0x3B] = olc::Pixel::rgb(152, 226, 180);
        pal_screen[0x3C] = olc::Pixel::rgb(160, 214, 228);
        pal_screen[0x3D] = olc::Pixel::rgb(160, 162, 160);
        pal_screen[0x3E] = olc::Pixel::rgb(0, 0, 0);
        pal_screen[0x3F] = olc::Pixel::rgb(0, 0, 0);

        let ppu = PPU {
            pal_screen,

            tbl_name: [[0; 1024]; 2],
            tbl_pattern: [[0; 4096]; 2],
            tbl_palette: [0; 32],

            spr_screen: olc::Sprite::with_dims(256, 240),
            _spr_name_table: [
                olc::Sprite::with_dims(256, 240),
                olc::Sprite::with_dims(256, 240),
            ],
            spr_pattern_table: [
                olc::Sprite::with_dims(128, 128),
                olc::Sprite::with_dims(128, 128),
            ],

            frame_complete: false,
            scanline: 0,
            cycle: 0,
            cart,

            status: PPUStatus::empty(),
            mask: PPUMask::empty(),
            control: PPUControl::empty(),
            vram_addr: LoopyRegister::new(),
            tram_addr: LoopyRegister::new(),
            address_latch: 0,
            ppu_data_buffer: 0,
            addr_hi: 0,
            nmi: false,
            fine_x: 0,
            bg_next_tile_attrib: 0,
            bg_next_tile_id: 0,
            bg_next_tile_lsb: 0,
            bg_next_tile_msb: 0,
            bg_shifter_attrib_hi: 0,
            bg_shifter_attrib_lo: 0,
            bg_shifter_pattern_hi: 0,
            bg_shifter_pattern_lo: 0,
        };

        ppu
    }

    pub fn cpu_read(&mut self, addr: u16, read_only: bool) -> u8 {
        match addr {
            0x0000 => 0x00,
            0x0001 => 0x00,
            0x0002 => {
                if read_only {
                    return self.status.bits();
                }

                let data = (self.status.bits() & 0xE0) | (self.ppu_data_buffer & 0x1F);
                self.status.set(PPUStatus::VERTICAL_BLANK, false);
                self.address_latch = 0;
                data
            },
            0x0003 => 0x00,
            0x0004 => 0x00,
            0x0005 => 0x00,
            0x0006 => 0x00,
            0x0007 => {
                if read_only {
                    return 0x00;
                }

                let mut data = self.ppu_data_buffer;
                self.ppu_data_buffer = self.ppu_read(self.vram_addr.register);

                if self.vram_addr.register >= 0x3F00 {
                    data = self.ppu_data_buffer;
                }

                self.vram_addr.register += if self.control.intersects(PPUControl::INCREMENT_MODE) { 32 } else { 1 };

                data
            },
            _ => panic!("invalid CPU read from PPU")
        }
    }

    pub fn cpu_write(&mut self, addr: u16, data: u8) {
        match addr {
            0x0000 => {
                self.control = PPUControl::from_bits(data).unwrap();

                let val_y = self.control.bits() & 0b0000_0010;
                let val_x = self.control.bits() & 0b0000_0001;
                self.tram_addr.set_nametable_select_x(val_x);
                self.tram_addr.set_nametable_select_y(val_y);
            },
            0x0001 => {
                self.mask = PPUMask::from_bits(data).unwrap();
            },
            0x0002 => (),
            0x0003 => (),
            0x0004 => (),
            0x0005 => {
                if self.address_latch == 0 {
                    let coarse_x_val = (data & 0b1111_1000) >> 3;
                    self.tram_addr.register |= u16::from(coarse_x_val);

                    self.fine_x = data & 0b0111;

                    self.address_latch = 1;
                } else {
                    let fine_y_scroll_val = data & 0b0000_0111;
                    self.tram_addr.set_fine_y_scroll(fine_y_scroll_val);

                    self.tram_addr.register |= u16::from(((data & 0b1111_1000) >> 3) << 5);

                    self.address_latch = 0;
                }
            },
            0x0006 => {
                if self.address_latch == 0 {
                    self.tram_addr.register = u16::from(data & 0b0011_1111) << 8;

                    self.tram_addr.register &= 0b011_1111_1111_1111;

                    self.address_latch = 1;
                } else {
                    self.tram_addr.register |= u16::from(data);
                    self.vram_addr = self.tram_addr;

                    self.address_latch = 0;
                }
            },
            0x0007 => {
                self.ppu_write(self.vram_addr.register, data);
                self.vram_addr.register += if self.control.intersects(PPUControl::INCREMENT_MODE) { 32 } else { 1 };
            },
            _ => panic!("invalid CPU write from PPU")
        }
    }

    pub fn ppu_read(&self, addr: u16) -> u8 {
        let addr = addr & 0x3FFF;

        if let (true, data) = self.cart.borrow().ppu_read(addr) {
            return data;
        } else if addr <= 0x1FFF {
            let idx1 = (addr & 0x1000) >> 12;
            let idx2 = addr & 0x0FFF;

            return self.tbl_pattern[idx1 as usize][idx2 as usize];
        } else if addr >= 0x2000 && addr <= 0x3EFF {
            let addr = addr & 0x0FFF;
            let idx = (addr & 0x03FF) as usize;

            if self.cart.borrow().mirror == Mirror::VERTICAL {
                match addr {
                    0x0000..=0x03FF => self.tbl_name[0][idx],
                    0x0400..=0x07FF => self.tbl_name[1][idx],
                    0x0800..=0x0BFF => self.tbl_name[0][idx],
                    0x0C00..=0x0FFF => self.tbl_name[1][idx],
                    _ => panic!("invalid nametable read 1"),
                }
            } else if self.cart.borrow().mirror == Mirror::HORIZONTAL {
                match addr {
                    0x0000..=0x03FF => self.tbl_name[0][idx],
                    0x0400..=0x07FF => self.tbl_name[0][idx],
                    0x0800..=0x0BFF => self.tbl_name[1][idx],
                    0x0C00..=0x0FFF => self.tbl_name[1][idx],
                    x => {
                        panic!("invalid nametable read 2, from: {x:04X}");
                    },
                }
            } else {
                panic!("invalid mirroring read");
            }
        } else if addr >= 0x3F00 && addr <= 0x3FFF {
            let mut addr = addr & 0x001F;
            if addr == 0x0010 { addr = 0x0000 };
            if addr == 0x0014 { addr = 0x0004 };
            if addr == 0x0018 { addr = 0x0008 };
            if addr == 0x001C { addr = 0x000C };

            return self.tbl_palette[addr as usize];
        } else {
            panic!("bad PPU read")
        }
    }

    pub fn ppu_write(&mut self, addr: u16, data: u8) {
        let addr = addr & 0x3FFF;

        if self.cart.borrow_mut().ppu_write(addr, data) {
            // done
        } else if addr <= 0x1FFF {
            let idx1 = (addr & 0x1000) >> 12;
            let idx2 = addr & 0x0FFF;

            self.tbl_pattern[idx1 as usize][idx2 as usize] = data;
        } else if addr >= 0x2000 && addr <= 0x3EFF {
            let addr = addr & 0x0FFF;
            let idx = (addr & 0x03FF) as usize;

            if self.cart.borrow().mirror == Mirror::VERTICAL {
                match addr {
                    0x0000..=0x03FF => self.tbl_name[0][idx] = data,
                    0x0400..=0x07FF => self.tbl_name[1][idx] = data,
                    0x0800..=0x0BFF => self.tbl_name[0][idx] = data,
                    0x0C00..=0x0FFF => self.tbl_name[1][idx] = data,
                    _ => panic!("invalid nametable write 1"),
                }
            } else if self.cart.borrow().mirror == Mirror::HORIZONTAL {
                match addr {
                    0x0000..=0x03FF => self.tbl_name[0][idx] = data,
                    0x0400..=0x07FF => self.tbl_name[0][idx] = data,
                    0x0800..=0x0BFF => self.tbl_name[1][idx] = data,
                    0x0C00..=0x0FFF => self.tbl_name[1][idx] = data,
                    _ => panic!("invalid nametable write 2"),
                }
            } else {
                panic!("invalid mirroring write");
            }
        } else if addr >= 0x3F00 && addr <= 0x3FFF {
            let mut addr = addr & 0x001F;
            if addr == 0x0010 { addr = 0x0000 };
            if addr == 0x0014 { addr = 0x0004 };
            if addr == 0x0018 { addr = 0x0008 };
            if addr == 0x001C { addr = 0x000C };

            self.tbl_palette[addr as usize] = data;
        } else {
            println!("invalid write to {:X}", addr);
        }
    }

    pub fn reset(&mut self) {
        self.address_latch = 0;
        self.ppu_data_buffer = 0;
        self.scanline = 0;
        self.cycle = 0;
        self.status = PPUStatus::empty();
        self.mask = PPUMask::empty();
        self.control = PPUControl::empty();
    }

    fn increment_scroll_x(&mut self) {
        if self.mask.intersects(PPUMask::RENDER_BACKGROUND) || self.mask.intersects(PPUMask::RENDER_SPRITES) {
            if self.vram_addr.get_coarse_x_scroll() == 31 {
                self.vram_addr.set_coarse_x_scroll(0);

                // [...]YX[...]
                let x_mask = 0b000_0100_0000_0000;
                self.vram_addr.register = self.vram_addr.register ^ x_mask;
            } else {
                self.vram_addr.set_coarse_x_scroll(self.vram_addr.get_coarse_x_scroll() + 1);
            }
        }
    }

    fn increment_scroll_y(&mut self) {
        if self.mask.intersects(PPUMask::RENDER_BACKGROUND) || self.mask.intersects(PPUMask::RENDER_SPRITES) {
            if self.vram_addr.get_fine_y_scroll() < 7 {
                self.vram_addr.set_fine_y_scroll(self.vram_addr.get_fine_y_scroll() + 1);
            } else {
                self.vram_addr.set_fine_y_scroll(0);

                if self.vram_addr.get_coarse_y_scroll() == 29 {
                    self.vram_addr.set_coarse_y_scroll(0);

                    // [...]YX[...]
                    let y_mask = 0b000_1000_0000_0000;
                    self.vram_addr.register = self.vram_addr.register ^ y_mask;
                } else if self.vram_addr.get_coarse_y_scroll() == 31 {
                    self.vram_addr.set_coarse_y_scroll(0);
                } else {
                    self.vram_addr.set_coarse_y_scroll(self.vram_addr.get_coarse_y_scroll() + 1);
                }
            }
        }
    }

    fn transfer_address_x(&mut self) {
        if self.mask.intersects(PPUMask::RENDER_BACKGROUND) || self.mask.intersects(PPUMask::RENDER_SPRITES) {
            let nametable_x_val = self.tram_addr.register & 0b000_0100_0000_0000;
            self.vram_addr.register |= nametable_x_val;

            self.vram_addr.set_coarse_x_scroll(self.tram_addr.get_coarse_x_scroll());
        }
    }

    fn transfer_address_y(&mut self) {
        if self.mask.intersects(PPUMask::RENDER_BACKGROUND) || self.mask.intersects(PPUMask::RENDER_SPRITES) {
            self.vram_addr.set_fine_y_scroll(self.tram_addr.get_fine_y_scroll());

            let nametable_y_val = self.tram_addr.register & 0b000_1000_0000_0000;
            self.vram_addr.register |= nametable_y_val;

            self.vram_addr.set_coarse_y_scroll(self.tram_addr.get_coarse_y_scroll());
        }
    }

    fn load_background_shifters(&mut self) {
        self.bg_shifter_pattern_lo = (self.bg_shifter_pattern_lo & 0xFF00) | u16::from(self.bg_next_tile_lsb);
        self.bg_shifter_pattern_hi = (self.bg_shifter_pattern_hi & 0xFF00) | u16::from(self.bg_next_tile_msb);
        self.bg_shifter_attrib_lo = (self.bg_shifter_attrib_lo & 0xFF00) | u16::from(if self.bg_next_tile_attrib & 0b01 > 0 { 0xFFu16 } else { 0x00 });
        self.bg_shifter_attrib_hi = (self.bg_shifter_attrib_hi & 0xFF00) | u16::from(if self.bg_next_tile_attrib & 0b10 > 0 { 0xFFu16 } else { 0x00 });
    }

    fn update_shifters(&mut self) {
        if self.mask.intersects(PPUMask::RENDER_BACKGROUND) {
            self.bg_shifter_pattern_lo <<= 1;
            self.bg_shifter_pattern_hi <<= 1;
            self.bg_shifter_attrib_lo <<= 1;
            self.bg_shifter_attrib_hi <<= 1;
        }
    }

    pub fn clock(&mut self) {
        if self.scanline >= -1 && self.scanline < 240 {
            if self.scanline == 0 && self.cycle == 0 {
                self.cycle = 1;
            }

            if self.scanline == -1 && self.cycle == 1 {
                self.status.set(PPUStatus::VERTICAL_BLANK, false);
            }

            if (self.cycle >= 2 && self.cycle < 258) || (self.cycle >= 321 && self.cycle < 338) {
                self.update_shifters();

                match (self.cycle - 1) % 8 {
                    0 => {
                        self.load_background_shifters();
                        self.bg_next_tile_id = self.ppu_read(0x2000 | (self.vram_addr.register & 0x0FFF))
                    },
                    2 => {
                        let addr: u16 = 0x23C0
                            | u16::from(self.vram_addr.get_nametable_select_y()) << 11
                            | u16::from(self.vram_addr.get_nametable_select_x()) << 10
                            | u16::from(self.vram_addr.get_coarse_y_scroll() >> 2) << 3
                            | u16::from(self.vram_addr.get_coarse_x_scroll() >> 2);

                        self.bg_next_tile_attrib = self.ppu_read(addr);
                        if self.vram_addr.get_coarse_y_scroll() & 0x02 > 0 { self.bg_next_tile_attrib >>= 4 }
                        if self.vram_addr.get_coarse_x_scroll() & 0x02 > 0 { self.bg_next_tile_attrib >>= 2 }

                        self.bg_next_tile_attrib &= 0x03;
                    },
                    4 => {
                        let bit: u16 = if self.control.intersects(PPUControl::PATTERN_BACKGROUND) { 1 } else { 0 };
                        let addr = (bit << 12) + (u16::from(self.bg_next_tile_id) << 4) + u16::from(self.vram_addr.get_fine_y_scroll());

                        self.bg_next_tile_lsb = self.ppu_read(addr);
                    },
                    6 => {
                        let bit: u16 = if self.control.intersects(PPUControl::PATTERN_BACKGROUND) { 1 } else { 0 };
                        let addr = (bit << 12) + (u16::from(self.bg_next_tile_id) << 4) + u16::from(self.vram_addr.get_fine_y_scroll()) + 8;

                        self.bg_next_tile_msb = self.ppu_read(addr);
                    },
                    7 => {
                        self.increment_scroll_x();
                    },
                    _ => (), // noop
                }

                if self.cycle == 256 {
                    self.increment_scroll_y();
                }

                if self.cycle == 257 {
                    self.load_background_shifters();
                    self.transfer_address_x();
                }

                if self.cycle == 338 || self.cycle == 340 {
                    self.bg_next_tile_id = self.ppu_read(0x2000 | (self.vram_addr.register & 0x0FFF));
                }

                if self.scanline == -1 && self.cycle >= 280 && self.cycle < 305 {
                    self.transfer_address_y();
                }
            }
        }

        if self.scanline == 241 && self.cycle == 1 {
            self.status.set(PPUStatus::VERTICAL_BLANK, true);
            if self.control.intersects(PPUControl::ENABLE_NMI) {
                self.nmi = true;
            }
        }

        let mut bg_pixel: u8 = 0x00;
        let mut bg_palette: u8 = 0x00;

        if self.mask.intersects(PPUMask::RENDER_BACKGROUND) {
            let bit_mux: u16 = 0x8000 >> self.fine_x;
            let p0_pixel: u8 = if self.bg_shifter_pattern_lo & bit_mux > 0 { 1 } else { 0 };
            let p1_pixel: u8 = if self.bg_shifter_pattern_hi & bit_mux > 0 { 1 } else { 0 };
            bg_pixel = (p1_pixel << 1) | p0_pixel;

            let bg_pal0 = if self.bg_shifter_attrib_lo & bit_mux > 0 { 1 } else { 0 };
            let bg_pal1 = if self.bg_shifter_attrib_hi & bit_mux > 0 { 1 } else { 0 };
            bg_palette = (bg_pal1 << 1) | bg_pal0;
        }

        if bg_pixel != 0 || bg_palette != 0 {
            //println!("pixel coords: {bg_pixel},{bg_palette}");
        }
        self.spr_screen.set_pixel(self.cycle - 1, self.scanline, self.get_color_from_palette_ram(bg_palette, bg_pixel));

        self.cycle += 1;
        if self.cycle >= 341 {
            self.cycle = 0;
            self.scanline += 1;
            if self.scanline >= 261 {
                self.scanline = -1;
                self.frame_complete = true;
            }
        }
    }

    pub fn get_pattern_table(&self, i: u8) -> &olc::Sprite {
        &self.spr_pattern_table[i as usize]
    }

    pub fn build_pattern_table(&mut self, i: u8, palette: u8) {
        for tile_y in 0..16 {
            for tile_x in 0..16 {
                let offset = (tile_y * 256) + (tile_x * 16);

                for row in 0..8 {
                    let mut tile_lsb = self.ppu_read(u16::from(i) * 0x1000 + offset + row);
                    let mut tile_msb = self.ppu_read(u16::from(i) * 0x1000 + offset + row + 8);

                    for col in 0..8 {
                        let pixel = (tile_lsb & 0x01) + (tile_msb & 0x01);

                        tile_lsb >>= 1;
                        tile_msb >>= 1;

                        self.spr_pattern_table[i as usize].set_pixel(
                            (tile_x * 8 + (7 - col)).into(),
                            (tile_y * 8 + row).into(),
                            self.get_color_from_palette_ram(palette, pixel)
                        );
                    }
                }
            }
        }
    }

    pub fn get_color_from_palette_ram(&self, palette: u8, pixel: u8) -> olc::Pixel {
        let idx = self.ppu_read(0x3F00 + (u16::from(palette) * 4) + u16::from(pixel));
        self.pal_screen[idx as usize & 0x3F]
    }
}
