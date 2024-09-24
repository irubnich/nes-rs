use bitflags::bitflags;

pub struct StatusArgs {
    pub n: bool,
    pub v: bool,
    pub u: bool,
    pub b: bool,
    pub d: bool,
    pub i: bool,
    pub z: bool,
    pub c: bool,
}

impl StatusArgs {
    pub const fn none() -> StatusArgs {
        StatusArgs {
            n: false,
            v: false,
            u: false,
            b: false,
            d: false,
            i: false,
            z: false,
            c: false,
        }
    }
}

#[derive(Default, Debug, Copy, Clone)]
pub struct StackPointer(pub u8);

impl StackPointer {
    pub const fn to_u16(self) -> u16 {
        let StackPointer(val) = self;
        u16::from_le_bytes([val, 0x01])
    }

    pub fn decrement(&mut self) {
        self.0 = self.0.wrapping_sub(1);
    }

    pub fn increment(&mut self) {
        self.0 = self.0.wrapping_add(1);
    }
}

#[derive(Default)]
pub struct Registers {
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub stkp: StackPointer,
    pub pc: u16,
    pub status: Status,
}

bitflags! {
    #[derive(Copy, Clone, Debug)]
    pub struct Status: u8 {
        const PS_NEGATIVE = 0b1000_0000;
        const PS_OVERFLOW = 0b0100_0000;
        const PS_UNUSED = 0b0010_0000;
        const PS_BRK = 0b0001_0000;
        const PS_DECIMAL_MODE = 0b0000_1000;
        const PS_DISABLE_INTERRUPTS = 0b0000_0100;
        const PS_ZERO = 0b0000_0010;
        const PS_CARRY = 0b0000_0001;
    }
}

impl Status {
    pub fn new(
        StatusArgs {
            n, v, u, b, d, i, z, c,
        }: StatusArgs,
    ) -> Status {
        let mut out = Status::empty();

        if n {
            out |= Status::PS_NEGATIVE;
        }

        if v {
            out |= Status::PS_OVERFLOW;
        }

        if u {
            out |= Status::PS_UNUSED;
        }

        if b {
            out |= Status::PS_BRK;
        }

        if d {
            out |= Status::PS_DECIMAL_MODE;
        }

        if i {
            out |= Status::PS_DISABLE_INTERRUPTS;
        }

        if z {
            out |= Status::PS_ZERO;
        }

        if c {
            out |= Status::PS_CARRY;
        }

        out
    }

    pub fn and(&mut self, rhs: Status) {
        *self &= rhs;
    }

    pub fn or(&mut self, rhs: Status) {
        *self |= rhs;
    }

    pub fn set_with_mask(&mut self, mask: Status, rhs: Status) {
        *self = (*self & !mask) | rhs;
    }
}

impl Default for Status {
    fn default() -> Self {
        Status::new(StatusArgs {
            n: false,
            v: false,
            u: true,
            b: false,
            d: false,
            i: true,
            z: false,
            c: false,
        })
    }
}

impl Registers {
    pub fn new() -> Registers {
        Registers {
            a: 0,
            x: 0,
            y: 0,
            stkp: StackPointer(0),
            pc: 0,
            status: Status::default(),
        }
    }
}