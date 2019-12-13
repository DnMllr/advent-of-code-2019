use std::io::BufRead;

use anyhow::Result;

use super::errors::ErrorKinds;

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub struct Program {
    inner: Vec<i32>,
}

impl Program {
    pub fn from_reader<T: BufRead>(reader: &mut T) -> Result<Self> {
        let mut s = String::new();
        reader
            .read_to_string(&mut s)
            .map_err(ErrorKinds::ReadToString)?;
        Self::from_source(&s)
    }

    pub fn from_source<T: AsRef<str>>(source: T) -> Result<Self> {
        let mut vec = Vec::new();
        for inst in source.as_ref().split(',') {
            vec.push(
                inst.trim()
                    .parse()
                    .map_err(|_| ErrorKinds::StringParseError(inst.to_owned()))?,
            );
        }
        Ok(Self { inner: vec })
    }

    pub fn load(&self) -> Vec<i32> {
        self.inner.clone()
    }

    pub fn load_to(&self, output: &mut Vec<i32>) {
        while output.len() < self.inner.len() {
            output.push(0);
        }
        for (l, r) in self.inner.iter().zip(output.iter_mut()) {
            *r = *l
        }
    }
}
