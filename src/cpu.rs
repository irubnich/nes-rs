use rs6502::Variant;
use rs6502::memory::Bus;
use rs6502::registers::Registers;

pub struct CPU<M, V>
where
    M: Bus,
    V: Variant,
{
    pub registers: Registers,
    pub memory: M,
    variant: core::marker::PhantomData<V>,
}

impl<M: Bus, V: Variant> CPU<M, V> {
    pub fn new(memory: M, _variant: V) -> CPU<M, V> {
        CPU {
            registers: Registers::new(),
            memory,
            variant: core::marker::PhantomData::<V>,
        }
    }
}
