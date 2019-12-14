use std::io;
use std::io::BufReader;

use anyhow::Result;
use thiserror::Error;

use advent_common::intcode::{Program, VMType, VM};

#[derive(Error, Debug)]
pub enum ErrorKinds {
    #[error("no result found")]
    NoResult,

    #[error("out of bounds reference")]
    OutOfBounds,
}

fn part_two<V: VMType>(vm: &mut V, program: &Program) -> Result<i64> {
    let target = 19_690_720;
    for noun in 0..=99 {
        for verb in 0..=99 {
            vm.load_program(program)?;
            vm.load_mut(1)
                .ok_or(ErrorKinds::OutOfBounds)
                .map(|r| *r = noun)?;
            vm.load_mut(2)
                .ok_or(ErrorKinds::OutOfBounds)
                .map(|r| *r = verb)?;
            vm.run();
            if vm.load(0) == Some(&target) {
                return Ok(100 * noun + verb);
            }
        }
    }
    Err(ErrorKinds::NoResult.into())
}

fn main() -> Result<()> {
    let program = Program::from_reader(&mut BufReader::new(io::stdin().lock()))?;
    let mut vm = VM::new();
    match part_two(&mut vm, &program) {
        Ok(answer) => {
            println!("part two found answer {}", answer);
            Ok(())
        }
        err => err.map(|_| ()),
    }
}
