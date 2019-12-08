use std::fmt::Write as FWrite;
use std::io;
use std::io::{BufRead, BufReader, Read, Stdin, Stdout, Write};

use anyhow::Result;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum OutOfBoundsReference {
    #[error("opcode referenced out of bounds memory")]
    ReferenceParameter,
    #[error("opcode expected more parameters")]
    OpCodeLength,
}

#[derive(Debug, Error)]
pub enum IOError {
    #[error("couldn't read from input {0}")]
    InputError(io::Error),
    #[error("couldn't write to output {0}")]
    OutputError(io::Error),
    #[error("couldn't parse input into i32: {0}")]
    StringParseError(String),
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
    UnknownOpcodeError(i32),
    #[error("io error: {0}")]
    IOError(IOError),
    #[error("output parameter was in immediate mode")]
    ImmediateModeOutputError,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Parameter {
    Immediate(i32),
    Reference(i32),
}

impl Parameter {
    fn new(idx: usize, immediate: bool, instructions: &[i32]) -> Result<Self> {
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

    fn read<I: BufRead, O: Write>(&self, vm: &VM<I, O>) -> i32 {
        match *self {
            Parameter::Immediate(x) => x,
            Parameter::Reference(r) => *vm.load(r as usize),
        }
    }

    fn read_mut<'a, I: BufRead, O: Write>(&self, vm: &'a mut VM<I, O>) -> Result<&'a mut i32> {
        match *self {
            Parameter::Reference(r) => Ok(vm.load_mut(r)),
            Parameter::Immediate(_) => Err(ErrorKinds::ImmediateModeOutputError.into()),
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
    InputInteger {
        out: Parameter,
    },
    OutputInteger {
        value: Parameter,
    },
    Exit,
}

static PLACES: [i32; 3] = [10000, 1000, 100];

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
            &OpCode::InputInteger { out: _ } | &OpCode::OutputInteger { value: _ } => 2,
            &OpCode::Exit => 1,
        }
    }

    pub fn parse(instructions: &[i32]) -> Result<Self> {
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
                x => Err(ErrorKinds::UnknownOpcodeError(x).into()),
            }
        } else {
            Err(ErrorKinds::MemoryError(OutOfBoundsReference::OpCodeLength).into())
        }
    }

    pub fn exec<I: BufRead, O: Write>(self, vm: &mut VM<I, O>) -> Result<bool> {
        match self {
            OpCode::Add { left, right, out } => {
                *out.read_mut(vm)? = left.read(vm) + right.read(vm);
                vm.advance(self.len());
            }
            OpCode::Mul { left, right, out } => {
                *out.read_mut(vm)? = left.read(vm) * right.read(vm);
                vm.advance(self.len());
            }
            OpCode::InputInteger { out } => {
                let mut s = String::new();
                vm.read_line(&mut s)
                    .map_err(|err| ErrorKinds::IOError(IOError::InputError(err)))?;
                *out.read_mut(vm)? = s
                    .parse()
                    .map_err(|_| ErrorKinds::IOError(IOError::StringParseError(s)))?;
            }
            OpCode::OutputInteger { value } => write!(vm, "{}", value.read(vm))
                .map_err(|err| ErrorKinds::IOError(IOError::OutputError(err)))?,
            OpCode::Exit => return Ok(true),
        };
        Ok(false)
    }
}

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

    pub fn load(&self, idx: usize) -> &i32 {
        &self.memory[idx]
    }

    pub fn load_left_operand_addr(&self) -> &i32 {
        self.load(self.ip() + 1)
    }

    pub fn load_right_operand_addr(&self) -> &i32 {
        self.load(self.ip() + 2)
    }

    pub fn load_output_addr(&self) -> &i32 {
        self.load(self.ip() + 3)
    }

    pub fn load_left_operand(&self) -> &i32 {
        self.load(*self.load_left_operand_addr() as usize)
    }

    pub fn load_right_operand(&self) -> &i32 {
        self.load(*self.load_right_operand_addr() as usize)
    }

    pub fn load_output(&mut self) -> &mut i32 {
        self.load_mut(*self.load_output_addr())
    }

    pub fn load_mut(&mut self, idx: i32) -> &mut i32 {
        &mut self.memory[idx as usize]
    }

    pub fn load_inst(&self) -> Result<OpCode> {
        OpCode::parse(&self.memory[self.ip()..])
    }

    pub fn advance(&mut self, amount: usize) -> &mut Self {
        self.instruction_pointer += amount;
        self
    }

    pub fn eval(&mut self) -> Result<i32> {
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
