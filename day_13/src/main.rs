use std::fmt::{Display, Formatter};
use std::io::Write;
use std::thread::sleep;
use std::time::Duration;

use anyhow::Result;
use termion::async_stdin;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

use advent_common::input::DayInput;
use advent_common::intcode::{Memory, Program, Runable, Status, VMType, VM};

#[derive(Debug, PartialEq, Eq)]
enum Tile {
    Empty,
    Wall,
    Block,
    HorizontalPaddle,
    Ball,
}

impl From<i64> for Tile {
    fn from(n: i64) -> Self {
        match n {
            0 => Tile::Empty,
            1 => Tile::Wall,
            2 => Tile::Block,
            3 => Tile::HorizontalPaddle,
            4 => Tile::Ball,
            x => panic!("unexpected tile code {}", x),
        }
    }
}

enum InternalStatus {
    ReadingX,
    ReadingY(i64),
    ReadingT(i64, i64),
}

impl InternalStatus {
    pub fn next(&mut self, output: i64) -> Option<(i64, i64, i64)> {
        match *self {
            InternalStatus::ReadingX => {
                *self = InternalStatus::ReadingY(output);
                None
            }
            InternalStatus::ReadingY(x) => {
                *self = InternalStatus::ReadingT(x, output);
                None
            }
            InternalStatus::ReadingT(x, y) => {
                *self = InternalStatus::ReadingX;
                Some((x, y, output))
            }
        }
    }
}

pub struct Arcade {
    vm: VM,
    score: i64,
    internal_status: InternalStatus,
    screen_state: Vec<Vec<Tile>>,
}

impl Display for Arcade {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "{}score: {}{}",
            termion::cursor::Goto(1, 1),
            self.score,
            termion::cursor::Goto(1, 3)
        )?;
        for (i, v) in self.screen_state.iter().enumerate() {
            for t in v.iter() {
                match t {
                    Tile::Empty => write!(f, " "),
                    Tile::Wall => write!(f, "+"),
                    Tile::Block => write!(f, "#"),
                    Tile::HorizontalPaddle => write!(f, "="),
                    Tile::Ball => write!(f, "O"),
                }?
            }
            write!(f, "{}", termion::cursor::Goto(1, 3 + (i as u16)))?;
        }
        Ok(())
    }
}

impl Arcade {
    pub fn new() -> Self {
        Self {
            vm: VM::new(),
            score: 0,
            internal_status: InternalStatus::ReadingX,
            screen_state: Vec::new(),
        }
    }

    pub fn free_play(&mut self) {
        *self.vm.load_mut(0).unwrap() = 2;
    }

    pub fn load_program(&mut self, program: &Program) -> Result<()> {
        self.vm.load_program(program)
    }

    pub fn clear(&mut self) {
        for v in self.screen_state.iter_mut() {
            for t in v.iter_mut() {
                *t = Tile::Empty;
            }
        }
    }

    fn set(&mut self, x: i64, y: i64, t: Tile) {
        while self.screen_state.len() < y as usize + 1 {
            self.screen_state.push(Vec::new());
        }
        let row: &mut Vec<Tile> = &mut self.screen_state[y as usize];
        while row.len() < x as usize + 1 {
            row.push(Tile::Empty);
        }
        row[x as usize] = t;
    }

    pub fn handle_status(&mut self, status: Status) -> Status {
        match status {
            Status::HasOutput(output) => {
                if let Some((x, y, t)) = self.internal_status.next(output) {
                    if x == -1 && y == 0 {
                        self.score = t;
                    } else {
                        self.set(x, y, t.into());
                    }
                }
                self.run()
            }
            x => x,
        }
    }

    pub fn run(&mut self) -> Status {
        let status = self.vm.run();
        self.handle_status(status)
    }

    pub fn run_with_input(&mut self, input: i64) -> Status {
        let status = self.vm.run_with_input(input);
        self.handle_status(status)
    }
}

fn read_program() -> Result<Program> {
    DayInput::new(13).with_input(|mut r| Program::from_reader(&mut r))
}

fn part1(program: &Program) -> Result<usize> {
    let mut arcade = Arcade::new();
    arcade.load_program(program)?;
    arcade.run();
    Ok(arcade
        .screen_state
        .iter()
        .map(|v| v.iter().filter(|&t| t == &Tile::Block).count())
        .sum())
}

fn part2(program: &Program) -> Result<()> {
    let mut arcade = Arcade::new();
    println!("program:\n{}", program);
    arcade.load_program(program)?;
    arcade.free_play();
    let stdin = &mut async_stdin();
    let stdout = std::io::stdout();
    let mut stdout = stdout.into_raw_mode()?;
    let mut status: Option<Status> = None;
    write!(
        stdout,
        "{}{}{}",
        termion::cursor::Goto(1, 1),
        termion::clear::All,
        termion::cursor::Hide
    )?;
    loop {
        status = Some(if let Some(s) = status.take() {
            match s {
                Status::Exited(r) => {
                    write!(stdout, "{}", termion::cursor::Show)?;
                    stdout.flush()?;
                    return r;
                }
                Status::RequiresInput => {
                    let mut val = 0;
                    for c in stdin.keys() {
                        match c? {
                            Key::Left => val = -1,
                            Key::Right => val = 1,
                            _ => {}
                        }
                    }
                    arcade.run_with_input(val)
                }
                unknown => panic!("unexpected status {:?}", unknown),
            }
        } else {
            arcade.run()
        });
        write!(stdout, "{}", &arcade)?;
        stdout.flush()?;
        sleep(Duration::from_millis(200));
    }
}

fn main() -> Result<()> {
    let program = read_program()?;
    println!("part 1 answer >> {}", part1(&program)?);
    part2(&program)?;
    Ok(())
}
