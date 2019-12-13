use std::io::BufRead;

use anyhow::Result;

use super::errors::ErrorKinds;
use crate::intcode::opcodes::OpCode;
use std::fmt::{Display, Error, Formatter};

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

impl Display for Program {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        let mut memory_location = 0;
        let mut instruction_count = 0;
        let mut raw_mode = false;
        writeln!(f, "inst:mem \t\tcode\tparams")?;
        writeln!(
            f,
            "------------------------------------------------------------------"
        )?;
        while memory_location < self.inner.len() {
            write!(f, "{:04}:{:04}\t\t", instruction_count, memory_location)?;
            if !raw_mode {
                match OpCode::parse(&self.inner[memory_location..]) {
                    Ok(code) => {
                        writeln!(f, "{}", code)?;
                        memory_location += code.len();
                    }
                    Err(e) => {
                        let val = self.inner[memory_location];
                        if val != 0 {
                            writeln!(f, "\n|||parse error {}|||", e)?;
                            return Ok(());
                        }
                        raw_mode = true;
                        writeln!(f, "{}", val)?;
                        memory_location += 1;
                    }
                }
            } else {
                writeln!(f, "{}", self.inner[memory_location])?;
                memory_location += 1;
            }
            instruction_count += 1;
        }
        Ok(())
    }
}
