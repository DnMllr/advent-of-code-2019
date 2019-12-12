use std::fmt::Write as FWrite;
use std::io;
use std::io::{BufRead, BufReader, Read, Stdin, Stdout, Write};

use anyhow::Result;

use crate::intcode::errors::ErrorKinds;
use crate::intcode::opcodes::OpCode;

pub struct VM<I: BufRead, O: Write> {
    input: I,
    output: O,
    initial_program: Vec<i32>,
    memory: Vec<i32>,
    instruction_pointer: usize,
}

impl VM<BufReader<Stdin>, Stdout> {
    pub fn default() -> Self {
        Default::default()
    }

    pub fn default_from_source<T: BufRead>(reader: &mut T) -> Result<Self> {
        Self::default().read_source(reader)
    }
}

impl<I: BufRead, O: Write> VM<I, O> {
    pub fn with_io(i: I, o: O) -> Self {
        Self {
            input: i,
            output: o,
            initial_program: Vec::with_capacity(64),
            memory: Vec::with_capacity(64),
            instruction_pointer: 0,
        }
    }

    pub fn read_source<T: BufRead>(mut self, reader: &mut T) -> Result<Self> {
        self.initial_program.clear();
        self.memory.clear();
        self.reset();
        let mut s = String::new();
        reader
            .read_to_string(&mut s)
            .map_err(ErrorKinds::ReadToString)?;
        for c in s.split(',') {
            self.add_raw_instruction(
                c.trim()
                    .parse()
                    .map_err(|_| ErrorKinds::StringParseError(c.to_owned()))?,
            );
        }
        Ok(self)
    }

    fn add_raw_instruction(&mut self, instruction: i32) -> &mut Self {
        self.initial_program.push(instruction);
        self.memory.push(instruction);
        self
    }

    pub fn reset(&mut self) -> &mut Self {
        for (left, right) in self.memory.iter_mut().zip(self.initial_program.iter()) {
            *left = *right;
        }
        self.instruction_pointer = 0;
        self
    }

    pub fn ip(&self) -> usize {
        self.instruction_pointer
    }

    pub fn load(&self, idx: usize) -> Option<&i32> {
        self.memory.get(idx)
    }

    pub fn load_mut(&mut self, idx: i32) -> Option<&mut i32> {
        self.memory.get_mut(idx as usize)
    }

    fn load_inst(&self) -> Result<OpCode> {
        OpCode::parse(&self.memory[self.ip()..])
    }

    pub fn advance(&mut self, amount: usize) -> &mut Self {
        self.jump_to(amount + self.instruction_pointer)
    }

    pub fn jump_to(&mut self, to: usize) -> &mut Self {
        self.instruction_pointer = to;
        self
    }

    pub fn eval(&mut self) -> Result<i32> {
        loop {
            if self.load_inst()?.exec(self)? {
                return Ok(*self
                    .load(0)
                    .expect("there should always be at least the first memory address"));
            }
        }
    }

    pub fn dump(&self) -> String {
        let msg = "it should be impossible to fail here, just writing a format into memory";
        let mut s = String::new();
        for (i, num) in self.memory.iter().enumerate() {
            if i > 0 {
                write!(&mut s, ",").expect(msg);
            }
            write!(&mut s, "{}", num).expect(msg);
        }
        s
    }
}

impl Default for VM<BufReader<Stdin>, Stdout> {
    fn default() -> Self {
        Self::with_io(BufReader::new(io::stdin()), io::stdout())
    }
}

impl<I: BufRead, O: Write> Write for VM<I, O> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, io::Error> {
        self.output.write(buf)
    }

    fn flush(&mut self) -> Result<(), io::Error> {
        self.output.flush()
    }
}

impl<I: BufRead, O: Write> Read for VM<I, O> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, io::Error> {
        self.input.read(buf)
    }
}

impl<I: BufRead, O: Write> BufRead for VM<I, O> {
    fn fill_buf(&mut self) -> Result<&[u8], io::Error> {
        self.input.fill_buf()
    }

    fn consume(&mut self, amt: usize) {
        self.input.consume(amt)
    }
}
