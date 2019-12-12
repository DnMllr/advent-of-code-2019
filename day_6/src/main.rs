use std::collections::{HashMap, VecDeque};
use std::io::{BufRead, BufReader};

use anyhow::Result;
use std::fs::File;
use std::iter;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ErrorKinds {
    #[error("didn't find \")\" in string at line {0}: {1}")]
    MissingOrbitSymbol(usize, String),

    #[error("unable to open file")]
    UnableToOpen(std::io::Error),
}

fn collect<T: BufRead>(read_from: T) -> Result<HashMap<String, Vec<String>>> {
    let mut m = HashMap::new();
    for (line_number, line) in read_from.lines().enumerate() {
        let line = line?;
        let at = line
            .find(')')
            .ok_or_else(|| ErrorKinds::MissingOrbitSymbol(line_number, line.clone()))?;
        let (attractor, orbit) = line.split_at(at);
        m.entry(attractor.to_owned())
            .or_insert_with(Vec::new)
            .push(orbit[1..].to_owned());
    }
    Ok(m)
}

fn count_children(key: &str, map: &HashMap<String, Vec<String>>, depth: usize) -> usize {
    map.get(key).map_or(depth, |entry| {
        entry
            .iter()
            .fold(depth, |a, b| a + count_children(b, map, depth + 1))
    })
}

fn count_relationships(map: &HashMap<String, Vec<String>>, depth: usize) -> usize {
    count_children("COM", map, depth)
}

fn part1(map: &HashMap<String, Vec<String>>) -> Result<usize> {
    Ok(count_relationships(map, 0))
}

fn inner_find<'a>(
    look_for: &str,
    at: &'a str,
    map: &'a HashMap<String, Vec<String>>,
    v: &mut VecDeque<&'a str>,
) -> bool {
    if let Some(children) = map.get(at) {
        for child in children {
            if child == look_for || inner_find(look_for, child, map, v) {
                v.push_front(at);
                return true;
            }
        }
    }
    false
}

fn find<'a>(key: &str, map: &'a HashMap<String, Vec<String>>) -> VecDeque<&'a str> {
    let mut results = VecDeque::new();
    inner_find(key, "COM", map, &mut results);
    results
}

fn part2(map: &HashMap<String, Vec<String>>) -> Result<usize> {
    let me = find("YOU", map);
    let santa = find("SAN", map);
    let stuff = me
        .into_iter()
        .chain(iter::repeat(""))
        .zip(santa.into_iter().chain(iter::repeat("")))
        .skip_while(|(l, r)| l == r)
        .take_while(|(l, r)| !l.is_empty() || !r.is_empty())
        .fold(0, |x, (l, r)| {
            !l.is_empty() as usize + !r.is_empty() as usize + x
        });
    Ok(stuff)
}

fn main() -> Result<()> {
    let map = if let Some(file_name) = std::env::args().nth(1) {
        let reader = BufReader::new(File::open(file_name).map_err(ErrorKinds::UnableToOpen)?);
        collect(reader)?
    } else {
        let stdin = std::io::stdin();
        let reader = BufReader::new(stdin.lock());
        collect(reader)?
    };
    println!("part 1 result >> {}", part1(&map)?);
    println!("part 2 result >> {}", part2(&map)?);
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_part_1() -> Result<()> {
        if let Ok(answer) = part1(&collect(BufReader::new(
            "COM)B\nB)C\nC)D\nD)E\nE)F\nB)G\nG)H\nD)I\nE)J\nJ)K\nK)L".as_bytes(),
        ))?) {
            assert_eq!(
                answer, 42,
                "the example problem for part 1 should be correct"
            )
        } else {
            assert!(false, "part1 returned error on test case");
        }
        Ok(())
    }

    #[test]
    fn test_find() -> Result<()> {
        let map = collect(BufReader::new(
            "COM)B\nB)C\nC)D\nD)E\nE)F\nB)G\nG)H\nD)I\nE)J\nJ)K\nK)L".as_bytes(),
        ))?;
        let result = find("K", &map);
        assert_eq!(
            result,
            vec!["COM", "B", "C", "D", "E", "J"],
            "find should return path"
        );
        Ok(())
    }
}
