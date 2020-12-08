use std::{
    env,
    fs::File,
    io::{BufRead, BufReader},
};

struct Bounds {
    min: usize,
    max: usize,
}

impl std::str::FromStr for Bounds {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.split('-');
        let min: usize = split
            .next()
            .ok_or_else(|| "expect at least one element in the range (e.g. '1-3')".to_string())?
            .parse()
            .map_err(|e| format!("expect the String to be parseable as a usize: {}", e))?;
        let max: usize = split
            .next()
            .ok_or_else(|| "expect at least two elements in the range (e.g. '1-3')".to_string())?
            .parse()
            .map_err(|e| format!("expect the String to be parseable as a usize: {}", e))?;
        let bounds = Bounds { min, max };
        Ok(bounds)
    }
}

struct Policy {
    bounds: Bounds,
    constraint: char,
}

impl std::str::FromStr for Policy {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.split(' ');
        let bounds: Bounds = split
            .next()
            .ok_or_else(|| "expect at least a range (e.g. '1-3')".to_string())?
            .parse()
            .map_err(|e| format!("expect the bounds to be in the format '1-3': {}", e))?;
        let c = split
            .next()
            .ok_or_else(|| "expect at least a character as the constraint".to_string())?;
        let constraint = if c.len() != 1 {
            return Err("constraint should be one char long".to_string());
        } else {
            c.chars()
                .next()
                .expect("expect at least one char for constraint")
        };
        let policy = Policy { bounds, constraint };
        Ok(policy)
    }
}

struct Entry {
    policy: Policy,
    password: String,
}

impl std::str::FromStr for Entry {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.split(": ");
        let policy: Policy = split
            .next()
            .ok_or_else(|| "expect at least a policy (e.g. '1-3 a')".to_string())?
            .parse()
            .map_err(|e| format!("expect the policy to be in the format '1-3 a': {}", e))?;
        let password: String = split
            .next()
            .ok_or_else(|| "expect a password after ':'".to_string())?
            .to_owned();
        let entry = Entry { policy, password };
        Ok(entry)
    }
}

impl Entry {
    #[cfg(not(feature = "positional"))]
    fn check(&self) -> bool {
        let count = self.password.matches(self.policy.constraint).count();
        count >= self.policy.bounds.min && count <= self.policy.bounds.max
    }
    #[cfg(feature = "positional")]
    fn check(&self) -> bool {
        let chars: Vec<char> = self.password.chars().collect();
        let is_first = chars[self.policy.bounds.min - 1] == self.policy.constraint;
        let is_second = chars[self.policy.bounds.max - 1] == self.policy.constraint;
        // XOR
        (is_first || is_second) && !(is_first && is_second)
    }
}

fn filter_valid_entries(lines: impl Iterator<Item = String>) -> impl Iterator<Item = Entry> {
    lines.filter_map(|line| {
        let entry: Entry = line.parse().ok()?;
        if entry.check() {
            Some(entry)
        } else {
            None
        }
    })
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("Only one argument is accepted, received {:#?}", args);
    }
    let file = File::open(&args[1]).expect(format!("expect file '{}' to exist", args[1]).as_str());
    let reader = BufReader::new(file);
    let valid_entries = filter_valid_entries(
        reader
            .lines()
            .map(|line| line.expect("expect a valid String")),
    );
    let count = valid_entries.count();
    println!("Total of valid entries is {}", count);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(not(feature = "positional"))]
    fn password_philosophy() {
        let entries = vec!["1-3 a: abcde", "1-3 b: cdefg", "2-9 c: ccccccccc"];
        let valid_entries =
            filter_valid_entries(entries.into_iter().map(std::borrow::ToOwned::to_owned));
        assert_eq!(2, valid_entries.count());
    }

    #[test]
    #[cfg(feature = "positional")]
    fn password_philosophy() {
        let entries = vec!["1-3 a: abcde", "1-3 b: cdefg", "2-9 c: ccccccccc"];
        let valid_entries =
            filter_valid_entries(entries.into_iter().map(std::borrow::ToOwned::to_owned));
        assert_eq!(1, valid_entries.count());
    }
}
