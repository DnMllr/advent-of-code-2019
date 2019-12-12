use std::io;
use std::io::{BufRead, BufReader, Write};

use anyhow::Result;
use thiserror::Error;

use advent_common::intcode::VM;

#[derive(Error, Debug)]
pub enum ErrorKinds {
    #[error("no result found")]
    NoResult,

    #[error("out of bounds reference")]
    OutOfBounds,
}

fn part_two<I: BufRead, O: Write>(vm: &mut VM<I, O>) -> Result<i32> {
    let target = 19_690_720;
    for noun in 0..=99 {
        for verb in 0..=99 {
            vm.reset();
            vm.load_mut(1)
                .ok_or(ErrorKinds::OutOfBounds)
                .map(|r| *r = noun)?;
            vm.load_mut(2)
                .ok_or(ErrorKinds::OutOfBounds)
                .map(|r| *r = verb)?;
            vm.eval()?;
            if vm.load(0) == Some(&target) {
                return Ok(100 * noun + verb);
            }
        }
    }
    Err(ErrorKinds::NoResult.into())
}

fn main() -> Result<()> {
    match part_two(&mut VM::default_from_source(&mut BufReader::new(
        io::stdin().lock(),
    ))?) {
        Ok(answer) => {
            println!("part two found answer {}", answer);
            Ok(())
        }
        err => err.map(|_| ()),
    }
}
