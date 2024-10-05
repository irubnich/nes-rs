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
    mask: Mask,
    control: Control,
    scroll: Scroll,
    pub nmi: bool,

    address_latch: u8,
    ppu_data_buffer: u8,
    fine_x: u8,

    prev_palette: u8,
    curr_palette: u8,
    next_palette: u8,
    tile_lo: u8,
    tile_hi: u8,
    tile_addr: u16,

    tile_shift_lo: u16,
    tile_shift_hi: u16,
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
    #[derive(Copy, Clone, Debug, Default)]
    pub struct PPUMask: u8 {
        const GRAYSCALE = 1;
        const SHOW_LEFT_BG = 1 << 1;
        const SHOW_LEFT_SPR = 1 << 2;
        const SHOW_BG = 1 << 3;
        const SHOW_SPR = 1 << 4;
        const EMPHASIZE_RED = 1 << 5;
        const EMPHASIZE_GREEN = 1 << 6;
        const EMPHASIZE_BLUE = 1 << 7;
    }
}

#[derive(Debug, Default)]
pub struct Mask {
    pub rendering_enabled: bool,
    pub grayscale: u8,
    pub emphasis: u16,
    pub show_left_bg: bool,
    pub show_left_spr: bool,
    pub show_bg: bool,
    pub show_spr: bool,
    bits: PPUMask,
}

impl Mask {
    pub fn new() -> Self {
        let mut mask = Self {
            ..Default::default()
        };
        mask.write(0);
        mask
    }

    pub fn write(&mut self, val: u8) {
        self.bits = PPUMask::from_bits_truncate(val);
        self.grayscale = if self.bits.contains(PPUMask::GRAYSCALE) {
            0x30
        } else {
            0x3F
        };
        self.show_left_bg = self.bits.contains(PPUMask::SHOW_LEFT_BG);
        self.show_left_spr = self.bits.contains(PPUMask::SHOW_LEFT_SPR);
        self.show_bg = self.bits.contains(PPUMask::SHOW_BG);
        self.show_spr = self.bits.contains(PPUMask::SHOW_SPR);
        self.rendering_enabled = self.show_bg || self.show_spr;
        self.emphasis = 0;
    }
}

bitflags! {
    #[derive(Copy, Clone, Debug, Default)]
    pub struct PPUControl: u8 {
        const NAMETABLE1 = 0b0000_0001;
        const NAMETABLE2 = 0b0000_0010;
        const VRAM_INCREMENT = 0b0000_0100;
        const SPR_SELECT = 0b0000_1000;
        const BG_SELECT = 0b0001_0000;
        const SPR_HEIGHT = 0b0010_0000;
        const MASTER_SLAVE = 0b0100_0000;
        const NMI_ENABLE = 0b1000_0000;
    }
}

#[derive(Default, Debug)]
pub struct Control {
    pub spr_select: u16,
    pub bg_select: u16,
    pub spr_height: u32,
    pub master_slave: u8,
    pub nmi_enabled: bool,
    pub nametable_addr: u16,
    pub vram_increment: u16,
    bits: PPUControl,
}

impl Control {
    pub fn new() -> Self {
        let mut ctrl = Self::default();
        ctrl.write(0);
        ctrl
    }

    pub fn write(&mut self, val: u8) {
        self.bits = PPUControl::from_bits_truncate(val);
        self.spr_select = self.bits.contains(PPUControl::SPR_SELECT) as u16 * 0x1000;
        self.bg_select = self.bits.contains(PPUControl::BG_SELECT) as u16 * 0x1000;
        self.spr_height = self.bits.contains(PPUControl::SPR_HEIGHT) as u32 * 8 + 8;
        self.master_slave = self.bits.contains(PPUControl::MASTER_SLAVE) as u8;
        self.nmi_enabled = self.bits.contains(PPUControl::NMI_ENABLE);
        self.nametable_addr = match self.bits.bits() & 0b11 {
            0b00 => 0x01,
            0b01 => 0x02,
            0b10 => 0x2800,
            0b11 => 0x2C00,
            _ => unreachable!("impossible"),
        };
        self.vram_increment = self.bits.contains(PPUControl::VRAM_INCREMENT) as u16 * 31 + 1;
    }
}

#[derive(Debug)]
pub struct Scroll {
    pub fine_x: u16,
    pub coarse_x: u16,
    pub fine_y: u16,
    pub coarse_y: u16,
    pub v: u16,
    pub t: u16,
    pub write_latch: bool,
}

impl Scroll {
    pub const fn new() -> Self {
        Self {
            v: 0,
            t: 0,
            fine_x: 0,
            coarse_x: 0,
            fine_y: 0,
            coarse_y: 0,
            write_latch: false,
        }
    }

    pub const fn attr_addr(&self) -> u16 {
        let nametable_select = self.v & (0x0400 | 0x0800);
        let y_bits = (self.v >> 4) & 0x38;
        let x_bits = (self.v >> 2) & 0x07;
        0x23C0 | nametable_select | y_bits | x_bits
    }

    pub const fn attr_shift(&self) -> u16 {
        (self.v & 0x02) | ((self.v >> 4) & 0x04)
    }

    pub const fn addr(&self) -> u16 {
        self.v & 0x3FFF
    }

    pub fn set_v(&mut self, val: u16) {
        self.v = val;
        self.coarse_x = self.v & 0x001F;
        self.fine_y = self.v >> 12;
        self.coarse_y = (self.v & 0x03E0) >> 5;
    }

    pub fn increment_x(&mut self) {
        if (self.v & 0x001F) == 31 {
            self.set_v((self.v & !0x001F) ^ 0x0400);
        } else {
            self.set_v(self.v + 1);
        }
    }

    pub fn increment_y(&mut self) {
        if (self.v & 0x7000) == 0x7000 {
            self.set_v(self.v & !0x7000);
            let mut y = (self.v & 0x03E0) >> 5;
            if y == 29 {
                y = 0;
                self.set_v(self.v ^ 0x0800);
            } else if y == 31 {
                y = 0;
            } else {
                y += 1;
            }

            self.set_v((self.v & !0x03E0) | (y << 5));
        } else {
            self.set_v(self.v + 0x1000);
        }
    }

    pub fn copy_x(&mut self) {
        let x_mask = 0x0400 | 0x001F;
        self.set_v((self.v & !x_mask) | (self.t & x_mask));
    }

    pub fn copy_y(&mut self) {
        let y_mask = 0x7000 | 0x0800 | 0x03E0;
        self.set_v((self.v & !y_mask) | (self.t & y_mask));
    }

    pub fn write_nametable_select(&mut self, val: u8) {
        let nt_mask = 0x0800 | 0x0400;
        self.t = (self.t & !nt_mask) | (u16::from(val) & 0x03) << 10;
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
            mask: Mask::new(),
            control: Control::new(),
            scroll: Scroll::new(),
            address_latch: 0,
            ppu_data_buffer: 0,
            nmi: false,
            fine_x: 0,
            prev_palette: 0,
            curr_palette: 0,
            next_palette: 0,
            tile_lo: 0,
            tile_hi: 0,
            tile_addr: 0,
            tile_shift_hi: 0,
            tile_shift_lo: 0,
        };

        ppu
    }

    pub fn cpu_read(&mut self, addr: u16, read_only: bool) -> u8 {
        if read_only {
            println!("read only");
        }

        match addr {
            0x0000 => 0x00,
            0x0001 => 0x00,
            0x0002 => {
                let data = (self.status.bits() & 0xE0) | (self.ppu_data_buffer & 0x1F);

                if read_only {
                    return data;
                }

                self.stop_vblank();
                self.address_latch = 0;

                self.ppu_data_buffer |= self.status.bits() & 0xE0;

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

                let addr = self.scroll.addr();
                self.increment_vram_addr();

                let val= self.ppu_read(addr);
                let val = if addr < 0x3F00 {
                    val
                } else {
                    self.ppu_data_buffer = self.ppu_read(addr - 0x1000);
                    val | (self.ppu_data_buffer & 0xC0)
                };

                val
            },
            _ => panic!("invalid CPU read from PPU")
        }
    }

    pub fn cpu_write(&mut self, addr: u16, data: u8) {
        match addr {
            0x0000 => {
                self.control.write(data);
                self.scroll.write_nametable_select(data);
            },
            0x0001 => {
                self.mask.write(data);
            },
            0x0002 => (),
            0x0003 => (),
            0x0004 => (),
            0x0005 => {
                let val = u16::from(data);
                let lo_5_bit_mask: u16 = 0x1F;
                let fine_mask: u16 = 0x07;
                let fine_rshift = 3;

                if self.scroll.write_latch {
                    let coarse_y_lshift = 5;
                    let fine_y_lshift = 12;
                    self.scroll.t = self.scroll.t & !(0x7000 | 0x03E0)
                        | (((val >> fine_rshift) & lo_5_bit_mask) << coarse_y_lshift)
                        | ((val & fine_mask) << fine_y_lshift);
                } else {
                    self.scroll.t = self.scroll.t & !0x001F
                        | ((val >> fine_rshift) & lo_5_bit_mask);
                    self.scroll.fine_x = val & fine_mask;
                }

                self.scroll.write_latch = !self.scroll.write_latch;
            },
            0x0006 => {
                if self.scroll.write_latch {
                    let lo_bits_mask = 0x7F00;
                    self.scroll.t = (self.scroll.t & lo_bits_mask) | u16::from(data);
                    self.scroll.v = self.scroll.t;
                } else {
                    let hi_bits_mask = 0x00FF;
                    let six_bits_mask = 0x003F;
                    self.scroll.t = (self.scroll.t & hi_bits_mask) | ((u16::from(data) & six_bits_mask) << 8);
                }

                self.scroll.write_latch = !self.scroll.write_latch;
            },
            0x0007 => {
                let addr = self.scroll.addr();
                self.increment_vram_addr();
                self.ppu_write(addr, data);
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
        self.mask = Mask::new();
        self.control = Control::new();
    }

    pub fn increment_vram_addr(&mut self) {
        if self.mask.rendering_enabled && (self.scanline == 261 || self.scanline <= 239) {
            self.scroll.increment_x();
            self.scroll.increment_y();
        } else {
            self.scroll.set_v(self.scroll.v.wrapping_add(self.control.vram_increment));
        }
    }

    pub fn fetch_bg_nt_byte(&mut self) {
        self.prev_palette = self.curr_palette;
        self.curr_palette = self.next_palette;

        self.tile_shift_lo |= u16::from(self.tile_lo);
        self.tile_shift_hi |= u16::from(self.tile_hi);

        let addr = 0x2000 | (self.scroll.addr() & 0x0FFF);
        let tile_index = u16::from(self.ppu_read(addr));

        self.tile_addr = self.control.bg_select | (tile_index << 4) | self.scroll.fine_y;
    }

    pub fn fetch_bg_attr_byte(&mut self) {
        let addr = self.scroll.attr_addr();
        let shift = self.scroll.attr_shift();
        self.next_palette = ((self.ppu_read(addr) >> shift) & 0x03) << 2;
    }

    pub fn fetch_background(&mut self) {
        match self.cycle & 0x07 {
            1 => self.fetch_bg_nt_byte(),
            3 => self.fetch_bg_attr_byte(),
            5 => self.tile_lo = self.ppu_read(self.tile_addr),
            7 => self.tile_hi = self.ppu_read(self.tile_addr + 8),
            _ => (),
        }
    }

    pub fn tick(&mut self) {
        let cycle = self.cycle;
        let scanline = self.scanline;
        let visible_cycle = matches!(cycle, 1..=256);
        let bg_prefetch_cycle = matches!(cycle, 321..=336);
        let bg_fetch_cycle = bg_prefetch_cycle || visible_cycle;
        let visible_scanline = scanline <= 239;
        let prerender_scanline = self.scanline == 261;

        if self.mask.rendering_enabled {
            let render_scanline = visible_scanline || prerender_scanline;

            if render_scanline {
                let bg_dummy_cycle = matches!(cycle, 337..=340);

                if bg_fetch_cycle {
                    self.fetch_background();

                    if cycle & 0x07 == 0x00 {
                        self.scroll.increment_x();
                    }
                } else if bg_dummy_cycle {
                    self.fetch_bg_nt_byte();
                }

                if prerender_scanline {
                    match cycle {
                        280..=304 => {
                            self.scroll.copy_y();
                        },
                        _ => (),
                    }
                }

                match cycle {
                    256 => self.scroll.increment_y(),
                    257 => self.scroll.copy_x(),
                    _ => (),
                }
            }
        }

        if visible_cycle && visible_cycle { // && !skip_rendering
            self.render_pixel();
        }

        if bg_fetch_cycle {
            self.tile_shift_lo <<= 1;
            self.tile_shift_hi <<= 1;
        }
    }

    fn start_vblank(&mut self) {
        self.status.set(PPUStatus::VERTICAL_BLANK, true);
        if self.control.nmi_enabled {
            self.nmi = true;
        }
    }

    fn stop_vblank(&mut self) {
        self.status.set(PPUStatus::VERTICAL_BLANK, false);
    }

    pub fn clock(&mut self) -> usize {
        let prerender_scanline = 261; // 331 alternative
        let vblank_scanline = 241; // 291 alternative

        if self.cycle >= 340 {
            self.cycle = 0;
            self.scanline += 1;
            if self.scanline == vblank_scanline - 1 {
                // this is the post render scanline - do nothing?
                // self.frame.increment();
            } else {
                self.scanline *= (self.scanline <= prerender_scanline) as i32;
            }
        } else {
            self.cycle += 1;
            self.tick();

            if self.cycle == 1 {
                if self.scanline == vblank_scanline {
                    self.start_vblank();
                } else if self.scanline == prerender_scanline {
                    self.stop_vblank();
                    self.scanline = -1;
                    self.frame_complete = true;
                }
            }
        }

        1
    }

    fn render_pixel(&mut self) {
        let x = self.cycle - 1;
        let y = self.scanline;
        let addr = self.scroll.addr();

        let color = if self.mask.rendering_enabled || (addr & 0x3F00) != 0x3F00 {
            let color = u16::from(self.pixel_color());
            self.ppu_read(0x3F00 + (color & 0x03 > 0) as u16 * color)
        } else {
            self.ppu_read(addr)
        };

        let pixel = self.pal_screen[(u16::from(color & self.mask.grayscale) | self.mask.emphasis) as usize & 0x3F];
        self.spr_screen.set_pixel(x, y, pixel);
    }

    fn pixel_color(&self) -> u8 {
        let x = self.cycle - 1;

        let left_clip_bg = x < 8 && !self.mask.show_left_bg;
        let bg_color = if self.mask.show_bg && !left_clip_bg {
            ((((self.tile_shift_hi << self.fine_x) & 0x8000) >> 14)
                | (((self.tile_shift_lo << self.fine_x) & 0x8000) >> 15)) as u8
        } else {
            0
        };

        if (u16::from(self.fine_x) + ((x & 0x07) as u16)) < 8 {
            self.prev_palette + bg_color
        } else {
            self.curr_palette + bg_color
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
