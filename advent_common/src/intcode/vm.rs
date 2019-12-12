use std::fmt::Write as FWrite;
use std::io;
use std::io::{BufRead, BufReader, Stdin, Stdout};

use anyhow::Result;

use crate::intcode::errors::ErrorKinds;
use crate::intcode::opcodes::OpCode;
use crate::intcode::{Port, PortType, VMType};

pub struct VM<P: PortType> {
    port: P,
    initial_program: Vec<i32>,
    memory: Vec<i32>,
    instruction_pointer: usize,
}

impl VM<Port<BufReader<Stdin>, Stdout>> {
    pub fn default() -> Self {
        Default::default()
    }

    pub fn default_from_source<T: BufRead>(reader: &mut T) -> Result<Self> {
        Self::default().read_source(reader)
    }
}

impl<P: PortType> VM<P> {
    pub fn with_port(port: P) -> Self {
        Self {
            port,
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

    fn load_inst(&self) -> Result<OpCode> {
        OpCode::parse(&self.memory[self.ip()..])
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

impl<P: PortType> VMType for VM<P> {
    type Port = P;

    fn reset(&mut self) -> &mut Self {
        for (left, right) in self.memory.iter_mut().zip(self.initial_program.iter()) {
            *left = *right;
        }
        self.instruction_pointer = 0;
        self
    }

    fn ip(&self) -> usize {
        self.instruction_pointer
    }

    fn load(&self, idx: usize) -> Option<&i32> {
        self.memory.get(idx)
    }

    fn load_mut(&mut self, idx: i32) -> Option<&mut i32> {
        self.memory.get_mut(idx as usize)
    }

    fn advance(&mut self, amount: usize) -> &mut Self {
        self.jump_to(amount + self.instruction_pointer)
    }

    fn jump_to(&mut self, to: usize) -> &mut Self {
        self.instruction_pointer = to;
        self
    }

    fn eval(&mut self) -> Result<i32> {
        loop {
            if self.load_inst()?.exec(self)? {
                return Ok(*self
                    .load(0)
                    .expect("there should always be at least the first memory address"));
            }
        }
    }

    fn port(&mut self) -> &mut Self::Port {
        &mut self.port
    }
}

impl Default for VM<Port<BufReader<Stdin>, Stdout>> {
    fn default() -> Self {
        Self::with_port(Port::new(BufReader::new(io::stdin()), io::stdout()))
    }
}
