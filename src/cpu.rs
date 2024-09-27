use std::fmt::Write;
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
        const U = 1 << 5; // unused
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
    pub disasm: String,
}

impl CPU {
    const SP_BASE: u16 = 0x0100;

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
            disasm: String::with_capacity(100),
        };

        cpu.status.set(Status::I, true);

        cpu
    }

    pub fn clock(&mut self) -> usize {
        self.trace_instr();

        let start_cycle = self.cycle;

        let opcode = self.read_instr();
        self.instr = CPU::INSTRUCTIONS[opcode as usize];

        match self.instr.addr_mode() {
            IMM => self.imm(),
            ZP0 => self.zp0(),
            ABS => self.abs(),
            x => panic!("unimplemented addr mode {:?}", x)
        }

        match self.instr.op() {
            LDX => self.ldx(),
            STX => self.stx(),
            JSR => self.jsr(),
            JMP => self.jmp(),
            NOP => self.nop(),
            x => panic!("unimplemented op {:?}", x)
        }

        let cycles_ran = self.cycle - start_cycle;
        cycles_ran
    }

    pub fn reset(&mut self) {
        self.a = 0;
        self.x = 0;
        self.y = 0;
        self.sp = 0xFD;
        self.status = Status::U.union(Status::I);

        self.cycle = 0;

        let lo = self.bus.cpu_read(0xFFFC, true);
        let hi = self.bus.cpu_read(0xFFFD, true);
        self.pc = u16::from_le_bytes([lo, hi]);

        // 7 cycles
        for _ in 0..7 {
            self.start_cycle();
            self.end_cycle();
        }
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

    pub fn peek(&mut self, addr: u16) -> u8 {
        self.bus.cpu_read(addr, true)
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        self.start_cycle();
        self.bus.cpu_write(addr, data);
        self.end_cycle();
    }

    // read opcode and increment PC
    pub fn read_instr(&mut self) -> u8 {
        let val = self.read(self.pc);
        self.pc = self.pc.wrapping_add(1);
        val
    }

    pub fn read_instr_u16(&mut self) -> u16 {
        let lo = self.read_instr();
        let hi = self.read_instr();
        u16::from_le_bytes([lo, hi])
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
                _ => unreachable!("not possible")
            };

            // re-read data if page boundary was crossed
            if (self.abs_addr & 0x00FF) < u16::from(reg) {
                self.fetched_data = self.read(self.abs_addr);
            }
        } else {
            self.fetch_data();
        }
    }

    pub fn set_zn_status(&mut self, val: u8) {
        self.status.set(Status::Z, val == 0x00);
        self.status.set(Status::N, val & 0x80 == 0x80);
    }

    pub fn push(&mut self, val: u8) {
        self.write(Self::SP_BASE | u16::from(self.sp), val);
        self.sp = self.sp.wrapping_sub(1);
    }

    pub fn push_u16(&mut self, val: u16) {
        let [lo, hi] = val.to_le_bytes();
        self.push(hi);
        self.push(lo);
    }

    fn start_cycle(&mut self) {
        self.cycle = self.cycle.wrapping_add(1);
    }

    fn end_cycle(&mut self) {
        // later
    }

    fn trace_instr(&mut self) {
        let pc = self.pc;
        let acc = self.a;
        let x = self.x;
        let y = self.y;
        let sp = self.sp;
        let cycle = self.cycle;
        let st = self.status.bits();

        let ppu_cycle = 0; // todo
        let ppu_scanline= 0; // todo

        println!("{:<50} A:{acc:02X} X:{x:02X} Y:{y:02X} P:{st:02X} SP:{sp:02X} PPU:{ppu_cycle:3},{ppu_scanline:3} CYC:{cycle}", self.disassemble(pc));
    }

    fn disassemble(&mut self, pc: u16) -> &str {
        self.disasm.clear();

        let opcode = self.peek(pc);
        let instr = CPU::INSTRUCTIONS[opcode as usize].op();

        let _ = write!(self.disasm, "{pc:04X}  {opcode:02X} ");
        let _ = write!(self.disasm, "        {instr:?}");

        &self.disasm
    }
}
