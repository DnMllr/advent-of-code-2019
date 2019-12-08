use std::fmt::Write;
use std::io;
use std::io::BufRead;

use anyhow::Result;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum OutOfBoundsReference {
    #[error("opcode referenced out of bounds memory")]
    ReferenceParameter,
    #[error("opcode expected more parameters")]
    OpCodeLength,
}

#[derive(Error, Debug)]
pub enum ErrorKinds {
    #[error("failed to read to internal string")]
    ReadToString(#[from] io::Error),
    #[error("parse error: invalid uint string {0}")]
    StringParseError(String),
    #[error("parse error: out of bound reference {0}")]
    MemoryError(OutOfBoundsReference),
    #[error("parse error: unknown opcode {0}")]
    UnknownOpcode(usize),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Parameter {
    Immediate(usize),
    Reference(usize),
}

impl Parameter {
    fn new(idx: usize, immediate: bool, instructions: &[usize]) -> Result<Self> {
        Ok(if immediate {
            Parameter::Immediate(
                *instructions
                    .get(idx)
                    .ok_or(ErrorKinds::MemoryError(OutOfBoundsReference::OpCodeLength))?,
            )
        } else {
            Parameter::Reference(
                *instructions
                    .get(idx)
                    .ok_or(ErrorKinds::MemoryError(OutOfBoundsReference::OpCodeLength))?,
            )
        })
    }

    fn read(&self, vm: &VM) -> usize {
        match *self {
            Parameter::Immediate(x) => x,
            Parameter::Reference(r) => *vm.load(r),
        }
    }

    fn read_mut<'a>(&self, vm: &'a mut VM) -> &'a mut usize {
        match *self {
            Parameter::Reference(r) => vm.load_mut(r),
            _ => panic!("can't load immediate output parameter as mutable"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum OpCode {
    Add {
        left: Parameter,
        right: Parameter,
        out: Parameter,
    },
    Mul {
        left: Parameter,
        right: Parameter,
        out: Parameter,
    },
    Exit,
}

static PLACES: [usize; 3] = [10000, 1000, 100];

impl OpCode {
    pub fn len(&self) -> usize {
        match self {
            &OpCode::Add {
                left: _,
                right: _,
                out: _,
            }
            | &OpCode::Mul {
                left: _,
                right: _,
                out: _,
            } => 4,
            &OpCode::Exit => 1,
        }
    }

    pub fn parse(instructions: &[usize]) -> Result<Self> {
        let mut parameters = [false, false, false];
        if let Some(first) = instructions.first() {
            let mut value = *first;
            for (idx, &place) in PLACES.iter().enumerate() {
                let mut count = 0;
                while value >= place {
                    value -= place;
                    count += 1;
                }
                parameters[idx] = count == 1;
            }
            match value {
                1 => Ok(OpCode::Add {
                    left: Parameter::new(1, parameters[2], instructions)?,
                    right: Parameter::new(2, parameters[1], instructions)?,
                    out: Parameter::new(3, parameters[0], instructions)?,
                }),
                2 => Ok(OpCode::Mul {
                    left: Parameter::new(1, parameters[2], instructions)?,
                    right: Parameter::new(2, parameters[1], instructions)?,
                    out: Parameter::new(3, parameters[0], instructions)?,
                }),
                99 => Ok(OpCode::Exit),
                x => Err(ErrorKinds::UnknownOpcode(x).into()),
            }
        } else {
            Err(ErrorKinds::MemoryError(OutOfBoundsReference::OpCodeLength).into())
        }
    }

    pub fn exec(self, vm: &mut VM) -> Result<bool> {
        match self {
            OpCode::Add { left, right, out } => {
                *out.read_mut(vm) = left.read(vm) + right.read(vm);
                vm.advance(self.len());
            }
            OpCode::Mul { left, right, out } => {
                *out.read_mut(vm) = left.read(vm) * right.read(vm);
                vm.advance(self.len());
            }
            OpCode::Exit => return Ok(true),
        };
        Ok(false)
    }
}

pub struct VM {
    initial_program: Vec<usize>,
    memory: Vec<usize>,
    instruction_pointer: usize,
}

impl VM {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn from_source<T: BufRead>(reader: &mut T) -> Result<Self> {
        let mut s = String::new();
        reader
            .read_to_string(&mut s)
            .map_err(ErrorKinds::ReadToString)?;
        let mut prog = Self::new();
        for c in s.split(',') {
            prog.add_raw_instruction(
                c.trim()
                    .parse()
                    .map_err(|_| ErrorKinds::StringParseError(c.to_owned()))?,
            );
        }
        Ok(prog)
    }

    fn add_raw_instruction(&mut self, instruction: usize) -> &mut Self {
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

    pub fn load_inst(&self) -> Result<OpCode> {
        OpCode::parse(&self.memory[self.ip()..])
    }

    pub fn advance(&mut self, amount: usize) -> &mut Self {
        self.instruction_pointer += amount;
        self
    }

    pub fn eval(&mut self) -> Result<usize> {
        loop {
            if self.load_inst()?.exec(self)? {
                return Ok(*self.load(0));
            }
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

impl Default for VM {
    fn default() -> Self {
        Self {
            initial_program: Vec::with_capacity(64),
            memory: Vec::with_capacity(64),
            instruction_pointer: 0,
        }
    }
}
