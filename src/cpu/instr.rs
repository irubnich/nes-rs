use crate::cpu::Status;
use crate::cpu::CPU;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum AddrMode {
    IMM,
    ZP0, ZPX, ZPY,
    ABS, ABX, ABY,
    IND, IDX, IDY,
    REL, ACC, IMP,
}

#[derive(Copy, Clone, Debug)]
pub enum Operation {
    ADC, AND, ASL, BCC, BCS, BEQ, BIT, BMI, BNE, BPL, BRK, BVC, BVS, CLC, CLD, CLI, CLV, CMP, CPX,
    CPY, DEC, DEX, DEY, EOR, INC, INX, INY, JMP, JSR, LDA, LDX, LDY, LSR, NOP, ORA, PHA, PHP, PLA,
    PLP, ROL, ROR, RTI, RTS, SBC, SEC, SED, SEI, STA, STX, STY, TAX, TAY, TSX, TXA, TXS, TYA,
    // "Unofficial" opcodes
    SKB, IGN, ISB, DCP, AXS, LAS, LAX, AHX, SAX, XAA, SXA, RRA, TAS, SYA, ARR, SRE, ALR, RLA, ANC,
    SLO, XXX
}

// (opcode, Addressing Mode, Operation, cycles taken)
#[derive(Copy, Clone, Debug)]
pub struct Instr(u8, AddrMode, Operation, usize);

impl Instr {
    pub fn opcode(&self) -> u8 {
        self.0
    }

    pub fn addr_mode(&self) -> AddrMode {
        self.1
    }

    pub fn op(&self) -> Operation {
        self.2
    }

    pub fn cycles(&self) -> usize {
        self.3
    }
}

use AddrMode::{ABS, ABX, ABY, ACC, IDX, IDY, IMM, IMP, IND, REL, ZP0, ZPX, ZPY};
use Operation::{
    ADC, AHX, ALR, ANC, AND, ARR, ASL, AXS, BCC, BCS, BEQ, BIT, BMI, BNE, BPL, BRK, BVC, BVS, CLC,
    CLD, CLI, CLV, CMP, CPX, CPY, DCP, DEC, DEX, DEY, EOR, IGN, INC, INX, INY, ISB, JMP, JSR, LAS,
    LAX, LDA, LDX, LDY, LSR, NOP, ORA, PHA, PHP, PLA, PLP, RLA, ROL, ROR, RRA, RTI, RTS, SAX, SBC,
    SEC, SED, SEI, SKB, SLO, SRE, STA, STX, STY, SXA, SYA, TAS, TAX, TAY, TSX, TXA, TXS, TYA, XAA,
    XXX,
};

impl CPU {
    pub const INSTRUCTIONS: [Instr; 256] = [
        Instr(0x00, IMM, BRK, 7), Instr(0x01, IDX, ORA, 6), Instr(0x02, IMP, XXX, 2), Instr(0x03, IDX, SLO, 8), Instr(0x04, ZP0, NOP, 3), Instr(0x05, ZP0, ORA, 3), Instr(0x06, ZP0, ASL, 5), Instr(0x07, ZP0, SLO, 5), Instr(0x08, IMP, PHP, 3), Instr(0x09, IMM, ORA, 2), Instr(0x0A, ACC, ASL, 2), Instr(0x0B, IMM, ANC, 2), Instr(0x0C, ABS, NOP, 4), Instr(0x0D, ABS, ORA, 4), Instr(0x0E, ABS, ASL, 6), Instr(0x0F, ABS, SLO, 6),
        Instr(0x10, REL, BPL, 2), Instr(0x11, IDY, ORA, 5), Instr(0x12, IMP, XXX, 2), Instr(0x13, IDY, SLO, 8), Instr(0x14, ZPX, NOP, 4), Instr(0x15, ZPX, ORA, 4), Instr(0x16, ZPX, ASL, 6), Instr(0x17, ZPX, SLO, 6), Instr(0x18, IMP, CLC, 2), Instr(0x19, ABY, ORA, 4), Instr(0x1A, IMP, NOP, 2), Instr(0x1B, ABY, SLO, 7), Instr(0x1C, ABX, IGN, 4), Instr(0x1D, ABX, ORA, 4), Instr(0x1E, ABX, ASL, 7), Instr(0x1F, ABX, SLO, 7),
        Instr(0x20, ABS, JSR, 6), Instr(0x21, IDX, AND, 6), Instr(0x22, IMP, XXX, 2), Instr(0x23, IDX, RLA, 8), Instr(0x24, ZP0, BIT, 3), Instr(0x25, ZP0, AND, 3), Instr(0x26, ZP0, ROL, 5), Instr(0x27, ZP0, RLA, 5), Instr(0x28, IMP, PLP, 4), Instr(0x29, IMM, AND, 2), Instr(0x2A, ACC, ROL, 2), Instr(0x2B, IMM, ANC, 2), Instr(0x2C, ABS, BIT, 4), Instr(0x2D, ABS, AND, 4), Instr(0x2E, ABS, ROL, 6), Instr(0x2F, ABS, RLA, 6),
        Instr(0x30, REL, BMI, 2), Instr(0x31, IDY, AND, 5), Instr(0x32, IMP, XXX, 2), Instr(0x33, IDY, RLA, 8), Instr(0x34, ZPX, NOP, 4), Instr(0x35, ZPX, AND, 4), Instr(0x36, ZPX, ROL, 6), Instr(0x37, ZPX, RLA, 6), Instr(0x38, IMP, SEC, 2), Instr(0x39, ABY, AND, 4), Instr(0x3A, IMP, NOP, 2), Instr(0x3B, ABY, RLA, 7), Instr(0x3C, ABX, IGN, 4), Instr(0x3D, ABX, AND, 4), Instr(0x3E, ABX, ROL, 7), Instr(0x3F, ABX, RLA, 7),
        Instr(0x40, IMP, RTI, 6), Instr(0x41, IDX, EOR, 6), Instr(0x42, IMP, XXX, 2), Instr(0x43, IDX, SRE, 8), Instr(0x44, ZP0, NOP, 3), Instr(0x45, ZP0, EOR, 3), Instr(0x46, ZP0, LSR, 5), Instr(0x47, ZP0, SRE, 5), Instr(0x48, IMP, PHA, 3), Instr(0x49, IMM, EOR, 2), Instr(0x4A, ACC, LSR, 2), Instr(0x4B, IMM, ALR, 2), Instr(0x4C, ABS, JMP, 3), Instr(0x4D, ABS, EOR, 4), Instr(0x4E, ABS, LSR, 6), Instr(0x4F, ABS, SRE, 6),
        Instr(0x50, REL, BVC, 2), Instr(0x51, IDY, EOR, 5), Instr(0x52, IMP, XXX, 2), Instr(0x53, IDY, SRE, 8), Instr(0x54, ZPX, NOP, 4), Instr(0x55, ZPX, EOR, 4), Instr(0x56, ZPX, LSR, 6), Instr(0x57, ZPX, SRE, 6), Instr(0x58, IMP, CLI, 2), Instr(0x59, ABY, EOR, 4), Instr(0x5A, IMP, NOP, 2), Instr(0x5B, ABY, SRE, 7), Instr(0x5C, ABX, IGN, 4), Instr(0x5D, ABX, EOR, 4), Instr(0x5E, ABX, LSR, 7), Instr(0x5F, ABX, SRE, 7),
        Instr(0x60, IMP, RTS, 6), Instr(0x61, IDX, ADC, 6), Instr(0x62, IMP, XXX, 2), Instr(0x63, IDX, RRA, 8), Instr(0x64, ZP0, NOP, 3), Instr(0x65, ZP0, ADC, 3), Instr(0x66, ZP0, ROR, 5), Instr(0x67, ZP0, RRA, 5), Instr(0x68, IMP, PLA, 4), Instr(0x69, IMM, ADC, 2), Instr(0x6A, ACC, ROR, 2), Instr(0x6B, IMM, ARR, 2), Instr(0x6C, IND, JMP, 5), Instr(0x6D, ABS, ADC, 4), Instr(0x6E, ABS, ROR, 6), Instr(0x6F, ABS, RRA, 6),
        Instr(0x70, REL, BVS, 2), Instr(0x71, IDY, ADC, 5), Instr(0x72, IMP, XXX, 2), Instr(0x73, IDY, RRA, 8), Instr(0x74, ZPX, NOP, 4), Instr(0x75, ZPX, ADC, 4), Instr(0x76, ZPX, ROR, 6), Instr(0x77, ZPX, RRA, 6), Instr(0x78, IMP, SEI, 2), Instr(0x79, ABY, ADC, 4), Instr(0x7A, IMP, NOP, 2), Instr(0x7B, ABY, RRA, 7), Instr(0x7C, ABX, IGN, 4), Instr(0x7D, ABX, ADC, 4), Instr(0x7E, ABX, ROR, 7), Instr(0x7F, ABX, RRA, 7),
        Instr(0x80, IMM, SKB, 2), Instr(0x81, IDX, STA, 6), Instr(0x82, IMM, SKB, 2), Instr(0x83, IDX, SAX, 6), Instr(0x84, ZP0, STY, 3), Instr(0x85, ZP0, STA, 3), Instr(0x86, ZP0, STX, 3), Instr(0x87, ZP0, SAX, 3), Instr(0x88, IMP, DEY, 2), Instr(0x89, IMM, SKB, 2), Instr(0x8A, IMP, TXA, 2), Instr(0x8B, IMM, XAA, 2), Instr(0x8C, ABS, STY, 4), Instr(0x8D, ABS, STA, 4), Instr(0x8E, ABS, STX, 4), Instr(0x8F, ABS, SAX, 4),
        Instr(0x90, REL, BCC, 2), Instr(0x91, IDY, STA, 6), Instr(0x92, IMP, XXX, 2), Instr(0x93, IDY, AHX, 6), Instr(0x94, ZPX, STY, 4), Instr(0x95, ZPX, STA, 4), Instr(0x96, ZPY, STX, 4), Instr(0x97, ZPY, SAX, 4), Instr(0x98, IMP, TYA, 2), Instr(0x99, ABY, STA, 5), Instr(0x9A, IMP, TXS, 2), Instr(0x9B, ABY, TAS, 5), Instr(0x9C, ABX, SYA, 5), Instr(0x9D, ABX, STA, 5), Instr(0x9E, ABY, SXA, 5), Instr(0x9F, ABY, AHX, 5),
        Instr(0xA0, IMM, LDY, 2), Instr(0xA1, IDX, LDA, 6), Instr(0xA2, IMM, LDX, 2), Instr(0xA3, IDX, LAX, 6), Instr(0xA4, ZP0, LDY, 3), Instr(0xA5, ZP0, LDA, 3), Instr(0xA6, ZP0, LDX, 3), Instr(0xA7, ZP0, LAX, 3), Instr(0xA8, IMP, TAY, 2), Instr(0xA9, IMM, LDA, 2), Instr(0xAA, IMP, TAX, 2), Instr(0xAB, IMM, LAX, 2), Instr(0xAC, ABS, LDY, 4), Instr(0xAD, ABS, LDA, 4), Instr(0xAE, ABS, LDX, 4), Instr(0xAF, ABS, LAX, 4),
        Instr(0xB0, REL, BCS, 2), Instr(0xB1, IDY, LDA, 5), Instr(0xB2, IMP, XXX, 2), Instr(0xB3, IDY, LAX, 5), Instr(0xB4, ZPX, LDY, 4), Instr(0xB5, ZPX, LDA, 4), Instr(0xB6, ZPY, LDX, 4), Instr(0xB7, ZPY, LAX, 4), Instr(0xB8, IMP, CLV, 2), Instr(0xB9, ABY, LDA, 4), Instr(0xBA, IMP, TSX, 2), Instr(0xBB, ABY, LAS, 4), Instr(0xBC, ABX, LDY, 4), Instr(0xBD, ABX, LDA, 4), Instr(0xBE, ABY, LDX, 4), Instr(0xBF, ABY, LAX, 4),
        Instr(0xC0, IMM, CPY, 2), Instr(0xC1, IDX, CMP, 6), Instr(0xC2, IMM, SKB, 2), Instr(0xC3, IDX, DCP, 8), Instr(0xC4, ZP0, CPY, 3), Instr(0xC5, ZP0, CMP, 3), Instr(0xC6, ZP0, DEC, 5), Instr(0xC7, ZP0, DCP, 5), Instr(0xC8, IMP, INY, 2), Instr(0xC9, IMM, CMP, 2), Instr(0xCA, IMP, DEX, 2), Instr(0xCB, IMM, AXS, 2), Instr(0xCC, ABS, CPY, 4), Instr(0xCD, ABS, CMP, 4), Instr(0xCE, ABS, DEC, 6), Instr(0xCF, ABS, DCP, 6),
        Instr(0xD0, REL, BNE, 2), Instr(0xD1, IDY, CMP, 5), Instr(0xD2, IMP, XXX, 2), Instr(0xD3, IDY, DCP, 8), Instr(0xD4, ZPX, NOP, 4), Instr(0xD5, ZPX, CMP, 4), Instr(0xD6, ZPX, DEC, 6), Instr(0xD7, ZPX, DCP, 6), Instr(0xD8, IMP, CLD, 2), Instr(0xD9, ABY, CMP, 4), Instr(0xDA, IMP, NOP, 2), Instr(0xDB, ABY, DCP, 7), Instr(0xDC, ABX, IGN, 4), Instr(0xDD, ABX, CMP, 4), Instr(0xDE, ABX, DEC, 7), Instr(0xDF, ABX, DCP, 7),
        Instr(0xE0, IMM, CPX, 2), Instr(0xE1, IDX, SBC, 6), Instr(0xE2, IMM, SKB, 2), Instr(0xE3, IDX, ISB, 8), Instr(0xE4, ZP0, CPX, 3), Instr(0xE5, ZP0, SBC, 3), Instr(0xE6, ZP0, INC, 5), Instr(0xE7, ZP0, ISB, 5), Instr(0xE8, IMP, INX, 2), Instr(0xE9, IMM, SBC, 2), Instr(0xEA, IMP, NOP, 2), Instr(0xEB, IMM, SBC, 2), Instr(0xEC, ABS, CPX, 4), Instr(0xED, ABS, SBC, 4), Instr(0xEE, ABS, INC, 6), Instr(0xEF, ABS, ISB, 6),
        Instr(0xF0, REL, BEQ, 2), Instr(0xF1, IDY, SBC, 5), Instr(0xF2, IMP, XXX, 2), Instr(0xF3, IDY, ISB, 8), Instr(0xF4, ZPX, NOP, 4), Instr(0xF5, ZPX, SBC, 4), Instr(0xF6, ZPX, INC, 6), Instr(0xF7, ZPX, ISB, 6), Instr(0xF8, IMP, SED, 2), Instr(0xF9, ABY, SBC, 4), Instr(0xFA, IMP, NOP, 2), Instr(0xFB, ABY, ISB, 7), Instr(0xFC, ABX, IGN, 4), Instr(0xFD, ABX, SBC, 4), Instr(0xFE, ABX, INC, 7), Instr(0xFF, ABX, ISB, 7),
    ];

    //
    // addressing modes
    //

    pub fn imp(&mut self) {
        let _ = self.read(self.pc);
    }

    pub fn abs(&mut self) {
        self.abs_addr = self.read_instr_u16();
    }

    pub fn imm(&mut self) {
        self.abs_addr = self.pc;
        self.pc = self.pc.wrapping_add(1);
    }

    pub fn rel(&mut self) {
        self.rel_addr = u16::from(self.read_instr());
    }

    pub fn zp0(&mut self) {
        self.abs_addr = u16::from(self.read_instr());
    }

    pub fn acc(&mut self) {
        let _ = self.read(self.pc);
    }

    pub fn idx(&mut self) {
        let addr = self.read_instr();
        let _ = self.read(u16::from(addr));
        let addr = addr.wrapping_add(self.x);
        self.abs_addr = self.read_zp_u16(addr);
    }

    pub fn idy(&mut self) {
        let addr = self.read_instr();
        let addr = self.read_zp_u16(addr);
        self.abs_addr = addr.wrapping_add(self.y.into());
        self.fetched_data = self.read((addr & 0xFF00) | (self.abs_addr & 0x00FF));
    }

    pub fn ind(&mut self) {
        let addr = self.read_instr_u16();
        if addr & 0xFF == 0xFF {
            // buggy indirect
            let lo = self.read(addr);
            let hi = self.read(addr & 0xFF00);
            self.abs_addr = u16::from_le_bytes([lo, hi])
        } else {
            self.abs_addr = self.read_u16(addr);
        }
    }

    pub fn abx(&mut self) {
        let addr = self.read_instr_u16();
        self.abs_addr = addr.wrapping_add(self.x.into());
        self.fetched_data = self.read((addr & 0xFF00) | (self.abs_addr & 0x00FF));
    }

    pub fn aby(&mut self) {
        let addr = self.read_instr_u16();
        self.abs_addr = addr.wrapping_add(self.y.into());
        self.fetched_data = self.read((addr & 0xFF00) | (self.abs_addr & 0x00FF));
    }

    pub fn zpx(&mut self) {
        let addr = u16::from(self.read_instr());
        let _ = self.read(addr);
        self.abs_addr = addr.wrapping_add(self.x.into()) & 0x00FF;
    }

    pub fn zpy(&mut self) {
        let addr = u16::from(self.read_instr());
        let _ = self.read(addr);
        self.abs_addr = addr.wrapping_add(self.y.into()) & 0x00FF;
    }

    //
    // operations
    //

    pub fn sec(&mut self) {
        self.status.set(Status::C, true);
    }

    pub fn clc(&mut self) {
        self.status.set(Status::C, false);
    }

    pub fn cld(&mut self) {
        self.status.set(Status::D, false);
    }

    pub fn clv(&mut self) {
        self.status.set(Status::V, false);
    }

    pub fn sei(&mut self) {
        self.status.set(Status::I, true);
    }

    pub fn sed(&mut self) {
        self.status.set(Status::D, true);
    }

    pub fn jmp(&mut self) {
        self.pc = self.abs_addr;
    }

    pub fn lda(&mut self) {
        self.fetch_data_cross();
        self.a = self.fetched_data;
        self.set_zn_status(self.a);
    }

    pub fn ldx(&mut self) {
        self.fetch_data_cross();
        self.x = self.fetched_data;
        self.set_zn_status(self.x);
    }

    pub fn ldy(&mut self) {
        self.fetch_data_cross();
        self.y = self.fetched_data;
        self.set_zn_status(self.y);
    }

    pub fn sta(&mut self) {
        self.write(self.abs_addr, self.a);
    }

    pub fn stx(&mut self) {
        self.write(self.abs_addr, self.x);
    }

    pub fn sty(&mut self) {
        self.write(self.abs_addr, self.y);
    }

    pub fn bit(&mut self) {
        self.fetch_data_cross();
        let val = self.a & self.fetched_data;
        self.status.set(Status::Z, val == 0);
        self.status.set(Status::N, self.fetched_data & (1 << 7) > 0);
        self.status.set(Status::V, self.fetched_data & (1 << 6) > 0);
    }

    pub fn bcc(&mut self) {
        if !self.status.intersects(Status::C) {
            self.branch();
        }
    }

    pub fn bcs(&mut self) {
        if self.status.intersects(Status::C) {
            self.branch();
        }
    }

    pub fn bvs(&mut self) {
        if self.status.intersects(Status::V) {
            self.branch();
        }
    }

    pub fn bvc(&mut self) {
        if !self.status.intersects(Status::V) {
            self.branch();
        }
    }

    pub fn bpl(&mut self) {
        if !self.status.intersects(Status::N) {
            self.branch();
        }
    }

    pub fn beq(&mut self) {
        if self.status.intersects(Status::Z) {
            self.branch();
        }
    }

    pub fn bne(&mut self) {
        if !self.status.intersects(Status::Z) {
            self.branch();
        }
    }

    pub fn bmi(&mut self) {
        if self.status.intersects(Status::N) {
            self.branch();
        }
    }

    pub fn ora(&mut self) {
        self.fetch_data_cross();
        self.a |= self.fetched_data;
        self.set_zn_status(self.a);
    }

    pub fn eor(&mut self) {
        self.fetch_data_cross();
        self.a ^= self.fetched_data;
        self.set_zn_status(self.a);
    }

    pub fn adc(&mut self) {
        self.fetch_data_cross();
        let a = self.a;
        let (x1, o1) = self.fetched_data.overflowing_add(a);
        let (x2, o2) = x1.overflowing_add(self.status_bit(Status::C));
        self.a = x2;
        self.status.set(Status::C, o1 | o2);
        self.status.set(
            Status::V,
            (a ^ self.fetched_data) & 0x80 == 0 && (a ^ self.a) & 0x80 != 0,
        );
        self.set_zn_status(self.a);
    }

    pub fn lsr(&mut self) {
        self.fetch_data();
        self.write_fetched(self.fetched_data);
        self.status.set(Status::C, self.fetched_data & 1 > 0);
        let val = self.fetched_data.wrapping_shr(1);
        self.set_zn_status(val);
        self.write_fetched(val);
    }

    pub fn asl(&mut self) {
        self.fetch_data();
        self.write_fetched(self.fetched_data);
        self.status.set(Status::C, (self.fetched_data >> 7) & 1 > 0);
        let val = self.fetched_data.wrapping_shl(1);
        self.set_zn_status(val);
        self.write_fetched(val);
    }

    pub fn ror(&mut self) {
        self.fetch_data();
        self.write_fetched(self.fetched_data);
        let mut ret = self.fetched_data.rotate_right(1);
        if self.status.intersects(Status::C) {
            ret |= 1 << 7;
        } else {
            ret &= !(1 << 7);
        }
        self.status.set(Status::C, self.fetched_data & 1 > 0);
        self.set_zn_status(ret);
        self.write_fetched(ret);
    }

    pub fn rol(&mut self) {
        self.fetch_data();
        self.write_fetched(self.fetched_data);
        let old_c = self.status_bit(Status::C);
        self.status.set(Status::C, (self.fetched_data >> 7) & 1 > 0);
        let val = (self.fetched_data << 1) | old_c;
        self.set_zn_status(val);
        self.write_fetched(val);
    }

    pub fn php(&mut self) {
        self.push((self.status | Status::U | Status::B).bits());
    }

    pub fn pha(&mut self) {
        self.push(self.a);
    }

    pub fn pla(&mut self) {
        let _ = self.read(Self::SP_BASE | u16::from(self.sp));
        self.a = self.pop();
        self.set_zn_status(self.a);
    }

    pub fn plp(&mut self) {
        let _ = self.read(Self::SP_BASE | u16::from(self.sp));
        self.status = Status::from_bits_truncate(self.pop()).difference(Status::U);
    }

    pub fn and(&mut self) {
        self.fetch_data_cross();
        self.a &= self.fetched_data;
        self.set_zn_status(self.a);
    }

    pub fn cmp(&mut self) {
        self.fetch_data_cross();
        self.compare(self.a, self.fetched_data);
    }

    pub fn cpy(&mut self) {
        self.fetch_data();
        self.compare(self.y, self.fetched_data);
    }

    pub fn cpx(&mut self) {
        self.fetch_data();
        self.compare(self.x, self.fetched_data);
    }

    pub fn jsr(&mut self) {
        let _ = self.read(Self::SP_BASE | u16::from(self.sp));
        self.push_u16(self.pc.wrapping_sub(1));
        self.pc = self.abs_addr;
    }

    pub fn rts(&mut self) {
        self.pc = self.pop_u16().wrapping_add(1);
    }

    pub fn sbc(&mut self) {
        self.fetch_data_cross();
        let a = self.a;
        let (x1, o1) = a.overflowing_sub(self.fetched_data);
        let (x2, o2) = x1.overflowing_sub(1 - self.status_bit(Status::C));
        self.a = x2;
        self.status.set(Status::C, !(o1 | o2));
        self.status.set(
            Status::V,
            (a ^ self.fetched_data) & 0x80 != 0 && (a ^ self.a) & 0x80 != 0,
        );
        self.set_zn_status(self.a);
    }

    pub fn inc(&mut self) {
        self.fetch_data();
        self.write_fetched(self.fetched_data);
        let val = self.fetched_data.wrapping_add(1);
        self.set_zn_status(val);
        self.write_fetched(val);
    }

    pub fn dec(&mut self) {
        self.fetch_data();
        self.write_fetched(self.fetched_data);
        let val = self.fetched_data.wrapping_sub(1);
        self.set_zn_status(val);
        self.write_fetched(val);
    }

    pub fn iny(&mut self) {
        self.y = self.y.wrapping_add(1);
        self.set_zn_status(self.y);
    }

    pub fn inx(&mut self) {
        self.x = self.x.wrapping_add(1);
        self.set_zn_status(self.x);
    }

    pub fn dey(&mut self) {
        self.y = self.y.wrapping_sub(1);
        self.set_zn_status(self.y);
    }

    pub fn dex(&mut self) {
        self.x = self.x.wrapping_sub(1);
        self.set_zn_status(self.x);
    }

    pub fn tay(&mut self) {
        self.y = self.a;
        self.set_zn_status(self.y);
    }

    pub fn tax(&mut self) {
        self.x = self.a;
        self.set_zn_status(self.x);
    }

    pub fn tya(&mut self) {
        self.a = self.y;
        self.set_zn_status(self.a);
    }

    pub fn txa(&mut self) {
        self.a = self.x;
        self.set_zn_status(self.a);
    }

    pub fn tsx(&mut self) {
        self.x = self.sp;
        self.set_zn_status(self.x);
    }

    pub fn txs(&mut self) {
        self.sp = self.x;
    }

    pub fn rti(&mut self) {
        let _ = self.read(Self::SP_BASE | u16::from(self.sp));
        self.status = Status::from_bits_truncate(self.pop());
        self.status &= !Status::U;
        self.status &= !Status::B;
        self.pc = self.pop_u16();
    }

    pub fn nop(&mut self) {
        self.fetch_data_cross();
    }

    pub fn skb(&mut self) {
        self.fetch_data();
    }

    pub fn ign(&mut self) {
        self.fetch_data_cross();
    }

    pub fn lax(&mut self) {
        self.lda();
        self.tax();
    }

    pub fn sax(&mut self) {
        if self.instr.addr_mode() == IDY {
            self.fetch_data();
        }
        let val = self.a & self.x;
        self.write_fetched(val);
    }

    pub fn dcp(&mut self) {
        self.fetch_data();
        self.write_fetched(self.fetched_data);
        let val = self.fetched_data.wrapping_sub(1);
        self.compare(self.a, val);
        self.write_fetched(val);
    }

    pub fn isb(&mut self) {
        self.fetch_data();
        self.write_fetched(self.fetched_data);
        let val = self.fetched_data.wrapping_add(1);
        let a = self.a;
        let (x1, o1) = a.overflowing_sub(val);
        let (x2, o2) = x1.overflowing_sub(1 - self.status_bit(Status::C));
        self.a = x2;
        self.status.set(Status::C, !(o1 | o2));
        self.status.set(
            Status::V,
            (a ^ val) & 0x80 != 0 && (a ^ self.a) & 0x80 != 0,
        );
        self.set_zn_status(self.a);
        self.write_fetched(val);
    }

    pub fn slo(&mut self) {
        self.fetch_data();
        self.write_fetched(self.fetched_data);
        self.status.set(Status::C, (self.fetched_data >> 7) & 1 > 0);
        let val = self.fetched_data.wrapping_shl(1);
        self.write_fetched(val);
        self.a |= val;
        self.set_zn_status(self.a);
    }

    pub fn rla(&mut self) {
        self.fetch_data();
        self.write_fetched(self.fetched_data);
        let old_c = self.status_bit(Status::C);
        self.status.set(Status::C, (self.fetched_data >> 7) & 1 > 0);
        let val = (self.fetched_data << 1) | old_c;
        self.a &= val;
        self.set_zn_status(self.a);
        self.write_fetched(val);
    }

    pub fn sre(&mut self) {
        self.fetch_data();
        self.write_fetched(self.fetched_data);
        self.status.set(Status::C, self.fetched_data & 1 > 0);
        let val = self.fetched_data.wrapping_shr(1);
        self.a ^= val;
        self.set_zn_status(self.a);
        self.write_fetched(val);
    }

    pub fn rra(&mut self) {
        self.fetch_data();
        self.write_fetched(self.fetched_data);
        let mut ret = self.fetched_data.rotate_right(1);
        if self.status.intersects(Status::C) {
            ret |= 1 << 7;
        } else {
            ret &= !(1 << 7);
        }
        self.status.set(Status::C, self.fetched_data & 1 > 0);
        let a = self.a;
        let (x1, o1) = ret.overflowing_add(a);
        let (x2, o2) = x1.overflowing_add(self.status_bit(Status::C));
        self.a = x2;
        self.status.set(Status::C, o1 | o2);
        self.status.set(
            Status::V,
            (a ^ ret) & 0x80 == 0 && (a ^ self.a) & 0x80 != 0,
        );
        self.set_zn_status(self.a);
        self.write_fetched(ret);
    }

    pub fn brk(&mut self) {
        self.pc += 1;

        self.status.set(Status::I, true);

        self.push_u16(self.pc);

        self.status.set(Status::B, true);
        self.push(self.status.bits());
        self.status.set(Status::B, false);

        self.pc = self.read_u16(0xFFFE);
    }

    pub fn tas(&mut self) {
        self.write(self.abs_addr, self.a);
        self.sp = self.x;
    }

    pub fn sxa(&mut self) {
        let hi = (self.abs_addr >> 8) as u8;
        let lo = (self.abs_addr & 0xFF) as u8;
        let val = self.x & hi.wrapping_add(1);
        self.abs_addr = u16::from_le_bytes([lo, self.x & hi.wrapping_add(1)]);
        self.write_fetched(val);
    }

    pub fn cli(&mut self) {
        self.status.set(Status::I, false);
    }

    pub fn xxx(&mut self) {
        self.corrupted = true;
        panic!("corrupted");
    }

    fn branch(&mut self) {
        self.read(self.pc);

        self.abs_addr = if self.rel_addr & 0x80 == 0x80 {
            self.pc.wrapping_add(self.rel_addr | 0xFF00)
        } else {
            self.pc.wrapping_add(self.rel_addr)
        };

        if Self::pages_differ(self.abs_addr, self.pc) {
            self.read(self.pc);
        }

        self.pc = self.abs_addr;
    }

    fn compare(&mut self, a: u8, b: u8) {
        let result = a.wrapping_sub(b);
        self.set_zn_status(result);
        self.status.set(Status::C, a >= b);
    }
}
