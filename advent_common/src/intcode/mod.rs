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
    #[error("couldn't parse input into i32 {0}")]
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
pub struct BinaryParams {
    left: Parameter,
    right: Parameter,
    out: Parameter,
}

impl BinaryParams {
    pub fn new(parameters: u8, instructions: &[i32]) -> Result<Self> {
        Ok(Self {
            left: Parameter::new(1, parameters & 4 > 0, instructions)?,
            right: Parameter::new(2, parameters & 2 > 0, instructions)?,
            out: Parameter::new(3, parameters & 1 > 0, instructions)?,
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct UnaryParams {
    value: Parameter,
}

impl UnaryParams {
    pub fn new(parameters: u8, instructions: &[i32]) -> Result<Self> {
        Ok(Self {
            value: Parameter::new(1, parameters & 4 > 0, instructions)?,
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ConditionParams {
    test: Parameter,
    location: Parameter,
}

impl ConditionParams {
    pub fn new(parameters: u8, instructions: &[i32]) -> Result<Self> {
        Ok(Self {
            test: Parameter::new(1, parameters & 4 > 0, instructions)?,
            location: Parameter::new(2, parameters & 2 > 0, instructions)?,
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum OpCode {
    Add(BinaryParams),
    Mul(BinaryParams),
    LessThan(BinaryParams),
    Equals(BinaryParams),
    InputInteger(UnaryParams),
    OutputInteger(UnaryParams),
    JumpIfTrue(ConditionParams),
    JumpIfFalse(ConditionParams),
    Exit,
}

static PLACES: [i32; 3] = [10000, 1000, 100];

impl OpCode {
    pub fn len(&self) -> usize {
        match self {
            &OpCode::Add(_) | &OpCode::Mul(_) | &OpCode::LessThan(_) | &OpCode::Equals(_) => 4,
            &OpCode::JumpIfTrue(_) | &OpCode::JumpIfFalse(_) => 3,
            &OpCode::InputInteger(_) | &OpCode::OutputInteger(_) => 2,
            &OpCode::Exit => 1,
        }
    }

    pub fn parse(instructions: &[i32]) -> Result<Self> {
        let mut parameters: u8 = 0;
        if let Some(first) = instructions.first() {
            let mut value = *first;
            for (idx, &place) in PLACES.iter().enumerate() {
                let mut count = 0;
                while value >= place {
                    value -= place;
                    count += 1;
                }
                if count == 1 {
                    parameters |= (1 << idx) as u8;
                }
            }
            match value {
                1 => Ok(OpCode::Add(BinaryParams::new(parameters, instructions)?)),
                2 => Ok(OpCode::Mul(BinaryParams::new(parameters, instructions)?)),
                3 => Ok(OpCode::InputInteger(UnaryParams::new(
                    parameters,
                    instructions,
                )?)),
                4 => Ok(OpCode::OutputInteger(UnaryParams::new(
                    parameters,
                    instructions,
                )?)),
                5 => Ok(OpCode::JumpIfTrue(ConditionParams::new(
                    parameters,
                    instructions,
                )?)),
                6 => Ok(OpCode::JumpIfFalse(ConditionParams::new(
                    parameters,
                    instructions,
                )?)),
                7 => Ok(OpCode::LessThan(BinaryParams::new(
                    parameters,
                    instructions,
                )?)),
                8 => Ok(OpCode::Equals(BinaryParams::new(parameters, instructions)?)),
                99 => Ok(OpCode::Exit),
                x => Err(ErrorKinds::UnknownOpcodeError(x).into()),
            }
        } else {
            Err(ErrorKinds::MemoryError(OutOfBoundsReference::OpCodeLength).into())
        }
    }

    fn prompt<I: BufRead, O: Write>(vm: &mut VM<I, O>) -> Result<usize> {
        OpCode::write_str(vm, "please enter int> ")
    }

    fn write_str<I: BufRead, O: Write>(vm: &mut VM<I, O>, content: &str) -> Result<usize> {
        vm.write(content.as_bytes())
            .and_then(|amount| vm.flush().and_then(|_| Ok(amount)))
            .map_err(|e| ErrorKinds::IOError(IOError::OutputError(e)).into())
    }

    fn write_int<I: BufRead, O: Write>(vm: &mut VM<I, O>, i: i32) -> Result<usize> {
        OpCode::write_str(vm, &format!("output>>> {}\n", i))
    }

    fn read_line<I: BufRead, O: Write>(vm: &mut VM<I, O>) -> Result<String> {
        let mut s = String::new();
        vm.read_line(&mut s)
            .map_err(|err| ErrorKinds::IOError(IOError::InputError(err)))?;
        Ok(s)
    }

    fn read_int<I: BufRead, O: Write>(vm: &mut VM<I, O>) -> Result<i32> {
        let s = OpCode::read_line(vm)?;
        s.trim()
            .parse()
            .map_err(|_| ErrorKinds::IOError(IOError::StringParseError(s)).into())
    }

    pub fn exec<I: BufRead, O: Write>(self, vm: &mut VM<I, O>) -> Result<bool> {
        match self {
            OpCode::Add(BinaryParams { left, right, out }) => {
                *out.read_mut(vm)? = left.read(vm) + right.read(vm);
            }
            OpCode::Mul(BinaryParams { left, right, out }) => {
                *out.read_mut(vm)? = left.read(vm) * right.read(vm);
            }
            OpCode::InputInteger(UnaryParams { value }) => {
                OpCode::prompt(vm)?;
                *value.read_mut(vm)? = OpCode::read_int(vm)?;
            }
            OpCode::OutputInteger(UnaryParams { value }) => {
                OpCode::write_int(vm, value.read(vm))?;
            }
            OpCode::LessThan(BinaryParams { left, right, out }) => {
                *out.read_mut(vm)? = if left.read(vm) < right.read(vm) { 1 } else { 0 };
            }
            OpCode::Equals(BinaryParams { left, right, out }) => {
                *out.read_mut(vm)? = if left.read(vm) == right.read(vm) {
                    1
                } else {
                    0
                };
            }
            OpCode::JumpIfTrue(ConditionParams { test, location }) => {
                if test.read(vm) != 0 {
                    vm.jump_to(location.read(vm) as usize);
                    return Ok(false);
                }
            }
            OpCode::JumpIfFalse(ConditionParams { test, location }) => {
                if test.read(vm) == 0 {
                    vm.jump_to(location.read(vm) as usize);
                    return Ok(false);
                }
            }
            OpCode::Exit => return Ok(true),
        };
        vm.advance(self.len());
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

    pub fn load_mut(&mut self, idx: i32) -> &mut i32 {
        &mut self.memory[idx as usize]
    }

    pub fn load_inst(&self) -> Result<OpCode> {
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
                return Ok(*self.load(0));
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
