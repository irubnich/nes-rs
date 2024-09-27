use bitflags::bitflags;

use crate::{bus::Bus, memory::Memory};

pub mod instr;

bitflags! {
    pub struct Status: u8 {
        const N = 1 << 7;
        const V = 1 << 6;
        const _ = 1 << 5; // unused
        const B = 1 << 4;
        const D = 1 << 3;
        const I = 1 << 2;
        const Z = 1 << 1;
        const C = 1;
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

    // memory
    pub memory: Memory,
    pub bus: Bus,

    // state
    pub cycles: usize,
}

impl CPU {
    pub fn new(bus: Bus) -> CPU {
        let mut cpu = CPU {
            a: 0,
            x: 0,
            y: 0,
            pc: 0xFFFC,
            sp: 0xFD,
            status: Status::empty(),
            memory: Memory::new(),
            cycles: 0,
            bus,
        };

        cpu.status.set(Status::I, true);

        cpu
    }

    pub fn clock(&mut self) {
        self.cycles += 1;
    }

    pub fn reset(&mut self) {
        self.pc = 0xFFFC;
        self.sp = self.sp - 3;
        self.status.set(Status::I, true);
    }

    pub fn complete(&self) -> bool {
        self.cycles == 0
    }

    pub fn read(&mut self, addr: u16) -> u8 {
        self.bus.cpu_read(addr, false)
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        self.bus.cpu_write(addr, data);
    }

    // read opcode and increment PC
    pub fn read_instr(&mut self) -> u8 {
        let val = self.read(self.pc);
        self.pc = self.pc.wrapping_add(1);
        val
    }
}
