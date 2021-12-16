pub(crate) mod collection_like {
    use std::hash::Hash;

    pub trait Insertable<T: Hash> {
        fn insert(&mut self, element: T);
    }

    pub trait SetLike<T: Hash> {
        fn contains(&self, element: T) -> bool;
        fn insert(&mut self, element: T);
    }
}

pub(crate) mod hash_utils {
    use std::hash::Hash;
    use std::hash::Hasher;

    pub fn hash_kernel<T: Hasher + Clone>(h1:&mut T, h2:&mut T, element: &impl Hash) -> (u64, u64) {
        Hash::hash(element, h1);
        Hash::hash(element, h2);
        (h1.finish(), h2.finish())
    }

    pub fn get_index(m:u64, h1: u64, h2: u64, k: u32) -> usize {
      (h1.wrapping_add((k as u64).wrapping_add(h2)) % m) as usize
  }
}
