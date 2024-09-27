use bitflags::bitflags;

use crate::memory::Memory;

bitflags! {
    pub struct Status: u8 {
        const N = 0b1000_0000;
        const V = 0b0100_0000;
        const _ = 0b0010_0000; // unused
        const B = 0b0001_0000;
        const D = 0b0000_1000;
        const I = 0b0000_0100;
        const Z = 0b0000_0010;
        const C = 0b0000_0001;
    }
}

pub struct CPU {
    // registers
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub pc: u16,
    pub sp: u8,
    pub status: Status,

    pub memory: Memory,
}

impl CPU {
    pub fn new() -> CPU {
        CPU {
            a: 0,
            x: 0,
            y: 0,
            pc: 0,
            sp: 0,
            status: Status::empty(),
            memory: Memory::new(),
        }
    }

    pub fn clock(&self) {

    }

    pub fn reset(&self) {

    }

    pub fn complete(&self) -> bool {
        false
    }

    pub fn get_byte(&self, addr: u16) -> u8 {
        0
    }
}
