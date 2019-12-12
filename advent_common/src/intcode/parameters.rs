use anyhow::Result;

use crate::intcode::errors::{ErrorKinds, OutOfBoundsReference};
use crate::intcode::VMType;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Parameter {
    Immediate(i32),
    Reference(i32),
}

impl Parameter {
    pub fn new(idx: usize, immediate: bool, instructions: &[i32]) -> Result<Self> {
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

    pub fn read<V: VMType>(self, vm: &V) -> Result<i32> {
        Ok(match self {
            Parameter::Immediate(x) => x,
            Parameter::Reference(r) => *vm.load(r as usize).ok_or(ErrorKinds::MemoryError(
                OutOfBoundsReference::ReferenceParameter,
            ))?,
        })
    }

    pub fn read_mut<V: VMType>(self, vm: &mut V) -> Result<&mut i32> {
        match self {
            Parameter::Reference(r) => Ok(vm.load_mut(r).ok_or(ErrorKinds::MemoryError(
                OutOfBoundsReference::ReferenceParameter,
            ))?),
            Parameter::Immediate(_) => Err(ErrorKinds::ImmediateModeOutputError.into()),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct BinaryParams {
    pub left: Parameter,
    pub right: Parameter,
    pub out: Parameter,
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
    pub value: Parameter,
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
    pub test: Parameter,
    pub location: Parameter,
}

impl ConditionParams {
    pub fn new(parameters: u8, instructions: &[i32]) -> Result<Self> {
        Ok(Self {
            test: Parameter::new(1, parameters & 4 > 0, instructions)?,
            location: Parameter::new(2, parameters & 2 > 0, instructions)?,
        })
    }
}
