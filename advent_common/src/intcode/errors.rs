use std::io;

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
    #[error("couldn't parse input into i64 {0}")]
    StringParseError(String),
    #[error("out of static input")]
    OutOfStaticInputError,
}

#[derive(Error, Debug)]
pub enum ErrorKinds {
    #[error("the vm exited with error")]
    RanAfterErrorExitError,
    #[error("failed to read to internal string")]
    ReadToString(#[from] io::Error),
    #[error("parse error: invalid uint string {0}")]
    StringParseError(String),
    #[error("parse error: out of bound reference {0}")]
    MemoryError(OutOfBoundsReference),
    #[error("parse error: unknown opcode {0}")]
    UnknownOpcodeError(i64),
    #[error("io error: {0}")]
    IOError(IOError),
    #[error("output parameter was in immediate mode")]
    ImmediateModeOutputError,
    #[error("input provided to a vm which wasn't expecting input")]
    UnexpectedInputError,
    #[error("no input provided to a vm which was expecting input")]
    ExpectedInputError,
    #[error("not enough memory to load program")]
    NotEnoughMemoryToLoadProgramError,
    #[error("reference less than zero")]
    ReferenceLessThanZeroError,
}
