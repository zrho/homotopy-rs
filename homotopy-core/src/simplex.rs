use std::ops::Add;
use std::{collections::HashSet, iter::Sum};

use crate::common::Generator;
use crate::diagram::Diagram;
use crate::diagram::Diagram::*;

/// A (directed) Simplex is a sequence of Generator, with dimension equal to the number of elements
pub type Simplex = Vec<Generator>;

/// A Complex is a set of Simplex
#[derive(Default)]
pub struct Complex(HashSet<Simplex>);

/// Two instances of Complex can be merged by union on their sets
impl Add for Complex {
    type Output = Complex;

    fn add(self, rhs: Self) -> Self::Output {
        Complex(self.0.union(&rhs.0).cloned().collect())
    }
}

impl Sum for Complex {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Default::default(), |acc, x| acc + x)
    }
}

impl From<Diagram> for Complex {
    fn from(diagram: Diagram) -> Self {
        match diagram {
            Diagram0(g) => Complex(vec![vec![g]].into_iter().collect()),
            DiagramN(internal) => {
                let slices = internal.slices();
                let disjoint_complex: Complex = slices.map(Complex::from).sum();
                unimplemented!()
            }
        }
    }
}
