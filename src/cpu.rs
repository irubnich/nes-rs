use std::{fmt::Write, ops::Sub};
use bitflags::bitflags;

use crate::{bus::Bus, memory::Memory};

pub mod instr;
use instr::{
    AddrMode::{ABS, ABX, ABY, ACC, IDX, IDY, IMM, IMP, IND, REL, ZP0, ZPX, ZPY},
    Instr,
    Operation::{
        ADC, AND, ASL, BCC, BCS, BEQ, BIT, BMI, BNE, BPL, BRK, BVC, BVS,
        CLC, CLD, CLV, CMP, CPX, CPY, DCP, DEC, DEX, DEY, EOR, IGN, INC, INX, INY, ISB, JMP,
        JSR, LAX, LDA, LDX, LDY, LSR, NOP, ORA, PHA, PHP, PLA, PLP, RLA, ROL, ROR, RRA, RTI,
        RTS, SAX, SBC, SEC, SED, SEI, SKB, SLO, SRE, STA, STX, STY, TAX, TAY, TSX,
        TXA, TXS, TYA, XXX,

        // unimplemented
        // AHX, ALR, ANC, ARR, AXS, CLI, LAS, SXA, SYA, TAS, XAA
    },
};

bitflags! {
    #[derive(Copy, Clone)]
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
    pub clock_count: usize,
    pub cycles_remaining: usize,
    pub abs_addr: u16,
    pub rel_addr: u16,
    pub instr: Instr,
    pub fetched_data: u8,
    pub disasm: String,

    // interrupts
    pub prev_run_irq: bool,
    pub nmi: bool,
    pub prev_nmi: bool,

    // misc
    pub corrupted: bool,
}

impl CPU {
    const SP_BASE: u16 = 0x0100;
    const NMI_VECTOR: u16 = 0xFFFA;
    const IRQ_VECTOR: u16 = 0xFFFE;

    pub fn new(bus: Bus) -> CPU {
        let mut cpu = CPU {
            a: 0,
            x: 0,
            y: 0,
            pc: 0xFFFC,
            sp: 0xFD,
            status: Status::empty(),
            memory: Memory::new(),
            clock_count: 0,
            cycles_remaining: 0,
            bus,
            abs_addr: 0,
            rel_addr: 0,
            instr: CPU::INSTRUCTIONS[0x00],
            fetched_data: 0,
            disasm: String::with_capacity(100),
            nmi: false,
            prev_nmi: false,
            prev_run_irq: false,
            corrupted: false,
        };

        cpu.status.set(Status::I, true);

        cpu
    }

    pub fn clock(&mut self) -> usize {
        if self.cycles_remaining > 0 {
            self.cycles_remaining -= 1;
            return 0;
        }

        //self.trace_instr();

        let start_cycle = self.clock_count;

        let opcode = self.read_instr();
        self.instr = CPU::INSTRUCTIONS[opcode as usize];

        match self.instr.addr_mode() {
            IMP => self.imp(),
            IMM => self.imm(),
            REL => self.rel(),
            ZP0 => self.zp0(),
            ABS => self.abs(),
            ACC => self.acc(),
            IDX => self.idx(),
            IDY => self.idy(),
            IND => self.ind(),
            ABX => self.abx(),
            ABY => self.aby(),
            ZPX => self.zpx(),
            ZPY => self.zpy(),
        }

        match self.instr.op() {
            LDA => self.lda(),
            LDX => self.ldx(),
            LDY => self.ldy(),
            STA => self.sta(),
            STX => self.stx(),
            STY => self.sty(),
            SEI => self.sei(),
            SED => self.sed(),
            AND => self.and(),
            ORA => self.ora(),
            EOR => self.eor(),
            ADC => self.adc(),
            CMP => self.cmp(),
            CPY => self.cpy(),
            CPX => self.cpx(),
            JSR => self.jsr(),
            SEC => self.sec(),
            SBC => self.sbc(),
            INC => self.inc(),
            DEC => self.dec(),
            INY => self.iny(),
            INX => self.inx(),
            DEY => self.dey(),
            DEX => self.dex(),
            TAY => self.tay(),
            TAX => self.tax(),
            TYA => self.tya(),
            TXA => self.txa(),
            TSX => self.tsx(),
            TXS => self.txs(),
            RTI => self.rti(),
            LSR => self.lsr(),
            ASL => self.asl(),
            ROR => self.ror(),
            ROL => self.rol(),
            CLC => self.clc(),
            CLD => self.cld(),
            CLV => self.clv(),
            BCS => self.bcs(),
            BCC => self.bcc(),
            BIT => self.bit(),
            BEQ => self.beq(),
            BVS => self.bvs(),
            BVC => self.bvc(),
            BPL => self.bpl(),
            BNE => self.bne(),
            BMI => self.bmi(),
            PHP => self.php(),
            PHA => self.pha(),
            PLA => self.pla(),
            PLP => self.plp(),
            JMP => self.jmp(),
            RTS => self.rts(),
            SKB => self.skb(),
            IGN => self.ign(),
            LAX => self.lax(),
            SAX => self.sax(),
            DCP => self.dcp(),
            ISB => self.isb(),
            SLO => self.slo(),
            RLA => self.rla(),
            SRE => self.sre(),
            RRA => self.rra(),
            BRK => self.brk(),
            NOP => self.nop(),
            XXX => self.xxx(),
            x => panic!("unimplemented op {:?}", x)
        }

        if self.prev_run_irq || self.prev_nmi {
            self.irq();
        }

        let cycles_ran = self.clock_count - start_cycle;

        self.cycles_remaining = cycles_ran;

        cycles_ran
    }

    pub fn reset(&mut self) {
        self.a = 0;
        self.x = 0;
        self.y = 0;
        self.sp = 0xFD;
        self.status = Status::U.union(Status::I);

        self.clock_count = 0;
        self.cycles_remaining = 0;
        self.prev_run_irq = false;
        self.nmi = false;
        self.prev_nmi = false;

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
        self.cycles_remaining == 0
    }

    pub fn irq(&mut self) {
        self.read(self.pc);
        self.read(self.pc);
        self.push_u16(self.pc);

        let status = ((self.status | Status::U) & !Status::B).bits();

        if self.nmi {
            self.nmi = false;
            self.push(status);
            self.status.set(Status::I, true);

            self.pc = self.read_u16(Self::NMI_VECTOR);
        } else {
            self.push(status);
            self.status.set(Status::I, true);

            self.pc = self.read_u16(Self::IRQ_VECTOR);
        }
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

    pub fn read_u16(&mut self, addr: u16) -> u16 {
        let lo = self.read(addr);
        let hi = self.read(addr.wrapping_add(1));
        u16::from_le_bytes([lo, hi])
    }

    pub fn read_zp_u16(&mut self, addr: u8) -> u16 {
        let lo = self.read(addr.into());
        let hi = self.read(addr.wrapping_add(1).into());
        u16::from_le_bytes([lo, hi])
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        self.start_cycle();
        self.bus.cpu_write(addr, data);
        self.end_cycle();
    }

    fn write_fetched(&mut self, val: u8) {
        match self.instr.addr_mode() {
            IMP | ACC => self.a = val,
            IMM => (),
            _ => self.write(self.abs_addr, val),
        }
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

    pub fn pop_u16(&mut self) -> u16 {
        let lo = self.pop();
        let hi = self.pop();
        u16::from_le_bytes([lo, hi])
    }

    pub fn pop(&mut self) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        self.read(Self::SP_BASE | u16::from(self.sp))
    }

    fn start_cycle(&mut self) {
        self.clock_count = self.clock_count.wrapping_add(1);
    }

    fn end_cycle(&mut self) {
        // later
    }

    pub fn trace_instr(&mut self) {
        let pc = self.pc;
        let acc = self.a;
        let x = self.x;
        let y = self.y;
        let sp = self.sp;
        let cycle = self.clock_count;
        let st = (self.status | Status::U).sub(Status::B); // remove U and B

        let ppu_cycle = 0; // todo
        let ppu_scanline= 0; // todo

        println!("{:<50} A:{acc:02X} X:{x:02X} Y:{y:02X} P:{st:02X} SP:{sp:02X} PPU:{ppu_cycle:3},{ppu_scanline:3} CYC:{cycle}", self.disassemble(pc));
    }

    pub fn disassemble(&mut self, pc: u16) -> &str {
        self.disasm.clear();

        let opcode = self.peek(pc);
        let instr = CPU::INSTRUCTIONS[opcode as usize].op();

        let _ = write!(self.disasm, "{pc:04X}  {opcode:02X} ");
        let _ = write!(self.disasm, "        {instr:?}");

        &self.disasm
    }

    fn pages_differ(addr1: u16, addr2: u16) -> bool {
        (addr1 & 0xFF00) != (addr2 & 0xFF00)
    }

    fn status_bit(&self, reg: Status) -> u8 {
        self.status.intersection(reg).bits()
    }
}
