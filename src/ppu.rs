use olc_pixel_game_engine::{self as olc, Pixel};

pub struct PPU {
    //tbl_name: [[u8; 1024]; 2],
    //tbl_pattern: [[u8; 4096]; 2],
    //tbl_palette: [u8; 32],

    pal_screen: Vec<olc::Pixel>,

    spr_screen: olc::Sprite,
    spr_name_table: [olc::Sprite; 2],
    spr_pattern_table: [olc::Sprite; 2],

    frame_complete: bool,
    scanline: i16,
    cycle: i16,
}

impl PPU {
    pub fn new() -> PPU {
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

            spr_screen: olc::Sprite::with_dims(256, 240),
            spr_name_table: [
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
        };

        ppu
    }
}
