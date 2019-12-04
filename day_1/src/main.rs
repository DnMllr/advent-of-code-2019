use std::error::Error;
use std::io;
use std::io::prelude::*;
use std::io::BufReader;

fn calculate_fuel(n: f64) -> f64 {
    (n / 3.).floor() - 2.
}

fn add_fuel(sum: &mut f64, module: f64) {
    let mut delta = calculate_fuel(module);
    while delta > 0. {
        *sum += delta;
        delta = calculate_fuel(delta);
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut sum = 0.;
    for line in BufReader::new(io::stdin().lock()).lines() {
        add_fuel(&mut sum, line?.parse()?);
    }
    println!("\n{}", sum);
    Ok(())
}
