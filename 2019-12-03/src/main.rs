use std::{
    collections::HashSet,
    env,
    fs::File,
    io::{self, BufRead, BufReader},
    str::FromStr,
};
use thiserror::Error;

#[derive(Debug, Error)]
enum MyError {
    #[error("'{to_parse}' can not be parsed as a number")]
    NotANumber {
        to_parse: String,
        #[source]
        source: std::num::ParseIntError,
    },
    #[error("'{to_parse}' can not be parsed as a Direction")]
    UnknownDirection { to_parse: String },
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct Position {
    x: i64,
    y: i64,
}
type Positions = Vec<Position>;

impl From<(i64, i64)> for Position {
    fn from((x, y): (i64, i64)) -> Position {
        Position { x, y }
    }
}

enum Direction {
    Up,
    Down,
    Left,
    Right,
}
impl FromStr for Direction {
    type Err = MyError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Direction::*;
        match s {
            "U" => Ok(Up),
            "D" => Ok(Down),
            "L" => Ok(Left),
            "R" => Ok(Right),
            unknown => Err(MyError::UnknownDirection {
                to_parse: unknown.to_string(),
            }),
        }
    }
}

struct Directive {
    direction: Direction,
    length: i64,
}
type Directives = Vec<Directive>;
impl FromStr for Directive {
    type Err = MyError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let direction: Direction = s[0..1].parse()?;
        let length: i64 = s[1..].parse().map_err(|source| MyError::NotANumber {
            source,
            to_parse: s[1..].to_string(),
        })?;
        Ok(Directive { direction, length })
    }
}

impl Directive {
    fn positions_from(&self, init: &Position) -> Positions {
        let mut positions = Vec::new();
        let mut last_position = init.clone();
        for _ in 0..self.length {
            use Direction::*;
            match self.direction {
                Up => {
                    last_position = Position {
                        x: last_position.x,
                        y: last_position.y + 1,
                    }
                }
                Down => {
                    last_position = Position {
                        x: last_position.x,
                        y: last_position.y - 1,
                    }
                }
                Left => {
                    last_position = Position {
                        x: last_position.x - 1,
                        y: last_position.y,
                    }
                }
                Right => {
                    last_position = Position {
                        x: last_position.x + 1,
                        y: last_position.y,
                    }
                }
            }
            positions.push(last_position.clone());
        }
        positions
    }
}

fn wire_directives(wire_path: &str) -> Directives {
    wire_path
        .trim()
        .split(",")
        .map(Directive::from_str)
        .map(Result::unwrap)
        .collect()
}

fn wire_positions(wire_directives: Directives) -> Positions {
    let init = Position::from((0, 0));
    let mut positions = Vec::new();
    let mut last_position = None;
    for directive in wire_directives {
        let new_positions = directive.positions_from(last_position.unwrap_or_else(|| &init));
        positions.extend(new_positions);
        last_position = positions.last();
    }
    positions
}

#[cfg(not(feature = "shortest"))]
fn optimized_crossed_wires(wire1_directives: Directives, wire2_directives: Directives) -> i64 {
    fn manhattan_distance(position: &Position) -> i64 {
        position.x.abs() + position.y.abs()
    }

    let wire1_positions: HashSet<_> = wire_positions(wire1_directives).into_iter().collect();
    let wire2_positions: HashSet<_> = wire_positions(wire2_directives).into_iter().collect();
    wire1_positions
        .intersection(&wire2_positions)
        .map(manhattan_distance)
        .min()
        .unwrap()
}

#[cfg(feature = "shortest")]
fn optimized_crossed_wires(wire1_directives: Directives, wire2_directives: Directives) -> i64 {
    let wire1_positions = wire_positions(wire1_directives);
    let wire2_positions = wire_positions(wire2_directives);
    let wire1_positions_set: HashSet<_> = wire1_positions.iter().collect();
    let wire2_positions_set: HashSet<_> = wire2_positions.iter().collect();
    let intersections: HashSet<_> = wire1_positions_set
        .intersection(&wire2_positions_set)
        .collect();
    let mut min_steps = std::i64::MAX;
    for intersection in intersections {
        let wire1_steps = wire1_positions
            .iter()
            .position(|position| position == *intersection)
            .map(|steps| steps + 1);
        let wire2_steps = wire2_positions
            .iter()
            .position(|position| position == *intersection)
            .map(|steps| steps + 1);
        if let (Some(wire1_steps), Some(wire2_steps)) = (wire1_steps, wire2_steps) {
            let total_steps = (wire1_steps + wire2_steps) as i64;
            if total_steps < min_steps {
                min_steps = total_steps;
            }
        }
    }
    min_steps
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("Only one argument is accepted, received {:#?}", args);
    }
    let file = File::open(&args[1])?;
    let mut reader = BufReader::new(file);
    let mut wire1 = String::new();
    reader.read_line(&mut wire1)?;
    let mut wire2 = String::new();
    reader.read_line(&mut wire2)?;
    let wire1_directives = wire_directives(&wire1);
    let wire2_directives = wire_directives(&wire2);
    let distance = optimized_crossed_wires(wire1_directives, wire2_directives);
    println!("The optimized intersection is {} unit away", distance);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(not(feature = "shortest"))]
    #[test]
    fn cross_wires() {
        let wire1 = wire_directives("R8,U5,L5,D3");
        let wire2 = wire_directives("U7,R6,D4,L4");
        assert_eq!(6, optimized_crossed_wires(wire1, wire2));
        let wire1 = wire_directives("R75,D30,R83,U83,L12,D49,R71,U7,L72");
        let wire2 = wire_directives("U62,R66,U55,R34,D71,R55,D58,R83");
        assert_eq!(159, optimized_crossed_wires(wire1, wire2));
        let wire1 = wire_directives("R98,U47,R26,D63,R33,U87,L62,D20,R33,U53,R51");
        let wire2 = wire_directives("U98,R91,D20,R16,D67,R40,U7,R15,U6,R7");
        assert_eq!(135, optimized_crossed_wires(wire1, wire2));
    }

    #[cfg(feature = "shortest")]
    #[test]
    fn cross_wires() {
        let wire1 = wire_directives("R8,U5,L5,D3");
        let wire2 = wire_directives("U7,R6,D4,L4");
        assert_eq!(30, optimized_crossed_wires(wire1, wire2));
        let wire1 = wire_directives("R75,D30,R83,U83,L12,D49,R71,U7,L72");
        let wire2 = wire_directives("U62,R66,U55,R34,D71,R55,D58,R83");
        assert_eq!(610, optimized_crossed_wires(wire1, wire2));
        let wire1 = wire_directives("R98,U47,R26,D63,R33,U87,L62,D20,R33,U53,R51");
        let wire2 = wire_directives("U98,R91,D20,R16,D67,R40,U7,R15,U6,R7");
        assert_eq!(410, optimized_crossed_wires(wire1, wire2));
    }
}
