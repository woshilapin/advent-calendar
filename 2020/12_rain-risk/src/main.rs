#[cfg(not(feature = "waypoint"))]
#[derive(Debug, Clone, Copy)]
enum Direction {
    Forward,
    Left,
    Right,
}

#[cfg(not(feature = "waypoint"))]
#[derive(Debug, Clone, Copy)]
enum Cardinal {
    North,
    East,
    South,
    West,
}
#[cfg(not(feature = "waypoint"))]
impl Cardinal {
    fn turn(&self, direction: Direction) -> Self {
        use Cardinal::*;
        use Direction::*;
        match (*self, direction) {
            (North, Left) | (South, Right) | (West, Forward) => West,
            (North, Right) | (South, Left) | (East, Forward) => East,
            (North, Forward) | (West, Right) | (East, Left) => North,
            (South, Forward) | (West, Left) | (East, Right) => South,
        }
    }
}

#[derive(Debug)]
enum Action {
    North(isize),
    East(isize),
    South(isize),
    West(isize),
    Left(isize),
    Right(isize),
    Forward(isize),
}

impl std::str::FromStr for Action {
    type Err = std::convert::Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let split = s.split_at(1);
        let value: isize = split
            .1
            .parse()
            .expect("expect at least a value in the action");
        use Action::*;
        let action = match split.0 {
            "N" => North(value),
            "E" => East(value),
            "S" => South(value),
            "W" => West(value),
            "L" => Left(value / 90),
            "R" => Right(value / 90),
            "F" => Forward(value),
            _ => panic!("expect valid action"),
        };
        Ok(action)
    }
}

#[derive(Debug)]
struct Boat {
    position: (isize, isize),
    #[cfg(not(feature = "waypoint"))]
    facing: Cardinal,
    #[cfg(feature = "waypoint")]
    waypoint: (isize, isize),
}

impl std::default::Default for Boat {
    fn default() -> Self {
        Self {
            position: (0, 0),
            #[cfg(not(feature = "waypoint"))]
            facing: Cardinal::East,
            #[cfg(feature = "waypoint")]
            waypoint: (10, 1),
        }
    }
}
impl Boat {
    fn step(mut self, action: Action) -> Self {
        use Action::*;
        #[cfg(not(feature = "waypoint"))]
        match action {
            North(n) => self.position.1 += n,
            East(n) => self.position.0 += n,
            South(n) => self.position.1 -= n,
            West(n) => self.position.0 -= n,
            Left(n) => (0..n).for_each(|_| self.facing = self.facing.turn(Direction::Left)),
            Right(n) => (0..n).for_each(|_| self.facing = self.facing.turn(Direction::Right)),
            Forward(n) => match self.facing {
                Cardinal::North => self = self.step(Action::North(n)),
                Cardinal::East => self = self.step(Action::East(n)),
                Cardinal::South => self = self.step(Action::South(n)),
                Cardinal::West => self = self.step(Action::West(n)),
            },
        }
        #[cfg(feature = "waypoint")]
        match action {
            North(n) => self.waypoint.1 += n,
            East(n) => self.waypoint.0 += n,
            South(n) => self.waypoint.1 -= n,
            West(n) => self.waypoint.0 -= n,
            Left(n) => (0..n).for_each(|_| {
                let x = self.waypoint.0 - self.position.0;
                let y = self.waypoint.1 - self.position.1;
                self.waypoint.0 = self.position.0 - y;
                self.waypoint.1 = self.position.1 + x;
            }),
            Right(n) => (0..n).for_each(|_| {
                let x = self.waypoint.0 - self.position.0;
                let y = self.waypoint.1 - self.position.1;
                self.waypoint.0 = self.position.0 + y;
                self.waypoint.1 = self.position.1 - x;
            }),
            Forward(n) => {
                let x = self.waypoint.0 - self.position.0;
                let y = self.waypoint.1 - self.position.1;
                self.waypoint.0 += x * n;
                self.waypoint.1 += y * n;
                self.position.0 += x * n;
                self.position.1 += y * n;
            }
        }
        self
    }
    fn manhattan(&self) -> isize {
        self.position.0.abs() + self.position.1.abs()
    }
}

#[derive(Debug)]
struct Actions<I>
where
    I: Iterator<Item = &'static str>,
{
    stream: I,
}

impl<I> std::convert::From<I> for Actions<I>
where
    I: Iterator<Item = &'static str>,
{
    fn from(stream: I) -> Self {
        Self { stream }
    }
}

impl<I> std::iter::Iterator for Actions<I>
where
    I: Iterator<Item = &'static str>,
{
    type Item = Action;
    fn next(&mut self) -> Option<Self::Item> {
        self.stream
            .next()
            .map(|action| action.parse().expect("expect to have a valid action"))
    }
}

impl<I> Actions<I>
where
    I: Iterator<Item = &'static str>,
{
    fn execute(self) -> Boat {
        self.fold(Boat::default(), |boat, action| boat.step(action))
    }
}

fn main() {
    let boat = Actions::from(include_str!("../actions.txt").trim().split('\n')).execute();
    println!(
        "The boat moved {} units (Manhattan distance)",
        boat.manhattan()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(not(feature = "waypoint"))]
    #[test]
    fn boat() {
        let boat = Actions::from("F10 N3 F7 R90 F11".split_whitespace()).execute();
        assert_eq!(25, boat.manhattan());
    }

    #[cfg(feature = "waypoint")]
    #[test]
    fn boat() {
        let boat = Actions::from("F10 N3 F7 R90 F11".split_whitespace()).execute();
        assert_eq!(286, boat.manhattan());
    }
}
