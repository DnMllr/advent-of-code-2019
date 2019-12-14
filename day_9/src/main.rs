use anyhow::{Error, Result};

use advent_common::input::DayInput;
use advent_common::intcode::{Executor, Program, Runner, VMType, VecPort, VM};

fn read_program() -> Result<Program> {
    DayInput::new(9).with_input(|mut r| Program::from_reader(&mut r))
}

fn part_1(program: &Program) -> Result<i64> {
    let mut vm = VM::new();
    vm.load_program(program)?;
    let mut port = VecPort::new();
    port.input(1);
    if let Some(res) = Executor::run(vm, port).next() {
        return res;
    } else {
        Err(Error::msg("no result found"))
    }
}

fn part_2(program: &Program) -> Result<i64> {
    let mut vm = VM::new();
    vm.load_program(program)?;
    let mut port = VecPort::new();
    port.input(2);
    if let Some(res) = Executor::run(vm, port).next() {
        return res;
    } else {
        Err(Error::msg("no result found"))
    }
}

fn main() -> Result<()> {
    let program = read_program()?;
    println!("part 1 answer >> {}", part_1(&program)?);
    println!("part 2 answer >> {}", part_2(&program)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use advent_common::intcode::{Executor, Runner, VecPort};

    #[test]
    fn copy_of_self() -> Result<()> {
        let src = "109,1,204,-1,1001,100,1,100,1008,100,16,101,1006,101,0,99";
        let p = Program::from_source(src)?;
        let mut vm = VM::new();
        vm.load_program(&p)?;
        let mut out = String::new();
        for output in Executor::run(vm, VecPort::new()) {
            if out.len() == 0 {
                out.push_str(&format!("{}", output?));
            } else {
                out.push_str(&format!(",{}", output?));
            }
        }
        assert_eq!(
            out, src,
            "when when, the vm should output a copy of the input program"
        );
        Ok(())
    }

    #[test]
    fn should_output_16_digit_number() -> Result<()> {
        let p = Program::from_source("1102,34915192,34915192,7,4,7,99,0")?;
        let mut vm = VM::new();
        vm.load_program(&p)?;
        for output in Executor::run(vm, VecPort::new()) {
            let out = output?;
            assert!(
                out >= 1_000_000_000_000_000 && out < 10_000_000_000_000_000,
                "output should be a 16 digit number"
            )
        }
        Ok(())
    }

    #[test]
    fn should_output_large_number() -> Result<()> {
        let p = Program::from_source("104,1125899906842624,99")?;
        let mut vm = VM::new();
        vm.load_program(&p)?;
        let mut first = true;
        for output in Executor::run(vm, VecPort::new()) {
            assert!(first, "there should only be one output");
            first = false;
            assert_eq!(
                output?, 1_125_899_906_842_624,
                "the single output should be the large number in the middle"
            );
        }
        Ok(())
    }
}
