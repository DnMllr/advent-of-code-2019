use super::Memory as TMemory;
use std::collections::HashMap;

const BUFFER_SIZE: usize = 1024;

pub struct Memory {
    buf: Box<[i64; BUFFER_SIZE]>,
    large_address_storage: HashMap<usize, i64>,
}

impl Memory {
    pub fn new() -> Self {
        Self {
            buf: Box::new([0; BUFFER_SIZE]),
            large_address_storage: HashMap::new(),
        }
    }

    pub fn zero(&mut self) -> &mut Self {
        for address in self.iter_mut() {
            *address = 0;
        }
        self.large_address_storage.clear();
        self
    }

    pub fn as_inner(&self) -> &[i64; BUFFER_SIZE] {
        self.buf.as_ref()
    }

    pub fn as_inner_mut(&mut self) -> &mut [i64; BUFFER_SIZE] {
        self.buf.as_mut()
    }

    pub fn iter(&self) -> impl Iterator<Item = &i64> {
        self.buf.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut i64> {
        self.buf.iter_mut()
    }
}

impl TMemory for Memory {
    fn load(&self, idx: usize) -> Option<&i64> {
        if idx >= BUFFER_SIZE {
            self.large_address_storage.get(&idx).or(Some(&0))
        } else {
            self.buf.get(idx)
        }
    }

    fn load_mut(&mut self, idx: usize) -> Option<&mut i64> {
        if idx >= BUFFER_SIZE {
            Some(self.large_address_storage.entry(idx).or_insert(0))
        } else {
            self.buf.get_mut(idx)
        }
    }
}
