use bitflags::bitflags;

pub struct StatusArgs {
    pub N: bool,
    pub V: bool,
    pub U: bool,
    pub B: bool,
    pub D: bool,
    pub I: bool,
    pub Z: bool,
    pub C: bool,
}

impl StatusArgs {
    pub const fn none() -> StatusArgs {
        StatusArgs {
            N: false,
            V: false,
            U: false,
            B: false,
            D: false,
            I: false,
            Z: false,
            C: false,
        }
    }
}

pub struct StackPointer(pub u8);

pub struct Registers {
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub stkp: StackPointer,
    pub pc: u16,
    pub status: Status,
}

bitflags! {
    #[derive(Copy, Clone)]
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
            N, V, U, B, D, I, Z, C,
        }: StatusArgs,
    ) -> Status {
        let mut out = Status::empty();

        if N {
            out |= Status::PS_NEGATIVE;
        }

        if V {
            out |= Status::PS_OVERFLOW;
        }

        if U {
            out |= Status::PS_UNUSED;
        }

        if B {
            out |= Status::PS_BRK;
        }

        if D {
            out |= Status::PS_DECIMAL_MODE;
        }

        if I {
            out |= Status::PS_DISABLE_INTERRUPTS;
        }

        if Z {
            out |= Status::PS_ZERO;
        }

        if C {
            out |= Status::PS_CARRY;
        }

        out
    }

    pub fn set_with_mask(&mut self, mask: Status, rhs: Status) {
        *self = (*self & !mask) | rhs;
    }
}

impl Default for Status {
    fn default() -> Self {
        Status::new(StatusArgs {
            N: false,
            V: false,
            U: true,
            B: false,
            D: false,
            I: true,
            Z: false,
            C: false,
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