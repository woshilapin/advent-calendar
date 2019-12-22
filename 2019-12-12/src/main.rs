use regex::Regex;
use std::{
    collections::HashMap,
    convert::TryFrom,
    env,
    fs::File,
    io::{self, BufRead, BufReader},
    num::ParseIntError,
    ops::{Deref, DerefMut},
};

fn prime_factors(mut number: u64) -> Vec<u64> {
    let mut primes = vec![2];
    let mut factors = Vec::new();
    loop {
        for prime in &primes {
            while number % prime == 0 {
                factors.push(*prime);
                number = number / prime;
            }
        }
        if number == 1 {
            break;
        }
        let mut next_prime = *primes.last().unwrap();
        'outer: loop {
            next_prime += 1;
            for prime in &primes {
                if next_prime % prime == 0 {
                    continue 'outer;
                }
            }
            // If not a multiple of any of the primes, then break the loop
            break;
        }
        primes.push(next_prime);
    }
    factors
}

fn ppcm(num1: u64, num2: u64) -> u64 {
    let factors1 = prime_factors(num1);
    let factors2 = prime_factors(num2);
    let mut index1 = 0;
    let mut index2 = 0;
    let mut ppcm = 1;
    while index1 < factors1.len() || index2 < factors2.len() {
        if index2 == factors2.len() {
            for i in index1..factors1.len() {
                ppcm *= factors1[i];
            }
            break;
        }
        if index1 == factors1.len() {
            for i in index2..factors2.len() {
                ppcm *= factors2[i];
            }
            break;
        }
        if factors1[index1] < factors2[index2] {
            ppcm *= factors1[index1];
            index1 += 1;
        } else if factors2[index2] < factors1[index1] {
            ppcm *= factors2[index2];
            index2 += 1;
        } else {
            ppcm *= factors1[index1];
            index1 += 1;
            index2 += 1;
        }
    }
    ppcm
}

type Position = (i64, i64, i64);
type Velocity = (i64, i64, i64);

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct Moon {
    position: Position,
    velocity: Velocity,
}

impl From<Position> for Moon {
    fn from(position: Position) -> Self {
        Moon {
            position,
            velocity: (0, 0, 0),
        }
    }
}

impl TryFrom<&str> for Moon {
    type Error = ParseIntError;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let pattern = Regex::new(r"<x=([0-9-]+), y=([0-9-]+), z=([0-9-]+)>").unwrap();
        let captures = pattern.captures(s).unwrap();
        let moon = Moon::from((
            captures[1].parse()?,
            captures[2].parse()?,
            captures[3].parse()?,
        ));
        Ok(moon)
    }
}

impl Moon {
    fn gravity(&mut self, moon: &Self) {
        if self.position.0 < moon.position.0 {
            self.velocity.0 += 1;
        } else if self.position.0 > moon.position.0 {
            self.velocity.0 -= 1;
        }
        if self.position.1 < moon.position.1 {
            self.velocity.1 += 1;
        } else if self.position.1 > moon.position.1 {
            self.velocity.1 -= 1;
        }
        if self.position.2 < moon.position.2 {
            self.velocity.2 += 1;
        } else if self.position.2 > moon.position.2 {
            self.velocity.2 -= 1;
        }
    }

    fn step(&mut self) {
        self.position.0 += self.velocity.0;
        self.position.1 += self.velocity.1;
        self.position.2 += self.velocity.2;
    }

    fn potential_energy(&self) -> i64 {
        self.position.0.abs() + self.position.1.abs() + self.position.2.abs()
    }

    fn kinetic_energy(&self) -> i64 {
        self.velocity.0.abs() + self.velocity.1.abs() + self.velocity.2.abs()
    }

    fn energy(&self) -> i64 {
        self.potential_energy() * self.kinetic_energy()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct System {
    moons: HashMap<String, Moon>,
}

impl System {
    fn step(&mut self) {
        let mut system = self.clone();
        for moon in self.keys() {
            for to in self.keys() {
                system.get_mut(moon).unwrap().gravity(self.get(to).unwrap());
            }
        }
        for moon in system.values_mut() {
            moon.step();
        }
        *self = system;
    }

    fn steps(&mut self, count: usize) {
        for _ in 0..count {
            self.step();
        }
    }

    fn energy(&self) -> i64 {
        self.values().map(|moon| moon.energy()).sum()
    }

    fn next_cycle(&mut self) -> u64 {
        fn cycle(mut state: Vec<(i64, i64)>) -> u64 {
            let init = state.clone();
            let mut steps = 0;
            loop {
                let mut new_state = Vec::new();
                for (position, mut velocity) in &state {
                    for (p, _) in &state {
                        if position < p {
                            velocity += 1;
                        } else if position > p {
                            velocity -= 1;
                        }
                    }
                    new_state.push((position + velocity, velocity));
                }
                state = new_state;
                steps += 1;
                if state == init {
                    break;
                }
            }
            steps
        }
        let init_x: Vec<_> = self
            .moons
            .values()
            .map(|moon| (moon.position.0, moon.velocity.0))
            .collect();
        let init_y: Vec<_> = self
            .moons
            .values()
            .map(|moon| (moon.position.1, moon.velocity.1))
            .collect();
        let init_z: Vec<_> = self
            .moons
            .values()
            .map(|moon| (moon.position.2, moon.velocity.2))
            .collect();
        let cycle_x = cycle(init_x);
        let cycle_y = cycle(init_y);
        let cycle_z = cycle(init_z);
        ppcm(ppcm(cycle_x, cycle_y), cycle_z)
    }
}

impl Deref for System {
    type Target = HashMap<String, Moon>;

    fn deref(&self) -> &Self::Target {
        &self.moons
    }
}

impl DerefMut for System {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.moons
    }
}

fn main() -> Result<(), io::Error> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("Only one argument is accepted, received {:#?}", args);
    }
    let file = File::open(&args[1])?;
    let mut reader = BufReader::new(file);

    let mut system = System::default();
    for moon_name in vec!["io", "europa", "ganymede", "callisto"] {
        let mut input = String::new();
        reader.read_line(&mut input)?;
        let io = Moon::try_from(input.trim()).unwrap();
        system.insert(moon_name.to_string(), io);
    }

    println!(
        "The next cycle of the system is in {} steps",
        system.clone().next_cycle()
    );
    system.steps(1000);
    println!("The total energy after 1000 steps is {}", system.energy());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_prime_factors() {
        assert_eq!(Vec::<u64>::new(), prime_factors(1));
        assert_eq!(vec![2], prime_factors(2));
        assert_eq!(vec![2, 3, 3, 5, 7, 7], prime_factors(2 * 3 * 3 * 5 * 7 * 7));
        assert_eq!(vec![2, 2, 2, 3, 7, 1597], prime_factors(268296));
        assert_eq!(vec![2, 115807], prime_factors(231614));
        assert_eq!(vec![2, 107, 109], prime_factors(23326));
    }

    #[test]
    fn smaller_multiple() {
        assert_eq!(2, ppcm(2, 2));
        assert_eq!(15, ppcm(3, 5));
        assert_eq!(90, ppcm(10, 18));
        assert_eq!(31070554872, ppcm(268296, 231614));
        assert_eq!(362375881472136, ppcm(ppcm(268296, 231614), 23326));
    }

    #[test]
    fn steps_10() {
        let mut system = System::default();
        system.insert("io".to_string(), Moon::from((-1, 0, 2)));
        system.insert("europa".to_string(), Moon::from((2, -10, -7)));
        system.insert("ganymede".to_string(), Moon::from((4, -8, 8)));
        system.insert("callisto".to_string(), Moon::from((3, 5, -1)));

        assert_eq!((0, 0, 0), system.get("io").unwrap().velocity);
        assert_eq!((0, 0, 0), system.get("europa").unwrap().velocity);
        assert_eq!((0, 0, 0), system.get("ganymede").unwrap().velocity);
        assert_eq!((0, 0, 0), system.get("callisto").unwrap().velocity);

        system.step();
        assert_eq!((2, -1, 1), system.get("io").unwrap().position);
        assert_eq!((3, -1, -1), system.get("io").unwrap().velocity);
        assert_eq!((3, -7, -4), system.get("europa").unwrap().position);
        assert_eq!((1, 3, 3), system.get("europa").unwrap().velocity);
        assert_eq!((1, -7, 5), system.get("ganymede").unwrap().position);
        assert_eq!((-3, 1, -3), system.get("ganymede").unwrap().velocity);
        assert_eq!((2, 2, 0), system.get("callisto").unwrap().position);
        assert_eq!((-1, -3, 1), system.get("callisto").unwrap().velocity);

        system.step();
        assert_eq!((5, -3, -1), system.get("io").unwrap().position);
        assert_eq!((3, -2, -2), system.get("io").unwrap().velocity);
        assert_eq!((1, -2, 2), system.get("europa").unwrap().position);
        assert_eq!((-2, 5, 6), system.get("europa").unwrap().velocity);
        assert_eq!((1, -4, -1), system.get("ganymede").unwrap().position);
        assert_eq!((0, 3, -6), system.get("ganymede").unwrap().velocity);
        assert_eq!((1, -4, 2), system.get("callisto").unwrap().position);
        assert_eq!((-1, -6, 2), system.get("callisto").unwrap().velocity);

        system.step();
        assert_eq!((5, -6, -1), system.get("io").unwrap().position);
        assert_eq!((0, -3, 0), system.get("io").unwrap().velocity);
        assert_eq!((0, 0, 6), system.get("europa").unwrap().position);
        assert_eq!((-1, 2, 4), system.get("europa").unwrap().velocity);
        assert_eq!((2, 1, -5), system.get("ganymede").unwrap().position);
        assert_eq!((1, 5, -4), system.get("ganymede").unwrap().velocity);
        assert_eq!((1, -8, 2), system.get("callisto").unwrap().position);
        assert_eq!((0, -4, 0), system.get("callisto").unwrap().velocity);

        system.step();
        assert_eq!((2, -8, 0), system.get("io").unwrap().position);
        assert_eq!((-3, -2, 1), system.get("io").unwrap().velocity);
        assert_eq!((2, 1, 7), system.get("europa").unwrap().position);
        assert_eq!((2, 1, 1), system.get("europa").unwrap().velocity);
        assert_eq!((2, 3, -6), system.get("ganymede").unwrap().position);
        assert_eq!((0, 2, -1), system.get("ganymede").unwrap().velocity);
        assert_eq!((2, -9, 1), system.get("callisto").unwrap().position);
        assert_eq!((1, -1, -1), system.get("callisto").unwrap().velocity);

        system.step();
        assert_eq!((-1, -9, 2), system.get("io").unwrap().position);
        assert_eq!((-3, -1, 2), system.get("io").unwrap().velocity);
        assert_eq!((4, 1, 5), system.get("europa").unwrap().position);
        assert_eq!((2, 0, -2), system.get("europa").unwrap().velocity);
        assert_eq!((2, 2, -4), system.get("ganymede").unwrap().position);
        assert_eq!((0, -1, 2), system.get("ganymede").unwrap().velocity);
        assert_eq!((3, -7, -1), system.get("callisto").unwrap().position);
        assert_eq!((1, 2, -2), system.get("callisto").unwrap().velocity);

        system.step();
        assert_eq!((-1, -7, 3), system.get("io").unwrap().position);
        assert_eq!((0, 2, 1), system.get("io").unwrap().velocity);
        assert_eq!((3, 0, 0), system.get("europa").unwrap().position);
        assert_eq!((-1, -1, -5), system.get("europa").unwrap().velocity);
        assert_eq!((3, -2, 1), system.get("ganymede").unwrap().position);
        assert_eq!((1, -4, 5), system.get("ganymede").unwrap().velocity);
        assert_eq!((3, -4, -2), system.get("callisto").unwrap().position);
        assert_eq!((0, 3, -1), system.get("callisto").unwrap().velocity);

        system.step();
        assert_eq!((2, -2, 1), system.get("io").unwrap().position);
        assert_eq!((3, 5, -2), system.get("io").unwrap().velocity);
        assert_eq!((1, -4, -4), system.get("europa").unwrap().position);
        assert_eq!((-2, -4, -4), system.get("europa").unwrap().velocity);
        assert_eq!((3, -7, 5), system.get("ganymede").unwrap().position);
        assert_eq!((0, -5, 4), system.get("ganymede").unwrap().velocity);
        assert_eq!((2, 0, 0), system.get("callisto").unwrap().position);
        assert_eq!((-1, 4, 2), system.get("callisto").unwrap().velocity);

        system.step();
        assert_eq!((5, 2, -2), system.get("io").unwrap().position);
        assert_eq!((3, 4, -3), system.get("io").unwrap().velocity);
        assert_eq!((2, -7, -5), system.get("europa").unwrap().position);
        assert_eq!((1, -3, -1), system.get("europa").unwrap().velocity);
        assert_eq!((0, -9, 6), system.get("ganymede").unwrap().position);
        assert_eq!((-3, -2, 1), system.get("ganymede").unwrap().velocity);
        assert_eq!((1, 1, 3), system.get("callisto").unwrap().position);
        assert_eq!((-1, 1, 3), system.get("callisto").unwrap().velocity);

        system.step();
        assert_eq!((5, 3, -4), system.get("io").unwrap().position);
        assert_eq!((0, 1, -2), system.get("io").unwrap().velocity);
        assert_eq!((2, -9, -3), system.get("europa").unwrap().position);
        assert_eq!((0, -2, 2), system.get("europa").unwrap().velocity);
        assert_eq!((0, -8, 4), system.get("ganymede").unwrap().position);
        assert_eq!((0, 1, -2), system.get("ganymede").unwrap().velocity);
        assert_eq!((1, 1, 5), system.get("callisto").unwrap().position);
        assert_eq!((0, 0, 2), system.get("callisto").unwrap().velocity);

        system.step();
        assert_eq!((2, 1, -3), system.get("io").unwrap().position);
        assert_eq!((-3, -2, 1), system.get("io").unwrap().velocity);
        assert_eq!((1, -8, 0), system.get("europa").unwrap().position);
        assert_eq!((-1, 1, 3), system.get("europa").unwrap().velocity);
        assert_eq!((3, -6, 1), system.get("ganymede").unwrap().position);
        assert_eq!((3, 2, -3), system.get("ganymede").unwrap().velocity);
        assert_eq!((2, 0, 4), system.get("callisto").unwrap().position);
        assert_eq!((1, -1, -1), system.get("callisto").unwrap().velocity);

        assert_eq!(179, system.energy());
    }

    #[test]
    fn steps_100() {
        let mut system = System::default();
        system.insert("io".to_string(), Moon::from((-8, -10, 0)));
        system.insert("europa".to_string(), Moon::from((5, 5, 10)));
        system.insert("ganymede".to_string(), Moon::from((2, -7, 3)));
        system.insert("callisto".to_string(), Moon::from((9, -8, -3)));

        assert_eq!((0, 0, 0), system.get("io").unwrap().velocity);
        assert_eq!((0, 0, 0), system.get("europa").unwrap().velocity);
        assert_eq!((0, 0, 0), system.get("ganymede").unwrap().velocity);
        assert_eq!((0, 0, 0), system.get("callisto").unwrap().velocity);

        system.steps(10);
        assert_eq!((-9, -10, 1), system.get("io").unwrap().position);
        assert_eq!((-2, -2, -1), system.get("io").unwrap().velocity);
        assert_eq!((4, 10, 9), system.get("europa").unwrap().position);
        assert_eq!((-3, 7, -2), system.get("europa").unwrap().velocity);
        assert_eq!((8, -10, -3), system.get("ganymede").unwrap().position);
        assert_eq!((5, -1, -2), system.get("ganymede").unwrap().velocity);
        assert_eq!((5, -10, 3), system.get("callisto").unwrap().position);
        assert_eq!((0, -4, 5), system.get("callisto").unwrap().velocity);

        system.steps(10);
        assert_eq!((-10, 3, -4), system.get("io").unwrap().position);
        assert_eq!((-5, 2, 0), system.get("io").unwrap().velocity);
        assert_eq!((5, -25, 6), system.get("europa").unwrap().position);
        assert_eq!((1, 1, -4), system.get("europa").unwrap().velocity);
        assert_eq!((13, 1, 1), system.get("ganymede").unwrap().position);
        assert_eq!((5, -2, 2), system.get("ganymede").unwrap().velocity);
        assert_eq!((0, 1, 7), system.get("callisto").unwrap().position);
        assert_eq!((-1, -1, 2), system.get("callisto").unwrap().velocity);

        system.steps(10);
        assert_eq!((15, -6, -9), system.get("io").unwrap().position);
        assert_eq!((-5, 4, 0), system.get("io").unwrap().velocity);
        assert_eq!((-4, -11, 3), system.get("europa").unwrap().position);
        assert_eq!((-3, -10, 0), system.get("europa").unwrap().velocity);
        assert_eq!((0, -1, 11), system.get("ganymede").unwrap().position);
        assert_eq!((7, 4, 3), system.get("ganymede").unwrap().velocity);
        assert_eq!((-3, -2, 5), system.get("callisto").unwrap().position);
        assert_eq!((1, 2, -3), system.get("callisto").unwrap().velocity);

        system.steps(10);
        assert_eq!((14, -12, -4), system.get("io").unwrap().position);
        assert_eq!((11, 3, 0), system.get("io").unwrap().velocity);
        assert_eq!((-1, 18, 8), system.get("europa").unwrap().position);
        assert_eq!((-5, 2, 3), system.get("europa").unwrap().velocity);
        assert_eq!((-5, -14, 8), system.get("ganymede").unwrap().position);
        assert_eq!((1, -2, 0), system.get("ganymede").unwrap().velocity);
        assert_eq!((0, -12, -2), system.get("callisto").unwrap().position);
        assert_eq!((-7, -3, -3), system.get("callisto").unwrap().velocity);

        system.steps(10);
        assert_eq!((-23, 4, 1), system.get("io").unwrap().position);
        assert_eq!((-7, -1, 2), system.get("io").unwrap().velocity);
        assert_eq!((20, -31, 13), system.get("europa").unwrap().position);
        assert_eq!((5, 3, 4), system.get("europa").unwrap().velocity);
        assert_eq!((-4, 6, 1), system.get("ganymede").unwrap().position);
        assert_eq!((-1, 1, -3), system.get("ganymede").unwrap().velocity);
        assert_eq!((15, 1, -5), system.get("callisto").unwrap().position);
        assert_eq!((3, -3, -3), system.get("callisto").unwrap().velocity);

        system.steps(10);
        assert_eq!((36, -10, 6), system.get("io").unwrap().position);
        assert_eq!((5, 0, 3), system.get("io").unwrap().velocity);
        assert_eq!((-18, 10, 9), system.get("europa").unwrap().position);
        assert_eq!((-3, -7, 5), system.get("europa").unwrap().velocity);
        assert_eq!((8, -12, -3), system.get("ganymede").unwrap().position);
        assert_eq!((-2, 1, -7), system.get("ganymede").unwrap().velocity);
        assert_eq!((-18, -8, -2), system.get("callisto").unwrap().position);
        assert_eq!((0, 6, -1), system.get("callisto").unwrap().velocity);

        system.steps(10);
        assert_eq!((-33, -6, 5), system.get("io").unwrap().position);
        assert_eq!((-5, -4, 7), system.get("io").unwrap().velocity);
        assert_eq!((13, -9, 2), system.get("europa").unwrap().position);
        assert_eq!((-2, 11, 3), system.get("europa").unwrap().velocity);
        assert_eq!((11, -8, 2), system.get("ganymede").unwrap().position);
        assert_eq!((8, -6, -7), system.get("ganymede").unwrap().velocity);
        assert_eq!((17, 3, 1), system.get("callisto").unwrap().position);
        assert_eq!((-1, -1, -3), system.get("callisto").unwrap().velocity);

        system.steps(10);
        assert_eq!((30, -8, 3), system.get("io").unwrap().position);
        assert_eq!((3, 3, 0), system.get("io").unwrap().velocity);
        assert_eq!((-2, -4, 0), system.get("europa").unwrap().position);
        assert_eq!((4, -13, 2), system.get("europa").unwrap().velocity);
        assert_eq!((-18, -7, 15), system.get("ganymede").unwrap().position);
        assert_eq!((-8, 2, -2), system.get("ganymede").unwrap().velocity);
        assert_eq!((-2, -1, -8), system.get("callisto").unwrap().position);
        assert_eq!((1, 8, 0), system.get("callisto").unwrap().velocity);

        system.steps(10);
        assert_eq!((-25, -1, 4), system.get("io").unwrap().position);
        assert_eq!((1, -3, 4), system.get("io").unwrap().velocity);
        assert_eq!((2, -9, 0), system.get("europa").unwrap().position);
        assert_eq!((-3, 13, -1), system.get("europa").unwrap().velocity);
        assert_eq!((32, -8, 14), system.get("ganymede").unwrap().position);
        assert_eq!((5, -4, 6), system.get("ganymede").unwrap().velocity);
        assert_eq!((-1, -2, -8), system.get("callisto").unwrap().position);
        assert_eq!((-3, -6, -9), system.get("callisto").unwrap().velocity);

        system.steps(10);
        assert_eq!((8, -12, -9), system.get("io").unwrap().position);
        assert_eq!((-7, 3, 0), system.get("io").unwrap().velocity);
        assert_eq!((13, 16, -3), system.get("europa").unwrap().position);
        assert_eq!((3, -11, -5), system.get("europa").unwrap().velocity);
        assert_eq!((-29, -11, -1), system.get("ganymede").unwrap().position);
        assert_eq!((-3, 7, 4), system.get("ganymede").unwrap().velocity);
        assert_eq!((16, -13, 23), system.get("callisto").unwrap().position);
        assert_eq!((7, 1, 1), system.get("callisto").unwrap().velocity);

        assert_eq!(1940, system.energy());
    }

    #[test]
    fn moon() {
        let moon = Moon::try_from("<x=5, y=-8, z=3>").unwrap();
        assert_eq!((5, -8, 3), moon.position);
        assert_eq!((0, 0, 0), moon.velocity);
    }

    #[test]
    fn next_cycle() {
        let mut system = System::default();
        system.insert("io".to_string(), Moon::from((-1, 0, 2)));
        system.insert("europa".to_string(), Moon::from((2, -10, -7)));
        system.insert("ganymede".to_string(), Moon::from((4, -8, 8)));
        system.insert("callisto".to_string(), Moon::from((3, 5, -1)));

        assert_eq!(2772, system.next_cycle());
    }

    #[test]
    fn next_cycle_long() {
        let mut system = System::default();
        system.insert("io".to_string(), Moon::from((-8, -10, 0)));
        system.insert("europa".to_string(), Moon::from((5, 5, 10)));
        system.insert("ganymede".to_string(), Moon::from((2, -7, 3)));
        system.insert("callisto".to_string(), Moon::from((9, -8, -3)));

        assert_eq!(4686774924, system.next_cycle());
    }
}
