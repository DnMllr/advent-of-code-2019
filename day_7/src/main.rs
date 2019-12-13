use std::io::BufReader;

use advent_common::intcode::{Executor, Program, Runable, Runner, Status, VMType, VecPort, VM};
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
                    if let Some(output) = Executor::run(vm, port).next() {
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

fn part_2(program: &Program) -> i32 {
    let mut chain: Vec<VM> = (0..5).map(|_| VM::new()).collect();
    (5..=9)
        .permutations(5)
        .map(move |combo| {
            for (vm, &phase) in chain.iter_mut().zip(combo.iter()) {
                vm.load_program(program);
                if let Status::RequiresInput = vm.run() {
                    vm.run_with_input(phase);
                }
            }
            let mut output = 0;
            loop {
                let mut exited = false;
                for vm in chain.iter_mut() {
                    match vm.run_with_input(output) {
                        Status::Exited(_) => exited = true,
                        Status::HasOutput(out) => {
                            output = out;
                            if let Status::Exited(_) = vm.run() {
                                exited = true;
                            }
                        }
                        Status::RequiresInput => panic!("vm requested input two times in a row"),
                    }
                }
                if exited {
                    return output;
                }
            }
        })
        .max()
        .expect("there should be an answer")
}

fn main() -> Result<()> {
    let program = make_program()?;
    println!("part 1 out > {}", part_1(&program));
    println!("part 2 out > {}", part_2(&program));
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

    #[test]
    fn test_part_2_1() -> Result<()> {
        let program = Program::from_source(
            "3,26,1001,26,-4,26,3,27,1002,27,2,27,1,27,26,27,4,27,1001,28,-1,28,1005,28,6,99,0,0,5",
        )?;
        assert_eq!(part_2(&program), 139629729, "part 2 first example");
        Ok(())
    }

    #[test]
    fn test_part_2_2() -> Result<()> {
        let program = Program::from_source("3,52,1001,52,-5,52,3,53,1,52,56,54,1007,54,5,55,1005,55,26,1001,54,-5,54,1105,1,12,1,53,54,53,1008,54,0,55,1001,55,1,55,2,53,55,53,4,53,1001,56,-1,56,1005,56,6,99,0,0,0,0,10")?;
        assert_eq!(part_2(&program), 18216, "part 2 second example");
        Ok(())
    }
}
