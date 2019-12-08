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
