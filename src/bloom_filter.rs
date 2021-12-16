use crate::utils::hash_utils;

use super::utils::collection_like::*;
use std::{
    collections::hash_map::{DefaultHasher, RandomState},
    f64::consts::LN_2,
    hash::{BuildHasher, Hash, Hasher},
    marker::PhantomData,
};

use bit_vec::BitVec;
// bloom filter is a space efficient probablistic data structure.
// It consists of bit array of range m and k distinct hash functions, each of which returns [0,m).
// It inserts at the `x`-th positions of the array, where x is the value returned from hash function.
// When querying, it applies hash functions to the queried element and
// checks wheather all bit at the positions returned by hash functions set 1.
// If all bits are 1, it implies the given element is **probably** in the set.
// If any bit is 0, it **definitely** means the given element is not in the set.
// The false-negative rate is determined by parameters m and k.

struct BloomFilter<T: ?Sized, H: Hasher> {
    m: u64,
    k: u32,
    bits: BitVec,
    _marker: PhantomData<T>,
    hashers: [H; 2],
}

/// generic hashing utility
impl<T, H: Hasher + Clone> BloomFilter<T, H> {
    
    fn optimal_m(n: usize, false_positive_rate: f64) -> u64 {
        ((-1.0f64) * (n as f64) * false_positive_rate.ln() / (LN_2 * LN_2)).ceil() as u64
    }

    fn optimal_k(false_positive_rate: f64) -> u32 {
        (-1.0f64 * false_positive_rate.ln() / LN_2).ceil() as u32
    }

    fn hash_kernel(&self, element: &impl Hash) -> (u64, u64) {
        let mut h1 = self.hashers[0].clone();
        let mut h2 = self.hashers[1].clone();
        hash_utils::hash_kernel(&mut h1, &mut h2, element)
    }
}
/// implementation on DefaultHasher
impl<T> BloomFilter<T, DefaultHasher> {
    /// naively create bloom filter with arbitrary parameters
    pub fn new(m: u64, k: u32) -> Self {
        Self {
            m: m,
            k: k,
            bits: BitVec::from_elem(m as usize, false),
            _marker: PhantomData,
            hashers: [
                RandomState::new().build_hasher(),
                RandomState::new().build_hasher(),
            ],
        }
    }
    /// create bloom filter with optimal k and m from possible N and acceptable false positive rate.
    pub fn optimal(n: usize, false_positive_rate: f64) -> Self {
        Self::new(
            Self::optimal_m(n, false_positive_rate),
            Self::optimal_k(false_positive_rate),
        )
    }
}

impl<T: Hash, H: Hasher + Clone> SetLike<T> for BloomFilter<T, H> {
    fn contains(&self, element: T) -> bool {
        let (h1, h2) = self.hash_kernel(&element);
        (0..self.k)
            .map(|k| hash_utils::get_index(self.m, h1, h2, k))
            .all(|i| self.bits.get(i).unwrap())
    }

    fn insert(&mut self, element: T) {
        let (h1, h2) = self.hash_kernel(&element);
        (0..self.k).for_each(|k| {
            let at = hash_utils::get_index(self.m, h1, h2, k);
            self.bits.set(at, true);
        })
    }
}
// TODO(i10416): add property based tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert() {
        let mut bloom = BloomFilter::optimal(100, 0.01);
        bloom.insert("item");
        assert!(bloom.contains("item"));
    }

    #[test]
    fn check_and_insert() {
        let mut bloom = BloomFilter::optimal(100, 0.01);
        assert!(!bloom.contains("item_1"));
        assert!(!bloom.contains("item_2"));
        bloom.insert("item_1");
        assert!(bloom.contains("item_1"));
    }
}
