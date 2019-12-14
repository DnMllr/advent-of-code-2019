use advent_common::input::DayInput;

use anyhow::{Error, Result};

struct Day8 {
    image: String,
    width: usize,
    height: usize,
}

impl Day8 {
    pub fn new(width: usize, height: usize) -> Result<Self> {
        DayInput::new(8).with_input(|input| {
            let mut image = String::new();
            input.read_to_string(&mut image)?;
            Ok(Self::with(image, width, height))
        })
    }

    pub fn with(image: String, width: usize, height: usize) -> Self {
        Self {
            image,
            width,
            height,
        }
    }

    pub fn layers(&self) -> impl Iterator<Item = &[u8]> {
        self.image
            .trim()
            .as_bytes()
            .chunks(self.width * self.height)
    }

    pub fn part_1(&self) -> Result<usize> {
        let (_, ones, twos) = self
            .layers()
            .map(|layer| {
                layer.iter().fold((0, 0, 0), |m, c| match c {
                    b'0' => (m.0 + 1, m.1, m.2),
                    b'1' => (m.0, m.1 + 1, m.2),
                    b'2' => (m.0, m.1, m.2 + 1),
                    x => panic!("unexpected input {}", x),
                })
            })
            .min()
            .ok_or_else(|| Error::msg("didn't find answer"))?;
        Ok(ones * twos)
    }

    fn encode_image(&self, buffer: &[u8]) -> String {
        let mut result = String::with_capacity(self.width * self.height + self.height);
        let mut first = true;
        for h in 0..self.height {
            if !first {
                result.push('\n');
            }
            first = false;
            for w in 0..self.width {
                result.push(if buffer[h * self.width + w] == b'0' {
                    ' '
                } else {
                    'X'
                });
            }
        }
        result
    }

    pub fn part_2(&self) -> Result<String> {
        let mut buf = Vec::with_capacity(self.width * self.height);
        for _ in 0..self.width * self.height {
            buf.push(b'2');
        }
        for layer in self.layers() {
            for (l, r) in layer.iter().zip(buf.iter_mut()) {
                if *r == b'2' {
                    *r = *l;
                }
            }
        }
        Ok(self.encode_image(&buf))
    }
}

fn main() -> Result<()> {
    let width = 25;
    let height = 6;
    let day8 = Day8::new(width, height)?;
    println!("part1 answer >>> {}", day8.part_1()?);
    println!("part2 answer >>>\n\n{}", day8.part_2()?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn part_2() -> Result<()> {
        assert_eq!(
            Day8::with("0222112222120000".to_owned(), 2, 2).part_2()?,
            " X\nX ",
            "part two should be correct",
        );
        Ok(())
    }
}
