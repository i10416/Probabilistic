use super::utils::collection_like::*;
use std::{
    collections::hash_map::{DefaultHasher, RandomState},
    hash::{BuildHasher, Hash, Hasher},
    marker::PhantomData,
};

struct Register {
    // represent registers' states
    groups: Vec<u8>,
    mask: usize,
    chomp: u8,
}

impl Register {
    fn new(m: usize, mask: usize, chomp: u8) -> Self {
        Self {
            groups: vec![0; m],
            mask: mask,
            chomp: chomp,
        }
    }

    // return whether any group has 0b000...000
    fn count_group_with_zero(&self) -> usize {
        self.groups.iter().filter(|&g| *g == 0).count()
    }
}

impl Insertable<u64> for Register {
    fn insert(&mut self, element: u64) {
        // element is hash value of type u64.
        //bbbb.....01..10
        let group_id = element as usize & self.mask;
        //0000.......bbbb
        let w = element >> self.chomp; // chomp group id and add left padding
        let first_non_zero_occurence = (w.leading_zeros() + 1) as u8;
        let k = &mut self.groups[group_id];
        if *k < first_non_zero_occurence {
            *k = first_non_zero_occurence;
        }
    }
}

impl<T: Hash, H: Hasher + Clone> Insertable<T> for HyperLogLog<T, H> {
    fn insert(&mut self, element: T) {
        let x = self.get_hash(&element);
        self.register.insert(x);
    }
}

pub struct HyperLogLog<T: ?Sized, H: Hasher> {
    // group element by lower `b` bits
    // b should be within [4,16]
    _marker: PhantomData<T>,
    register: Register,
    alpha: f64,
    group_size: usize,
    hasher: H,
}

pub enum Estimator {
    HyperLoglog,
    LinearCounting,
}

// implementation on DefaultHasher
impl<T> HyperLogLog<T, DefaultHasher> {
    // b: group id bit length
    fn new(b: u8) -> Self {
        // TODO(i10416): use compile time validation to check b is within [4,16]
        let (m, r, a) = Self::initialize(b);

        Self {
            _marker: PhantomData,
            register: r,
            group_size: m,
            hasher: RandomState::new().build_hasher(),
            alpha: a,
        }
    }
}

impl<T, H: Hasher> HyperLogLog<T, H> {
    fn initialize(b: u8) -> (usize, Register, f64) {
        let m = 1 << b;
        let mask = m - 1;
        let r = Register::new(m, mask, b);
        let a = Self::get_alpha(b);
        (m, r, a)
    }

    fn get_alpha(b: u8) -> f64 {
        match b {
            4 => 0.673,
            5 => 0.697,
            6 => 0.709,
            _ => 0.7213 / (1.0 + 1.079 / (1 << b) as f64),
        }
    }

    pub fn cardinality(&self) -> f64 {
        let (est, _) = self.estimate_cardinality();
        est
    }

    fn estimate_cardinality(&self) -> (f64, Estimator) {
        let est =
            Self::hyperloglog_estimate(self.alpha, self.group_size as f64, &self.register.groups);

        if est < (5.0 / 2.0 * self.group_size as f64) {
            match self.register.count_group_with_zero() {
                0 => (est, Estimator::HyperLoglog),
                n => (
                    Self::linear_counting_estimate(self.group_size as f64, n as f64),
                    Estimator::LinearCounting,
                ),
            }
        } else {
            (est, Estimator::HyperLoglog)
        }
    }

    fn hyperloglog_estimate(alpha: f64, m: f64, registers: &[u8]) -> f64 {
        let sum = registers
            .iter()
            .map(|&x| 2.0f64.powi(-(x as i32)))
            .sum::<f64>();
        alpha * m * m / sum
    }

    fn linear_counting_estimate(m: f64, number_of_zero_registers: f64) -> f64 {
        m * (m / number_of_zero_registers).ln()
    }
}

/// generic hash utility
impl<T: Hash, H: Hasher + Clone> HyperLogLog<T, H> {
    fn get_hash(&self, element: &T) -> u64 {
        let mut h = self.hasher.clone();
        Hash::hash(element, &mut h);
        h.finish()
    }
}

// TODO(i10416): add property based tests
#[cfg(test)]
mod tests {

    
}
