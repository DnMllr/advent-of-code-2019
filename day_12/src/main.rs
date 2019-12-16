use std::io::BufRead;
use std::num::ParseIntError;

use advent_common::input::DayInput;
use anyhow::{Error, Result};
use itertools::process_results;
use num::integer::Integer;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Coord {
    X,
    Y,
    Z,
}

#[derive(Clone, PartialEq, Debug, Hash, Eq)]
pub struct System(Vec<Moon>);

impl System {
    pub fn from_reader<T: BufRead>(reader: T) -> Result<System> {
        process_results(reader.lines(), |line_iter| {
            process_results(line_iter.map(Moon::parse_from_str), |moons| {
                System(moons.collect())
            })
        })?
    }

    pub fn step(&mut self, coord: Coord) {
        for i in 0..self.0.len() {
            let (left, right) = self.0.split_at_mut(i);
            if let Some(l) = left.last_mut() {
                for r in right.iter_mut() {
                    l.apply_gravity(coord, r);
                }
            }
        }

        for moon in self.0.iter_mut() {
            moon.apply_velocity(coord);
        }
    }

    pub fn step_all(&mut self) {
        self.step(Coord::X);
        self.step(Coord::Y);
        self.step(Coord::Z);
    }

    pub fn at_rest(&mut self) -> bool {
        self.0.iter().map(|m| m.kinetic_energy()).all(|e| e == 0)
    }

    pub fn total_energy(&self) -> i32 {
        self.0.iter().map(Moon::total_energy).sum()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Ord, PartialOrd, Eq, Hash)]
struct Vec3 {
    x: i32,
    y: i32,
    z: i32,
}

impl Vec3 {
    pub fn new() -> Self {
        Self { x: 0, y: 0, z: 0 }
    }

    pub fn absolute_sum(&self) -> i32 {
        self.x.abs() + self.y.abs() + self.z.abs()
    }
}

fn apply_gravity_from_position(coord: i32, other: i32) -> i32 {
    if coord > other {
        1
    } else if coord < other {
        -1
    } else {
        0
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct Moon {
    position: Vec3,
    velocity: Vec3,
}

fn make_malformed_input() -> Error {
    Error::msg("malformed input")
}

fn read_decl<'a, T: Iterator<Item = &'a str>>(mut decl_iter: T) -> Result<i32> {
    decl_iter
        .next()
        .ok_or_else(make_malformed_input)
        .and_then(|s| s.parse().map_err(|e: ParseIntError| e.into()))
}

impl Moon {
    fn new() -> Self {
        Moon {
            position: Vec3::new(),
            velocity: Vec3::new(),
        }
    }

    fn parse_from_str<S: AsRef<str>>(source: S) -> Result<Self> {
        let mut m = Self::new();
        for decl in source
            .as_ref()
            .trim_start_matches('<')
            .trim_end_matches('>')
            .split(", ")
        {
            let mut decl_iter = decl.split('=');
            match decl_iter.next().ok_or_else(make_malformed_input)? {
                "x" => m.position.x = read_decl(decl_iter)?,
                "y" => m.position.y = read_decl(decl_iter)?,
                "z" => m.position.z = read_decl(decl_iter)?,
                x => panic!("unexpected coordinate {}", x),
            }
        }
        Ok(m)
    }

    fn exert_gravity_on(&mut self, coord: Coord, other: &mut Self, back: bool) {
        match coord {
            Coord::X => {
                self.velocity.x += apply_gravity_from_position(other.position.x, self.position.x);
            }
            Coord::Y => {
                self.velocity.y += apply_gravity_from_position(other.position.y, self.position.y);
            }
            Coord::Z => {
                self.velocity.z += apply_gravity_from_position(other.position.z, self.position.z);
            }
        }
        if !back {
            other.exert_gravity_on(coord, self, true);
        }
    }

    pub fn apply_gravity(&mut self, coord: Coord, other: &mut Self) {
        self.exert_gravity_on(coord, other, false);
    }

    pub fn apply_velocity(&mut self, coord: Coord) {
        match coord {
            Coord::X => {
                self.position.x += self.velocity.x;
            },
            Coord::Y => {
                self.position.y += self.velocity.y;
            },
            Coord::Z => {
                self.position.z += self.velocity.z;
            },
        }
    }

    pub fn potential_energy(&self) -> i32 {
        self.position.absolute_sum()
    }

    pub fn kinetic_energy(&self) -> i32 {
        self.velocity.absolute_sum()
    }

    pub fn total_energy(&self) -> i32 {
        self.potential_energy() * self.kinetic_energy()
    }
}

fn read_moons() -> Result<System> {
    DayInput::new(12).with_input(|f| System::from_reader(f))
}

fn part1(moons: &mut System) -> Result<i32> {
    for _ in 0..1000 {
        moons.step_all();
    }
    Ok(moons.total_energy())
}

fn cycle_for_coord(moons: &mut System, coord: Coord) -> usize {
    moons.step(coord);
    let mut count = 1;
    while !moons.at_rest() {
        moons.step(coord);
        count += 1;
    }
    count * 2
}

fn part2(moons: &mut System) -> usize {
    let x = cycle_for_coord(moons, Coord::X);
    let y = cycle_for_coord(moons, Coord::Y);
    let z = cycle_for_coord(moons, Coord::Z);
    x.lcm(&y).lcm(&z)
}

fn main() -> Result<()> {
    let mut system = read_moons()?;
    println!("part 1 answer >> {}", part1(&mut system.clone())?);
    println!("part 2 answer >> {}", part2(&mut system));
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_system() -> Result<()> {
        let mut system = System::from_reader(
            "<x=-1, y=0, z=2>\n<x=2, y=-10, z=-7>\n<x=4, y=-8, z=8>\n<x=3, y=5, z=-1>".as_bytes(),
        )?;
        system.step_all();
        assert_eq!(
            system,
            System(vec![
                Moon {
                    position: Vec3 { x: 2, y: -1, z: 1 },
                    velocity: Vec3 { x: 3, y: -1, z: -1 }
                },
                Moon {
                    position: Vec3 { x: 3, y: -7, z: -4 },
                    velocity: Vec3 { x: 1, y: 3, z: 3 }
                },
                Moon {
                    position: Vec3 { x: 1, y: -7, z: 5 },
                    velocity: Vec3 { x: -3, y: 1, z: -3 }
                },
                Moon {
                    position: Vec3 { x: 2, y: 2, z: 0 },
                    velocity: Vec3 { x: -1, y: -3, z: 1 }
                }
            ]),
            "system should equal this after a single step"
        );
        system.step_all();
        assert_eq!(
            system,
            System(vec![
                Moon {
                    position: Vec3 { x: 5, y: -3, z: -1 },
                    velocity: Vec3 { x: 3, y: -2, z: -2 }
                },
                Moon {
                    position: Vec3 { x: 1, y: -2, z: 2 },
                    velocity: Vec3 { x: -2, y: 5, z: 6 }
                },
                Moon {
                    position: Vec3 { x: 1, y: -4, z: -1 },
                    velocity: Vec3 { x: 0, y: 3, z: -6 }
                },
                Moon {
                    position: Vec3 { x: 1, y: -4, z: 2 },
                    velocity: Vec3 { x: -1, y: -6, z: 2 }
                }
            ]),
            "system should equal this after a second step"
        );
        Ok(())
    }
}
