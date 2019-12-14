use crate::intcode::errors::{ErrorKinds, OutOfBoundsReference};
use crate::intcode::parameters::{BinaryParams, ConditionParams, UnaryParams};
use crate::intcode::VMType;

use anyhow::Result;
use std::fmt::{Display, Error, Formatter};

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
    SetRelativeBase(UnaryParams),
    Exit,
}

static PLACES: [i64; 3] = [10000, 1000, 100];

impl OpCode {
    pub fn len(&self) -> usize {
        match self {
            &OpCode::Add(_) | &OpCode::Mul(_) | &OpCode::LessThan(_) | &OpCode::Equals(_) => 4,
            &OpCode::JumpIfTrue(_) | &OpCode::JumpIfFalse(_) => 3,
            &OpCode::InputInteger(_) | &OpCode::OutputInteger(_) | &OpCode::SetRelativeBase(_) => 2,
            &OpCode::Exit => 1,
        }
    }

    pub fn parse(instructions: &[i64]) -> Result<Self> {
        let mut parameters: [u8; 3] = [0, 0, 0];
        if let Some(first) = instructions.first() {
            let mut value = *first;
            for (idx, &place) in PLACES.iter().enumerate() {
                let mut count = 0;
                while value >= place {
                    value -= place;
                    count += 1;
                }
                parameters[idx] = count;
            }
            match value {
                1 => Ok(OpCode::Add(BinaryParams::new(&parameters, instructions)?)),
                2 => Ok(OpCode::Mul(BinaryParams::new(&parameters, instructions)?)),
                3 => Ok(OpCode::InputInteger(UnaryParams::new(
                    &parameters,
                    instructions,
                )?)),
                4 => Ok(OpCode::OutputInteger(UnaryParams::new(
                    &parameters,
                    instructions,
                )?)),
                5 => Ok(OpCode::JumpIfTrue(ConditionParams::new(
                    &parameters,
                    instructions,
                )?)),
                6 => Ok(OpCode::JumpIfFalse(ConditionParams::new(
                    &parameters,
                    instructions,
                )?)),
                7 => Ok(OpCode::LessThan(BinaryParams::new(
                    &parameters,
                    instructions,
                )?)),
                8 => Ok(OpCode::Equals(BinaryParams::new(
                    &parameters,
                    instructions,
                )?)),
                9 => Ok(OpCode::SetRelativeBase(UnaryParams::new(
                    &parameters,
                    instructions,
                )?)),
                99 => Ok(OpCode::Exit),
                x => Err(ErrorKinds::UnknownOpcodeError(x).into()),
            }
        } else {
            Err(ErrorKinds::MemoryError(OutOfBoundsReference::OpCodeLength).into())
        }
    }

    pub fn exec<V: VMType>(self, vm: &mut V) -> Result<bool> {
        match self {
            OpCode::Add(BinaryParams { left, right, out }) => {
                *out.read_mut(vm)? = left.read(vm)? + right.read(vm)?;
            }
            OpCode::Mul(BinaryParams { left, right, out }) => {
                *out.read_mut(vm)? = left.read(vm)? * right.read(vm)?;
            }
            OpCode::InputInteger(UnaryParams { value }) => {
                vm.input_to(value);
            }
            OpCode::OutputInteger(UnaryParams { value }) => {
                vm.output(value.read(vm)?);
            }
            OpCode::LessThan(BinaryParams { left, right, out }) => {
                *out.read_mut(vm)? = if left.read(vm)? < right.read(vm)? {
                    1
                } else {
                    0
                };
            }
            OpCode::Equals(BinaryParams { left, right, out }) => {
                *out.read_mut(vm)? = if left.read(vm)? == right.read(vm)? {
                    1
                } else {
                    0
                };
            }
            OpCode::JumpIfTrue(ConditionParams { test, location }) => {
                if test.read(vm)? != 0 {
                    vm.jump_to(location.read(vm)? as usize);
                    return Ok(false);
                }
            }
            OpCode::JumpIfFalse(ConditionParams { test, location }) => {
                if test.read(vm)? == 0 {
                    vm.jump_to(location.read(vm)? as usize);
                    return Ok(false);
                }
            }
            OpCode::SetRelativeBase(UnaryParams { value }) => {
                vm.offset_relative_base(value.read(vm)?)
            }
            OpCode::Exit => {
                vm.exit();
                return Ok(true);
            }
        };
        vm.advance(self.len());
        Ok(false)
    }
}

impl Display for OpCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            OpCode::Add(p) => write!(f, "add\t\t{}.", p),
            OpCode::Mul(p) => write!(f, "mul\t\t{}.", p),
            OpCode::LessThan(p) => write!(f, "lt\t\t{}.", p),
            OpCode::Equals(p) => write!(f, "eq\t\t{}.", p),
            OpCode::InputInteger(p) => write!(f, "in\t\t{}.", p),
            OpCode::OutputInteger(p) => write!(f, "out\t\t{}.", p),
            OpCode::JumpIfTrue(p) => write!(f, "jt\t\t{}.", p),
            OpCode::JumpIfFalse(p) => write!(f, "jf\t\t{}.", p),
            OpCode::SetRelativeBase(p) => write!(f, "srb\t\t{}.", p),
            OpCode::Exit => write!(f, "exit."),
        }
    }
}
