use std::collections::HashMap;
use std::f64::consts::PI;
use std::io::BufRead;
use std::ops::Sub;

use anyhow::{Error, Result};
use num::Integer;

use advent_common::input::DayInput;

#[derive(Copy, Clone, Debug, PartialEq, Ord, PartialOrd, Eq, Hash)]
struct Asteroid {
    x: i32,
    y: i32,
}

impl Sub for Asteroid {
    type Output = Asteroid;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Sub for &Asteroid {
    type Output = Asteroid;

    fn sub(self, rhs: Self) -> Self::Output {
        Asteroid {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Asteroid {
    pub fn slope(&self) -> (i32, i32) {
        let gcd = self.x.gcd(&self.y);
        if gcd == 0 {
            return (self.x, self.y);
        }
        (self.x / gcd, self.y / gcd)
    }

    pub fn slope_between(&self, other: &Self) -> (i32, i32) {
        (self - other).slope()
    }

    pub fn distance(&self, other: &Self) -> f64 {
        (((self.x - other.x) as f64).powi(2) + ((self.y - other.y) as f64).powi(2)).sqrt()
    }
}

fn read_asteroids() -> Result<Vec<Asteroid>> {
    DayInput::new(10).with_input(|r| {
        let mut asteroids = Vec::new();
        for (y, line) in r.lines().enumerate() {
            for (x, c) in line?.chars().enumerate() {
                if c == '#' {
                    asteroids.push(Asteroid {
                        x: x as i32,
                        y: y as i32,
                    });
                }
            }
        }
        Ok(asteroids)
    })
}

fn part1(asteroids: &[Asteroid]) -> Result<(usize, Asteroid)> {
    let mut answers: HashMap<Asteroid, usize> = HashMap::new();
    for roid_a in asteroids.iter() {
        let mut map = HashMap::new();
        for roid_b in asteroids.iter().filter(|b| *b != roid_a) {
            *map.entry(roid_a.slope_between(roid_b)).or_insert(0) += 1;
        }
        answers.insert(*roid_a, map.iter().count());
    }
    Ok(answers
        .into_iter()
        .map(|(a, b)| (b, a))
        .max()
        .ok_or_else(|| Error::msg("didn't find answer"))?)
}

fn part2(station: &Asteroid, asteroids: &[Asteroid]) -> Result<Asteroid> {
    let mut slopes = HashMap::new();
    for roid in asteroids.iter().filter(|&r| r != station) {
        slopes
            .entry(station.slope_between(roid))
            .or_insert_with(Vec::new)
            .push(roid);
    }
    let mut values: Vec<(f64, Vec<&Asteroid>)> = slopes
        .into_iter()
        .map(|(slope, mut vec)| {
            vec.sort_by(|&a, &b| {
                PartialOrd::partial_cmp(&station.distance(a), &station.distance(b)).unwrap()
            });
            let mut angle = (slope.0 as f64).atan2(slope.1 as f64);
            if angle < 0.0 {
                angle += 2.0 * PI;
            }
            if angle != 0.0 {
                angle = 2.0 * PI - angle;
            }
            (angle, vec)
        })
        .collect();

    values.sort_by(|(aa, _), (ab, _)| PartialOrd::partial_cmp(aa, ab).unwrap());
    let mut index = 0;
    let mut count = 0;
    loop {
        let (_, roids) = &mut values[index];
        if !roids.is_empty() {
            let roid = roids.remove(0);
            count += 1;
            if count == 200 {
                return Ok(*roid);
            }
        }
        index += 1;
        index %= values.len();
    }
}

fn main() -> Result<()> {
    let asteroids = read_asteroids()?;
    let (part1_answer, best_asteroid) = part1(&asteroids)?;
    println!("part 1 answer >> {}", part1_answer);
    let two_hundreth_roid = part2(&best_asteroid, &asteroids)?;
    println!(
        "part 2 answer >> {}",
        two_hundreth_roid.x * 100 + two_hundreth_roid.y
    );
    Ok(())
}
