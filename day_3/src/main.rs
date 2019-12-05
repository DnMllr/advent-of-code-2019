use std::collections::HashMap;
use std::error::Error;
use std::io::{BufRead, BufReader};

trait Part {
    fn left(&mut self, amount: i32);
    fn right(&mut self, amount: i32);
    fn up(&mut self, amount: i32);
    fn down(&mut self, amount: i32);
    fn line(&mut self);
    fn into_answer(self) -> Vec<(i32, i32, i32)>;
}

// part one

#[derive(Debug, Copy, Clone)]
enum Mode {
    Input,
    Intersections,
}

#[derive(Debug)]
struct PartOne {
    horizontal: HashMap<i32, Vec<(i32, i32)>>,
    vertical: HashMap<i32, Vec<(i32, i32)>>,
    intersections: Vec<(i32, i32, i32)>,
    position: (i32, i32),
    mode: Mode,
}

impl Part for PartOne {
    fn left(&mut self, spaces: i32) {
        self.set(-spaces, 0);
    }

    fn right(&mut self, spaces: i32) {
        self.set(spaces, 0);
    }

    fn up(&mut self, spaces: i32) {
        self.set(0, spaces);
    }

    fn down(&mut self, spaces: i32) {
        self.set(0, -spaces);
    }

    fn line(&mut self) {
        self.mode = Mode::Intersections;
        for v in self.horizontal.values_mut() {
            v.sort();
        }
        for v in self.vertical.values_mut() {
            v.sort();
        }
        self.position = (0, 0);
    }

    fn into_answer(self) -> Vec<(i32, i32, i32)> {
        self.intersections
    }
}

fn push_to_intersections(intersections: &mut Vec<(i32, i32, i32)>, (x, y, cost): (i32, i32, i32)) {
    if x != 0 || y != 0 {
        intersections.push((x, y, cost));
    }
}

fn scan_for_intersections(list: &Vec<(i32, i32)>, coord: i32) -> impl Iterator<Item = &(i32, i32)> {
    list.iter()
        .filter(move |(start, end)| coord >= *start && coord <= *end)
}

impl PartOne {
    pub fn new() -> Self {
        Self {
            horizontal: HashMap::new(),
            vertical: HashMap::new(),
            intersections: Vec::new(),
            position: (0, 0),
            mode: Mode::Input,
        }
    }

    fn horiz(&mut self, y: i32, start: i32, end: i32) {
        self.horizontal
            .entry(y)
            .or_insert_with(Vec::new)
            .push((start, end));
    }

    fn vert(&mut self, x: i32, start: i32, end: i32) {
        self.vertical
            .entry(x)
            .or_insert_with(Vec::new)
            .push((start, end));
    }

    fn set(&mut self, dx: i32, dy: i32) -> &mut Self {
        let (x, y) = self.position;
        self.position = (x + dx, y + dy);
        let (x_min, x_max, y_min, y_max) = (
            x.min(self.position.0),
            x.max(self.position.0),
            y.min(self.position.1),
            y.max(self.position.1),
        );
        match self.mode {
            Mode::Input => {
                if dx == 0 {
                    self.vert(x, y_min, y_max);
                } else if dy == 0 {
                    self.horiz(y, x_min, x_max);
                } else {
                    panic!("diagonal movement");
                }
            }
            Mode::Intersections => {
                if dx == 0 {
                    for y in y_min..y_max {
                        if let Some(list) = self.horizontal.get(&y) {
                            for _ in scan_for_intersections(list, x) {
                                push_to_intersections(
                                    &mut self.intersections,
                                    (x, y, x.abs() + y.abs()),
                                );
                                break;
                            }
                        }
                    }
                } else if dy == 0 {
                    for x in x_min..x_max {
                        if let Some(list) = self.vertical.get(&x) {
                            for _ in scan_for_intersections(list, y) {
                                push_to_intersections(
                                    &mut self.intersections,
                                    (x, y, y.abs() + x.abs()),
                                );
                                break;
                            }
                        }
                    }
                } else {
                    panic!("diagonal movement");
                }
            }
        }
        self
    }
}

// part two

#[derive(Debug)]
struct PartTwo {
    cells: HashMap<(i32, i32), i32>,
    intersections: Vec<(i32, i32, i32)>,
    cost: i32,
    position: (i32, i32),
    mode: Mode,
}

impl PartTwo {
    pub fn new() -> Self {
        PartTwo {
            cells: HashMap::new(),
            cost: 0,
            position: (0, 0),
            intersections: Vec::new(),
            mode: Mode::Input,
        }
    }

    pub fn mark(&mut self) {
        self.step();
        match self.mode {
            Mode::Input => {
                self.cells.entry(self.position).or_insert(self.cost);
            }
            Mode::Intersections => {
                if let Some(&cost) = self.cells.get(&self.position) {
                    self.intersections
                        .push((self.position.0, self.position.1, cost + self.cost));
                }
            }
        };
    }

    pub fn step(&mut self) {
        self.cost += 1;
    }
}

impl Part for PartTwo {
    fn left(&mut self, mut amount: i32) {
        while amount > 0 {
            amount -= 1;
            self.position.0 -= 1;
            self.mark();
        }
    }

    fn right(&mut self, mut amount: i32) {
        while amount > 0 {
            amount -= 1;
            self.position.0 += 1;
            self.mark();
        }
    }

    fn up(&mut self, mut amount: i32) {
        while amount > 0 {
            amount -= 1;
            self.position.1 += 1;
            self.mark();
        }
    }

    fn down(&mut self, mut amount: i32) {
        while amount > 0 {
            amount -= 1;
            self.position.1 -= 1;
            self.mark();
        }
    }

    fn line(&mut self) {
        self.position = (0, 0);
        self.cost = 0;
        self.mode = Mode::Intersections;
    }

    fn into_answer(self) -> Vec<(i32, i32, i32)> {
        self.intersections
    }
}

fn run<T: Part, R: BufRead>(mut part: T, reader: R) -> Result<i32, Box<dyn Error>> {
    for line in reader.lines() {
        for instruction in line?.split(',') {
            let (direction, amount) = instruction.split_at(1);
            let num = amount.parse()?;
            match direction {
                "R" => part.right(num),
                "L" => part.left(num),
                "D" => part.down(num),
                "U" => part.up(num),
                x => panic!("unknown direction {}", x),
            };
        }
        part.line();
    }
    if let Some(min) = part
        .into_answer()
        .into_iter()
        .map(|(_, _, cost)| cost)
        .min()
    {
        Ok(min)
    } else {
        Err("no answer".into())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    run(PartTwo::new(), BufReader::new(std::io::stdin().lock()))
        .map(|answer| println!("{}", answer))
}
