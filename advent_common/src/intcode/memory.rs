use super::Memory as TMemory;

pub struct Memory {
    buf: Box<[i64; 256]>,
}

impl Memory {
    pub fn new() -> Self {
        Self {
            buf: Box::new([0; 256]),
        }
    }

    pub fn zero(&mut self) -> &mut Self {
        for address in self.iter_mut() {
            *address = 0;
        }
        self
    }

    pub fn as_inner(&self) -> &[i64; 256] {
        self.buf.as_ref()
    }

    pub fn as_inner_mut(&mut self) -> &mut [i64; 256] {
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
        self.buf.get(idx)
    }

    fn load_mut(&mut self, idx: usize) -> Option<&mut i64> {
        self.buf.get_mut(idx)
    }
}
