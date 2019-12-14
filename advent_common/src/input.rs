use std::fs::File;
use std::io::{BufRead, BufReader, Stdin, StdinLock};
use std::path::Path;

use anyhow::Result;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Errors {
    #[error("no argument was passed to this program")]
    NoFirstArgumentError,

    #[error("failed to open input file: {0}")]
    FailedToOpenFileError(std::io::Error),

    #[error("failed to find input")]
    FailedToFindInputError,
}

pub struct DayInput {
    day: usize,
}

impl DayInput {
    pub fn new(day: usize) -> Self {
        Self { day }
    }

    pub fn with_input<F, O>(&self, f: F) -> Result<O>
    where
        F: Fn(&mut dyn BufRead) -> Result<O>,
    {
        if let Some(arg) = std::env::args().nth(1) {
            let p = Path::new(&arg);
            Ok(if p.exists() {
                let file = File::open(arg)?;
                f(&mut BufReader::new(file))?
            } else {
                f(&mut BufReader::new(arg.as_bytes()))?
            })
        } else if atty::is(atty::Stream::Stdin) {
            let file = self.input_file()?;
            Ok(f(&mut BufReader::new(file))?)
        } else {
            let stdin = std::io::stdin();
            let result = f(&mut BufReader::new(stdin.lock()))?;
            Ok(result)
        }
    }

    pub fn input_file(&self) -> Result<File> {
        let current_dir = std::env::current_dir()?;
        let mut p = current_dir.join("input.txt");
        if p.exists() {
            return Ok(File::open(p)?);
        }
        p = current_dir.join(format!("day_{}/input.txt", self.day));
        if p.exists() {
            return Ok(File::open(p)?);
        }
        Err(Errors::FailedToFindInputError.into())
    }
}

pub fn with_stdin<O, F>(f: F) -> O
where
    F: FnOnce(BufReader<StdinLock>) -> O,
{
    f(BufReader::new(std::io::stdin().lock()))
}

pub fn read_from_stdin() -> BufReader<Stdin> {
    BufReader::new(std::io::stdin())
}

pub fn read_from_locked_stdin(stdin: &Stdin) -> BufReader<StdinLock> {
    BufReader::new(stdin.lock())
}

pub fn read_from_input_file(filepath: &Path) -> Result<BufReader<File>> {
    Ok(BufReader::new(
        File::open(filepath).map_err(Errors::FailedToOpenFileError)?,
    ))
}
