use std::fs;

use nom::bytes::complete::{tag, take};
use nom::number::complete::u8;
use nom::IResult;
use nom::error::Error;

use crate::mapper::Mapper;

#[derive(Debug)]
pub struct Cartridge {
    v_prg_memory: Vec<u8>,
    v_chr_memory: Vec<u8>,
    mapper: Mapper,
}

impl Cartridge {
    pub fn new(filename: String) -> Cartridge {
        let binding = fs::read(filename).expect("can't read cart");
        let buf: &[u8] = binding.as_slice();
        let cart = Cartridge::parse(buf);
        cart.expect("can't parse cart").1
    }

    pub fn parse(i: &[u8]) -> IResult<&[u8], Self> {
        let (i, _) = tag::<&[u8; 4], &[u8], Error<&[u8]>>(b"NES\x1A")(i).expect("nes");
        let (i, prg_banks) = u8::<&[u8], Error<&[u8]>>(i).expect("prg_banks");
        let (i, chr_banks) = u8::<&[u8], Error<&[u8]>>(i).expect("chr_banks");
        let (i, flags_6) = u8::<&[u8], Error<&[u8]>>(i).expect("flags_6");
        let (i, flags_7) = u8::<&[u8], Error<&[u8]>>(i).expect("flags_7");
        let (i, size_prg_ram) = u8::<&[u8], Error<&[u8]>>(i).expect("size_prg_ram");
        let (i, _flags_9) = u8::<&[u8], Error<&[u8]>>(i).expect("_flags_9");
        let (i, _flags_10) = u8::<&[u8], Error<&[u8]>>(i).expect("_flags_11");
        let (i, _) = take::<usize, &[u8], Error<&[u8]>>(5usize)(i).expect("unused");
        let (i, prg) = take::<usize, &[u8], Error<&[u8]>>(0x4000 * prg_banks as usize)(i).expect("prg");
        let (i, chr) = take::<usize, &[u8], Error<&[u8]>>(0x2000 * prg_banks as usize)(i).expect("chr");

        let n_mapper_id = (flags_6 >> 4) | (flags_7 & 0xF0);
        if n_mapper_id != 0 {
            panic!("unsupported mapper ID")
        }

        let cart = Cartridge {
            v_prg_memory: prg.to_vec(),
            v_chr_memory: chr.to_vec(),
            mapper: Mapper::new(prg_banks, chr_banks),
        };

        Ok((i, cart))
    }

    pub fn cpu_read(&self, addr: u16) -> (bool, u8) {
        match self.mapper.cpu_map_read(addr) {
            (true, mapped_addr) => {
                return (true, self.v_prg_memory[mapped_addr as usize]);
            }
            _ => (false, 0)
        }
    }

    pub fn cpu_write(&mut self, addr: u16, data: u8) -> bool {
        match self.mapper.cpu_map_write(addr) {
            (true, mapped_addr) => {
                self.v_prg_memory[mapped_addr as usize] = data;
                true
            }
            _ => false
        }
    }

    pub fn ppu_read(&self, addr: u16) -> (bool, u8) {
        match self.mapper.ppu_map_read(addr) {
            (true, mapped_addr) => {
                return (true, self.v_chr_memory[mapped_addr as usize])
            }
            _ => (false, 0)
        }
    }

    pub fn ppu_write(&mut self, addr: u16, data: u8) -> bool {
        match self.mapper.ppu_map_write(addr) {
            (true, mapped_addr) => {
                self.v_chr_memory[mapped_addr as usize] = data;
                true
            }
            _ => false
        }
    }
}
