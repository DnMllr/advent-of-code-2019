use std::io;
use std::io::BufReader;

use anyhow::Result;
use thiserror::Error;

use advent_common::intcode::VM;

#[derive(Error, Debug)]
pub enum ErrorKinds {
    #[error("no result found")]
    NoResult,
}

fn part_two(vm: &mut VM) -> Result<usize> {
    let target = 19_690_720;
    for noun in 0..=99 {
        for verb in 0..=99 {
            vm.reset();
            *vm.load_mut(1) = noun;
            *vm.load_mut(2) = verb;
            vm.eval()?;
            if *vm.load(0) == target {
                return Ok(100 * noun + verb);
            }
        }
    }
    Err(ErrorKinds::NoResult.into())
}

fn main() -> Result<()> {
    match part_two(&mut VM::from_source(&mut BufReader::new(
        io::stdin().lock(),
    ))?) {
        Ok(answer) => Ok(println!("part two found answer {}", answer)),
        err => err.map(|_| ()),
    }
}
