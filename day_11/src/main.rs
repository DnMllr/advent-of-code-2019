use std::collections::HashMap;

use anyhow::Result;

use advent_common::input::DayInput;
use advent_common::intcode::{Program, Runable, Status, VMType, VM};

#[derive(Clone, Copy, Debug, PartialEq, Ord, PartialOrd, Eq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub fn left(self) -> Self {
        match self {
            Direction::Up => Direction::Left,
            Direction::Down => Direction::Right,
            Direction::Left => Direction::Down,
            Direction::Right => Direction::Up,
        }
    }

    pub fn right(self) -> Self {
        match self {
            Direction::Up => Direction::Right,
            Direction::Down => Direction::Left,
            Direction::Left => Direction::Up,
            Direction::Right => Direction::Down,
        }
    }

    pub fn turn_left(&mut self) {
        *self = self.left()
    }

    pub fn turn_right(&mut self) {
        *self = self.right()
    }
}

enum Color {
    White,
    Black,
}

struct Robot {
    position: (i32, i32),
    direction: Direction,
    output_count: usize,
    starting_panel: Option<Color>,
    vm: VM,
    painted_panels: HashMap<(i32, i32), Color>,
}

impl Robot {
    pub fn new() -> Self {
        Robot {
            position: (0, 0),
            direction: Direction::Up,
            output_count: 0,
            starting_panel: None,
            vm: VM::new(),
            painted_panels: HashMap::new(),
        }
    }

    pub fn start_on_white() -> Self {
        let mut robot = Self::new();
        robot.starting_panel = Some(Color::White);
        robot
    }

    fn step(&mut self) {
        match self.direction {
            Direction::Up => self.position.1 += 1,
            Direction::Down => self.position.1 -= 1,
            Direction::Left => self.position.0 -= 1,
            Direction::Right => self.position.0 += 1,
        }
    }

    pub fn load_program(&mut self, program: &Program) -> Result<()> {
        self.vm.load_program(program)
    }

    fn handle_status(&mut self, status: Status) -> Result<()> {
        match status {
            Status::Exited(result) => result,
            Status::HasOutput(output) => self.run_with_output(output),
            Status::RequiresInput => self.run_with_input(),
        }
    }

    fn run_with_output(&mut self, output: i64) -> Result<()> {
        if self.output_count % 2 == 0 {
            *self
                .painted_panels
                .entry(self.position)
                .or_insert(Color::Black) = if output == 0 {
                Color::Black
            } else {
                Color::White
            };
        } else {
            if output == 0 {
                self.direction.turn_left();
            } else {
                self.direction.turn_right();
            }
            self.step();
        }
        self.output_count += 1;
        self.run()
    }

    fn run_with_input(&mut self) -> Result<()> {
        let vm = &mut self.vm;
        let status = match self
            .painted_panels
            .get(&self.position)
            .unwrap_or(&self.starting_panel.take().unwrap_or(Color::Black))
        {
            Color::White => vm.run_with_input(1),
            Color::Black => vm.run_with_input(0),
        };
        self.handle_status(status)
    }

    pub fn run(&mut self) -> Result<()> {
        let status = self.vm.run();
        self.handle_status(status)
    }

    pub fn output(&self) -> Option<String> {
        let max_x = self.painted_panels.keys().map(|(x, _)| x).max()? + 1;
        let min_x = self.painted_panels.keys().map(|(x, _)| x).min()? - 1;
        let max_y = self.painted_panels.keys().map(|(_, y)| y).max()? + 1;
        let min_y = self.painted_panels.keys().map(|(_, y)| y).min()? - 1;

        let mut result = String::new();

        for x in min_x..max_x {
            for y in min_y..max_y {
                result.push(
                    match self.painted_panels.get(&(x, y)).unwrap_or(&Color::Black) {
                        Color::White => '#',
                        Color::Black => ' ',
                    },
                );
            }
            result.push('\n');
        }

        Some(result)
    }
}

fn read_program() -> Result<Program> {
    DayInput::new(11).with_input(|mut i| Program::from_reader(&mut i))
}

fn main() -> Result<()> {
    let program = read_program()?;
    let mut part1 = Robot::new();
    part1.load_program(&program)?;
    part1.run()?;
    println!("part 1 output >> {}", part1.painted_panels.len());
    let mut part2 = Robot::start_on_white();
    part2.load_program(&program)?;
    part2.run()?;
    println!("part 2 output\n\n{}", part2.output().unwrap());
    Ok(())
}
