use crate::intcode::errors::{ErrorKinds, OutOfBoundsReference};
use crate::intcode::parameters::{BinaryParams, ConditionParams, UnaryParams};
use crate::intcode::VMType;

use anyhow::Result;

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
            OpCode::Exit => {
                vm.exit();
                return Ok(true);
            }
        };
        vm.advance(self.len());
        Ok(false)
    }
}
