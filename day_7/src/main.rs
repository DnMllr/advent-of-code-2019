use std::io::BufReader;

use advent_common::intcode::{VMType, VecPort, VM};
use itertools::Itertools;

use anyhow::Result;
use std::fs::File;

struct FoldState<'a> {
    pub vm: &'a mut VM<VecPort>,
    pub output: i32,
}

impl<'a> From<&'a mut VM<VecPort>> for FoldState<'a> {
    fn from(vm: &'a mut VM<VecPort>) -> Self {
        Self { vm, output: 0 }
    }
}

fn make_vm() -> Result<VM<VecPort>> {
    if let Some(file_name) = std::env::args().nth(1) {
        let mut reader = BufReader::new(File::open(file_name)?);
        VM::with_port(VecPort::new()).read_source(&mut reader)
    } else {
        let stdin = std::io::stdin();
        let mut reader = BufReader::new(stdin.lock());
        VM::with_port(VecPort::new()).read_source(&mut reader)
    }
}

fn part_1(vm: &mut VM<VecPort>) -> i32 {
    (0..=4)
        .permutations(5)
        .map(move |combo| -> Result<i32> {
            Ok(combo
                .into_iter()
                .fold(Ok(vm.into()), |s: Result<FoldState>, phase| {
                    s.and_then(|mut state| {
                        state.vm.reset();
                        state.vm.port().input(phase);
                        state.vm.port().input(state.output);
                        state.vm.eval()?;
                        for &out in state.vm.port().output() {
                            state.output = out;
                        }
                        Ok(state)
                    })
                })?
                .output)
        })
        .map(Result::unwrap)
        .max()
        .expect("there should be a max")
}

fn main() -> Result<()> {
    println!("part 1 out > {}", part_1(&mut make_vm()?));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_1_1() -> Result<()> {
        let mut vm = VM::with_port(VecPort::new()).read_source(&mut BufReader::new(
            "3,15,3,16,1002,16,10,16,1,16,15,15,4,15,99,0,0".as_bytes(),
        ))?;
        assert_eq!(part_1(&mut vm), 43210, "first example");
        Ok(())
    }

    #[test]
    fn test_part_1_2() -> Result<()> {
        let mut vm = VM::with_port(VecPort::new()).read_source(&mut BufReader::new(
            "3,23,3,24,1002,24,10,24,1002,23,-1,23,101,5,23,23,1,24,23,23,4,23,99,0,0".as_bytes(),
        ))?;
        assert_eq!(part_1(&mut vm), 54321, "second example");
        Ok(())
    }

    #[test]
    fn test_part_1_3() -> Result<()> {
        let mut vm = VM::with_port(VecPort::new()).read_source(&mut BufReader::new("3,31,3,32,1002,32,10,32,1001,31,-2,31,1007,31,0,33,1002,33,7,33,1,33,31,31,1,32,31,31,4,31,99,0,0,0".as_bytes()))?;
        assert_eq!(part_1(&mut vm), 65210, "third example");
        Ok(())
    }
}
