use std::io::BufRead;

use anyhow::Result;

use super::errors::ErrorKinds;
use crate::intcode::opcodes::OpCode;
use std::fmt::{Display, Error, Formatter};

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub struct Program {
    inner: Vec<i64>,
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

    pub fn load(&self) -> Vec<i64> {
        self.inner.clone()
    }

    pub fn load_to(&self, output: &mut [i64]) -> Result<()> {
        if output.len() < self.inner.len() {
            return Err(ErrorKinds::NotEnoughMemoryToLoadProgramError.into());
        }
        for (l, r) in self.inner.iter().zip(output.iter_mut()) {
            *r = *l
        }
        Ok(())
    }
}

impl Display for Program {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        let mut memory_location = 0;
        let mut instruction_count = 0;
        writeln!(f, "inst:mem \t\tcode\tparams")?;
        writeln!(
            f,
            "------------------------------------------------------------------"
        )?;
        while memory_location < self.inner.len() {
            write!(f, "{:04}:{:04}\t\t", instruction_count, memory_location)?;
            match OpCode::parse(&self.inner[memory_location..]) {
                Ok(code) => {
                    writeln!(f, "{}", code)?;
                    memory_location += code.len();
                }
                Err(_) => {
                    let val = self.inner[memory_location];
                    writeln!(f, "{}", val)?;
                    memory_location += 1;
                }
            }
            instruction_count += 1;
        }
        Ok(())
    }
}
