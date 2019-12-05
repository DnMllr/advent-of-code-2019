use itertools::Itertools;
use rayon::prelude::*;

const START_N: u32 = 357253;
const END_N: u32 = 892942;

#[derive(Debug)]
struct NumSplitIter {
    current: u32,
}

impl Iterator for NumSplitIter {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current > 0 {
            let dig = self.current % 10;
            self.current -= dig;
            self.current /= 10;
            Some(dig)
        } else {
            None
        }
    }
}

fn split_num(num: u32) -> NumSplitIter {
    NumSplitIter { current: num }
}

fn has_double_digit(num: u32) -> bool {
    split_num(num)
        .zip(split_num(num).skip(1))
        .any(|(l, r)| l == r)
}

fn has_multi_digit(num: u32) -> bool {
    split_num(num)
        .zip(split_num(num).skip(1))
        .filter(|(l, r)| l == r)
        .group_by(|&(x, _)| x)
        .into_iter()
        .map(|(_, v)| v.count() + 1)
        .any(|x| x == 2)
}

fn only_increasing(num: u32) -> bool {
    split_num(num)
        .zip(split_num(num).skip(1))
        .all(|(l, r)| l >= r)
}

fn part1() {
    let time = std::time::Instant::now();
    let part1 = (START_N..END_N)
        .into_par_iter()
        .filter(|&n| only_increasing(n) && has_double_digit(n))
        .count();
    println!(
        "part one: {}\n\tcalculated in {}\n",
        part1,
        time.elapsed().as_millis()
    );
}

fn part2() {
    let time = std::time::Instant::now();
    let part2 = (START_N..END_N)
        .into_par_iter()
        .filter(|&n| only_increasing(n) && has_multi_digit(n))
        .count();
    println!(
        "part two: {}\n\tcalculated in {}\n",
        part2,
        time.elapsed().as_millis()
    );
}

fn main() {
    part1();
    part2();
}
