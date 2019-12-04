use std::fmt::Write;
use std::io;
use std::io::prelude::*;
use std::io::BufReader;

use anyhow::Result;
use thiserror::Error;

// it's ok in this short example to just panic on invalid input

#[derive(Error, Debug)]
pub enum ErrorKinds {
    #[error("failed to read to string from stdin")]
    StdinReadToString(#[from] io::Error),
    #[error("parse error: invalid uint string {0}")]
    ParseError(String),
    #[error("no result found")]
    NoResult,
}

struct Program {
    initial_program: Vec<usize>,
    memory: Vec<usize>,
    instruction_pointer: usize,
}

impl Program {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn from_source<T: BufRead>(reader: &mut T) -> Result<Self> {
        let mut s = String::new();
        reader
            .read_to_string(&mut s)
            .map_err(ErrorKinds::StdinReadToString)?;
        let mut prog = Self::new();
        for c in s.split(',') {
            prog.add_raw_instruction(
                c.trim()
                    .parse()
                    .map_err(|_| ErrorKinds::ParseError(c.to_owned()))?,
            );
        }
        Ok(prog)
    }

    fn add_raw_instruction(&mut self, instruction: usize) -> &mut Self {
        self.initial_program.push(instruction);
        self.memory.push(instruction);
        self
    }

    fn reset(&mut self) -> &mut Self {
        for (left, right) in self.memory.iter_mut().zip(self.initial_program.iter()) {
            *left = *right;
        }
        self.instruction_pointer = 0;
        self
    }

    pub fn ip(&self) -> usize {
        self.instruction_pointer
    }

    pub fn load(&self, idx: usize) -> &usize {
        &self.memory[idx]
    }

    pub fn load_left_operand_addr(&self) -> &usize {
        self.load(self.ip() + 1)
    }

    pub fn load_right_operand_addr(&self) -> &usize {
        self.load(self.ip() + 2)
    }

    pub fn load_output_addr(&self) -> &usize {
        self.load(self.ip() + 3)
    }

    pub fn load_left_operand(&self) -> &usize {
        self.load(*self.load_left_operand_addr())
    }

    pub fn load_right_operand(&self) -> &usize {
        self.load(*self.load_right_operand_addr())
    }

    pub fn load_output(&mut self) -> &mut usize {
        self.load_mut(*self.load_output_addr())
    }

    pub fn load_mut(&mut self, idx: usize) -> &mut usize {
        &mut self.memory[idx]
    }

    pub fn load_inst(&self) -> &usize {
        self.load(self.ip())
    }

    fn advance(&mut self, amount: usize) -> &mut Self {
        self.instruction_pointer += amount;
        self
    }

    pub fn eval(&mut self) {
        loop {
            match *self.load_inst() {
                1 => {
                    *self.load_output() = self.load_left_operand() + self.load_right_operand();
                    self.advance(4);
                }
                2 => {
                    *self.load_output() = self.load_left_operand() * self.load_right_operand();
                    self.advance(4);
                }
                99 => return,
                unknown => panic!(
                    "unknown opcode [{}] at instruction: {}\n\ndump: {}",
                    unknown,
                    self.ip(),
                    self.dump()
                ),
            };
        }
    }

    pub fn dump(&self) -> String {
        let mut s = String::new();
        for (i, num) in self.memory.iter().enumerate() {
            if i > 0 {
                write!(&mut s, ",").expect(
                    "it should be impossible to fail here, just writing a format into memory",
                );
            }
            write!(&mut s, "{}", num)
                .expect("it should be impossible to fail here, just writing a format into memory");
        }
        s
    }
}

impl Default for Program {
    fn default() -> Self {
        Self {
            initial_program: Vec::with_capacity(64),
            memory: Vec::with_capacity(64),
            instruction_pointer: 0,
        }
    }
}

fn main() -> Result<()> {
    let mut program = Program::from_source(&mut BufReader::new(io::stdin().lock()))?;
    let target = 19_690_720;
    // patch
    for noun in 0..=99 {
        for verb in 0..=99 {
            program.reset();
            *program.load_mut(1) = noun;
            *program.load_mut(2) = verb;
            program.eval();
            if *program.load(0) == target {
                println!("result found {}", 100 * noun + verb);
                return Ok(());
            }
        }
    }
    Err(ErrorKinds::NoResult.into())
}
