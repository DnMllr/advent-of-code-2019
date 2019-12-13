use std::io::BufReader;

use advent_common::intcode::{Executor, Program, Runner, VMType, VecPort, VM};
use itertools::Itertools;

use anyhow::{Error, Result};
use std::fs::File;

fn make_program() -> Result<Program> {
    if let Some(file_name) = std::env::args().nth(1) {
        let mut reader = BufReader::new(File::open(file_name)?);
        Program::from_reader(&mut reader)
    } else {
        let stdin = std::io::stdin();
        let mut reader = BufReader::new(stdin.lock());
        Program::from_reader(&mut reader)
    }
}

fn part_1(program: &Program) -> i32 {
    (0..=4)
        .permutations(5)
        .map(move |combo| -> Result<i32> {
            Ok(combo.into_iter().fold(Ok(0), |s, phase| {
                s.and_then(|state| {
                    let mut vm = VM::new();
                    vm.load_program(program);
                    let mut port = VecPort::new();
                    port.input(phase);
                    port.input(state);
                    for output in Executor::run(vm, port) {
                        return output;
                    }
                    Err(Error::msg("no output"))
                })
            })?)
        })
        .map(Result::unwrap)
        .max()
        .expect("there should be a max")
}

fn main() -> Result<()> {
    let program = make_program()?;
    println!("part 1 out > {}", part_1(&program));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_1_1() -> Result<()> {
        let program = Program::from_source("3,15,3,16,1002,16,10,16,1,16,15,15,4,15,99,0,0")?;
        assert_eq!(part_1(&program), 43210, "first example");
        Ok(())
    }

    #[test]
    fn test_part_1_2() -> Result<()> {
        let program = Program::from_source(
            "3,23,3,24,1002,24,10,24,1002,23,-1,23,101,5,23,23,1,24,23,23,4,23,99,0,0",
        )?;
        assert_eq!(part_1(&program), 54321, "second example");
        Ok(())
    }

    #[test]
    fn test_part_1_3() -> Result<()> {
        let program = Program::from_source("3,31,3,32,1002,32,10,32,1001,31,-2,31,1007,31,0,33,1002,33,7,33,1,33,31,31,1,32,31,31,4,31,99,0,0,0")?;
        assert_eq!(part_1(&program), 65210, "third example");
        Ok(())
    }
}
