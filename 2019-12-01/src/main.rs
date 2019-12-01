use std::{
    env,
    fs::File,
    io::{BufRead, BufReader, Result},
};

#[cfg(not(feature = "with-fuel"))]
fn calculate_fuel_requirement(mass: u64) -> u64 {
    if mass >= 6 {
        (mass / 3) - 2
    } else {
        0
    }
}

#[cfg(feature = "with-fuel")]
fn calculate_fuel_requirement(mass: u64) -> u64 {
    if mass >= 6 {
        let fuel = (mass / 3) - 2;
        fuel + calculate_fuel_requirement(fuel)
    } else {
        0
    }
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("Only one argument is accepted, received {:#?}", args);
    }
    let file = File::open(&args[1])?;
    let reader = BufReader::new(file);
    let mut fuel_requirement = 0;
    for line in reader.lines() {
        let mass: u64 = line?.parse().unwrap();
        fuel_requirement += calculate_fuel_requirement(mass);
    }
    println!("Total fuel requirement is {}", fuel_requirement);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(not(feature = "with-fuel"))]
    fn fuel_requirements() {
        assert_eq!(2, calculate_fuel_requirement(12));
        assert_eq!(2, calculate_fuel_requirement(14));
        assert_eq!(654, calculate_fuel_requirement(1969));
        assert_eq!(33583, calculate_fuel_requirement(100756));
    }

    #[test]
    #[cfg(feature = "with-fuel")]
    fn fuel_requirements() {
        assert_eq!(2, calculate_fuel_requirement(12));
        assert_eq!(2, calculate_fuel_requirement(14));
        assert_eq!(966, calculate_fuel_requirement(1969));
        assert_eq!(50346, calculate_fuel_requirement(100756));
    }
}
