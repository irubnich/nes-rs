use bitflags::bitflags;

use crate::{bus::Bus, memory::Memory};

pub mod instr;
use instr::{
    AddrMode::{ABS, ABX, ABY, ACC, IDX, IDY, IMM, IMP, IND, REL, ZP0, ZPX, ZPY},
    Instr,
    Operation::{
        ADC, AHX, ALR, ANC, AND, ARR, ASL, AXS, BCC, BCS, BEQ, BIT, BMI, BNE, BPL, BRK, BVC, BVS,
        CLC, CLD, CLI, CLV, CMP, CPX, CPY, DCP, DEC, DEX, DEY, EOR, IGN, INC, INX, INY, ISB, JMP,
        JSR, LAS, LAX, LDA, LDX, LDY, LSR, NOP, ORA, PHA, PHP, PLA, PLP, RLA, ROL, ROR, RRA, RTI,
        RTS, SAX, SBC, SEC, SED, SEI, SKB, SLO, SRE, STA, STX, STY, SXA, SYA, TAS, TAX, TAY, TSX,
        TXA, TXS, TYA, XAA, XXX,
    },
};

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
    pub cycle: usize,
    pub abs_addr: u16,
    pub instr: Instr,
    pub fetched_data: u8,
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
            cycle: 0,
            bus,
            abs_addr: 0,
            instr: CPU::INSTRUCTIONS[0x00],
            fetched_data: 0,
        };

        cpu.status.set(Status::I, true);

        cpu
    }

    pub fn clock(&mut self) -> usize {
        let start_cycle = self.cycle;

        let opcode = self.read_instr();
        self.instr = CPU::INSTRUCTIONS[opcode as usize];

        match self.instr.addr_mode() {
            IMM => self.imm(),
            ZP0 => self.zp0(),
            x => panic!("unimplemented addr mode {:?}", x)
        }

        match self.instr.op() {
            NOP => self.nop(),
            x => panic!("unimplemented op {:?}", x)
        }

        let cycles_ran = self.cycle - start_cycle;
        cycles_ran
    }

    pub fn reset(&mut self) {
        self.pc = 0xFFFC;
        self.sp = self.sp - 3;
        self.status.set(Status::I, true);
    }

    pub fn complete(&self) -> bool {
        self.cycle == 0
    }

    pub fn read(&mut self, addr: u16) -> u8 {
        self.start_cycle();
        let val = self.bus.cpu_read(addr, false);
        self.end_cycle();
        val
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

    pub fn fetch_data(&mut self) {
        self.fetched_data = if matches!(self.instr.addr_mode(), IMP | ACC) {
            self.a
        } else {
            self.read(self.abs_addr)
        }
    }

    // fetch data taking into account page boundary crossing
    pub fn fetch_data_cross(&mut self) {
        let mode = self.instr.addr_mode();
        if matches!(mode, ABX | ABY | IDY) {
            let reg = match mode {
                ABX => self.x,
                ABY | IDY => self.y,
                _ => unreachable!("not possible"),
            };

            // re-read data if page boundary was crossed
            if (self.abs_addr & 0x00FF) < u16::from(reg) {
                self.fetched_data = self.read(self.abs_addr);
            }
        } else {
            self.fetch_data();
        }
    }

    fn start_cycle(&mut self) {
        self.cycle = self.cycle.wrapping_add(1);
    }

    fn end_cycle(&mut self) {
        // later
    }
}
