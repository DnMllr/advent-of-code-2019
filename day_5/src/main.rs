use std::fs::File;
use std::io;
use std::io::BufReader;
use std::path::PathBuf;

use anyhow::Result;
use thiserror::Error;

use advent_common::intcode::{VMType, VM};

#[derive(Error, Debug)]
enum ErrorKinds {
    #[error("unable to open file, encountered error {0}")]
    UnableToOpen(io::Error),
    #[error("no file was provided for the argument to this script")]
    NoFileProvided,
}

fn main() -> Result<()> {
    if let Some(file_name) = std::env::args().nth(1) {
        let p = PathBuf::from(file_name);
        let f = File::open(p).map_err(ErrorKinds::UnableToOpen)?;
        let mut vm = VM::default_from_source(&mut BufReader::new(f))?;
        vm.eval()?;
        Ok(())
    } else {
        Err(ErrorKinds::NoFileProvided.into())
    }
}
