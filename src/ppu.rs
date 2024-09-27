use std::{cell::RefCell, rc::Rc};
use bitflags::bitflags;

use olc_pixel_game_engine as olc;
use rand::Rng;

use crate::cartridge::Cartridge;

#[derive(Debug)]
pub struct PPU {
    //tbl_name: [[u8; 1024]; 2],
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

    address_latch: u8,
    ppu_data_buffer: u8,
    ppu_address: u16,

    addr_hi: u8,
}

bitflags! {
    #[derive(Copy, Clone, Debug)]
    pub struct PPUStatus: u8 {
        const PS_UNUSED1 = 0b0000_0001;
        const PS_UNUSED2 = 0b0000_0010;
        const PS_UNUSED3 = 0b0000_0100;
        const PS_UNUSED4 = 0b0000_1000;
        const PS_UNUSED5 = 0b0001_0000;
        const PS_SPRITE_OVERFLOW = 0b0010_0000;
        const PS_SPRITE_ZERO_HIT = 0b0100_0000;
        const PS_VERTICAL_BLANK = 0b1000_0000;
    }
}

bitflags! {
    #[derive(Copy, Clone, Debug)]
    pub struct PPUMask: u8 {
        const PS_GRAYSCALE = 0b0000_0001;
        const PS_RENDER_BACKGROUND_LEFT = 0b0000_0010;
        const PS_RENDER_SPRITES_LEFT = 0b0000_0100;
        const PS_RENDER_BACKGROUND = 0b0000_1000;
        const PS_RENDER_SPRITES = 0b0001_0000;
        const PS_ENHANCE_RED = 0b0010_0000;
        const PS_ENHANCE_GREEN = 0b0100_0000;
        const PS_ENHANCE_BLUE = 0b1000_0000;
    }
}

bitflags! {
    #[derive(Copy, Clone, Debug)]
    pub struct PPUControl: u8 {
        const PS_NAMETABLE_X = 0b0000_0001;
        const PS_NAMETABLE_Y = 0b0000_0010;
        const PS_INCREMENT_MODE = 0b0000_0100;
        const PS_PATTERN_SPRITE = 0b0000_1000;
        const PS_PATTERN_BACKGROUND = 0b0001_0000;
        const PS_SPRITE_SIZE = 0b0010_0000;
        const PS_SLAVE_MODE = 0b0100_0000;
        const PS_ENABLE_NMI = 0b1000_0000;
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
            address_latch: 0,
            ppu_data_buffer: 0,
            ppu_address: 0,
            addr_hi: 0,
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

                // hack
                self.status.set(PPUStatus::PS_VERTICAL_BLANK, true);

                let data = self.status.bits();
                self.status.set(PPUStatus::PS_VERTICAL_BLANK, false);
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

                let data = self.ppu_data_buffer;
                self.ppu_data_buffer = self.ppu_read(self.ppu_address);

                if self.ppu_address >= 0x3F00 {
                    return self.ppu_data_buffer;
                }

                self.ppu_address += 1;

                data
            },
            _ => panic!("invalid CPU read from PPU")
        }
    }

    pub fn cpu_write(&mut self, addr: u16, data: u8) {
        match addr {
            0x0000 => {
                self.control = PPUControl::from_bits(data).unwrap();
            },
            0x0001 => {
                self.mask = PPUMask::from_bits(data).unwrap();
            },
            0x0002 => (),
            0x0003 => (),
            0x0004 => (),
            0x0005 => (),
            0x0006 => {
                if self.address_latch == 0 {
                    self.addr_hi = data;
                    self.address_latch = 1;
                } else {
                    let lo = data;
                    self.ppu_address = u16::from(self.addr_hi) << 8 | u16::from(lo);
                    println!("setting ppu address = {:04X}", self.ppu_address);
                    self.address_latch = 0;
                }
            },
            0x0007 => {
                //println!("[ppu] set {:04X} = {:02X}", self.ppu_address, data);
                self.ppu_write(self.ppu_address, data);
                self.ppu_address += 1;
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
            // TODO
            return 0x00;
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
            // this works... why
            // let mut addr = addr & 0x001F;
            // if addr == 0x0010 { addr = 0x0000 };
            // if addr == 0x0014 { addr = 0x0004 };
            // if addr == 0x0018 { addr = 0x0008 };
            // if addr == 0x001C { addr = 0x000C };

            // self.tbl_palette[addr as usize] = data;
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

    pub fn clock(&mut self) {
        let p: u8 = rand::thread_rng().gen_range(0..=1);
        self.spr_screen.set_pixel(self.cycle - 1, self.scanline, if p == 0 { self.pal_screen[0x3F] } else { self.pal_screen[0x30] });

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

        if idx != 0 {
            println!("requesting palette data from {:02X}", idx as usize & 0x3F);
        }

        self.pal_screen[idx as usize & 0x3F]
    }
}
