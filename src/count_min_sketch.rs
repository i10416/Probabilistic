use super::utils::collection_like::*;
use super::utils::hash_utils;
use std::collections::hash_map::RandomState;
use std::hash::BuildHasher;
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    marker::PhantomData,
};

pub struct CountMinSketch<T, H: Hasher> {
    _marker: PhantomData<T>,
    k: u32,
    m: u64,
    hashers: [H; 2],
    table: Vec<Vec<u32>>,
}

impl<T> CountMinSketch<T, DefaultHasher> {
    // k: number of hash functions
    // m: table size
    pub fn new(k: u32, m: u64) -> Self {
        Self {
            m: m,
            k: k,
            _marker: PhantomData,
            hashers: [
                RandomState::new().build_hasher(),
                RandomState::new().build_hasher(),
            ],
            table: vec![vec![0; m as usize]; k as usize],
        }
    }
}

impl<T: Hash, H: Hasher + Clone> Insertable<T> for CountMinSketch<T, H> {
    fn insert(&mut self, element: T) {
        let (h1, h2) = self.hash_kernel(element);

        (0..self.k)
            .into_iter()
            .map(|k| (k, hash_utils::get_index(self.m, h1, h2, k)))
            .for_each(|(k, i)| {
                self.table[k as usize][i] = self.table[k as usize][i] + 1;
            });
    }
}

impl<T: Hash, H: Hasher + Clone> CountMinSketch<T, H> {
    fn hash_kernel(&self, element: T) -> (u64, u64) {
        let mut h1 = self.hashers[0].clone();
        let mut h2 = self.hashers[1].clone();
        hash_utils::hash_kernel(&mut h1, &mut h2, &element)
    }

    pub fn get_count(&self, element: T) -> usize {
        let (h1, h2) = self.hash_kernel(element);

        (0..self.k)
            .into_iter()
            .map(|k| (k, hash_utils::get_index(self.m, h1, h2, k)))
            .fold(usize::MAX, |acc, (k, i)| match acc {
                min if min > self.table[k as usize][i] as usize => {
                    self.table[k as usize][i] as usize
                }
                min => min,
            })
    }
}
