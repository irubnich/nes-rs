use std::{fs::File, io::{Read, Seek}};

use crate::mapper::Mapper;

#[derive(Debug)]
pub struct Cartridge {
    b_image_valid: bool,
    v_prg_memory: Vec<u8>,
    v_chr_memory: Vec<u8>,
    mapper: Mapper,
}

#[derive(Debug)]
pub struct Header {
    name: [u8; 4],
    prg_rom_chunks: u8,
    chr_rom_chunks: u8,
    mapper1: u8,
    mapper2: u8,
    prg_ram_size: u8,
    tv_system1: u8,
    tv_system2: u8,
    unused: [u8; 5],
}

impl Cartridge {
    pub fn new(filename: String) -> Cartridge {
        let mut file = File::open(filename).expect("file not opened");

        let mut name: [u8; 4] = [0; 4];
        file.read(&mut name).expect("read name");

        let mut readed: [u8; 7] = [0; 7];
        file.read(&mut readed).expect("read");

        let header = Header {
            name,
            prg_rom_chunks: readed[0],
            chr_rom_chunks: readed[1],
            mapper1: readed[2],
            mapper2: readed[3],
            prg_ram_size: readed[4],
            tv_system1: readed[5],
            tv_system2: readed[6],
            unused: [0; 5],
        };

        // 78, 69, 83, 26
        let mut cart = Cartridge {
            b_image_valid: false,
            v_prg_memory: vec![],
            v_chr_memory: vec![],
            mapper: Mapper::new(0, 0),
        };

        if (header.mapper1 & 0x04) > 0 {
            file.seek_relative(512).expect("bad seek");
        }

        let n_mapper_id = ((header.mapper2 >> 4) << 4) | (header.mapper1 >> 4);
        //let mirror = if header.mapper1 & 0x01 > 0 { 1 } else { 0 };

        let n_file_type = 1;
        if n_file_type == 1 {
            cart.v_prg_memory.resize(header.prg_rom_chunks as usize * 16384, 0);
            file.read(&mut cart.v_prg_memory).expect("v_prg_memory");

            cart.v_chr_memory.resize(header.chr_rom_chunks as usize * 8192, 0);
            file.read(&mut cart.v_chr_memory).expect("v_chr_memory");
        }

        if n_mapper_id == 0 {
            cart.mapper = Mapper::new(header.prg_rom_chunks, header.chr_rom_chunks);
        }

        cart.b_image_valid = true;

        cart
    }
}
