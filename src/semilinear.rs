// Semi-linear sets

use crate::kleene::Kleene;

pub struct Linear {
    base: Vec<i64>,
    gens: Vec<Vec<i64>>,
}

pub struct SemiLinear {
    lins: Vec<Linear>
}

impl Kleene for SemiLinear {
    fn zero() -> Self {
        SemiLinear { lins: vec![] }
    }
    fn one() -> Self {
        todo!()
    }
    fn plus(self, _other: Self) -> Self {
        todo!()
    }
    fn times(self, _other: Self) -> Self {
        todo!()
    }
    fn star(self) -> Self {
        todo!()
    }
}

impl SemiLinear {
    fn complement(self) -> Self {
        todo!()
    }
}
