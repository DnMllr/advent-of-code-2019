use std::io::{BufRead, BufReader, Stdin, Stdout, Write};

use anyhow::{Error, Result};

use crate::intcode::errors::{ErrorKinds, IOError};
use crate::intcode::{ReadInt, WriteInt};

pub fn stdport() -> Port<BufReader<Stdin>, Stdout> {
    Port::new(BufReader::new(std::io::stdin()), std::io::stdout())
}

pub struct Port<I: BufRead, O: Write> {
    buffer: String,
    input: I,
    output: O,
}

impl<I: BufRead, O: Write> WriteInt for Port<I, O> {
    fn write_int(&mut self, i: i32) -> Result<()> {
        writeln!(self.output, "output >>> {}", i)
            .map_err(|e| ErrorKinds::IOError(IOError::OutputError(e)).into())
    }
}

impl<I: BufRead, O: Write> ReadInt for Port<I, O> {
    fn read_int(&mut self) -> Result<i32> {
        self.buffer.clear();
        self.output
            .write_all(b"please enter an int <<< ")
            .map_err(|e| ErrorKinds::IOError(IOError::OutputError(e)))?;
        self.output
            .flush()
            .map_err(|e| ErrorKinds::IOError(IOError::OutputError(e)))?;
        self.input
            .read_line(&mut self.buffer)
            .map_err(|e| ErrorKinds::IOError(IOError::InputError(e)))?;
        self.buffer
            .trim()
            .parse()
            .map_err(|_| ErrorKinds::IOError(IOError::StringParseError(self.buffer.clone())).into())
    }
}

impl<I: BufRead, O: Write> Port<I, O> {
    pub fn new(input: I, output: O) -> Self {
        Self {
            buffer: String::new(),
            input,
            output,
        }
    }
}

#[derive(Default)]
pub struct VecPort {
    input: Vec<i32>,
    output: Vec<i32>,
}

impl ReadInt for VecPort {
    fn read_int(&mut self) -> Result<i32> {
        if !self.input.is_empty() {
            Ok(self.input.remove(0))
        } else {
            Err(ErrorKinds::IOError(IOError::OutOfStaticInputError).into())
        }
    }
}

impl WriteInt for VecPort {
    fn write_int(&mut self, i: i32) -> Result<(), Error> {
        self.output.push(i);
        Ok(())
    }
}

impl VecPort {
    pub fn new() -> Self {
        Self {
            input: Vec::new(),
            output: Vec::new(),
        }
    }

    pub fn input(&mut self, i: i32) -> &mut Self {
        self.input.push(i);
        self
    }

    pub fn into_output(self) -> Vec<i32> {
        self.output
    }

    pub fn output(&self) -> impl Iterator<Item = &i32> {
        self.output.iter()
    }
}
