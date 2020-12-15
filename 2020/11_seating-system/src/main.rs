#[cfg(not(feature = "sight"))]
const OCCUPIED_LIMIT: usize = 4;
#[cfg(feature = "sight")]
const OCCUPIED_LIMIT: usize = 5;

#[derive(PartialEq, Eq, Clone, Copy)]
enum Emplacement {
    Floor,
    Empty,
    Occupied,
}

impl std::fmt::Debug for Emplacement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        use Emplacement::*;
        match self {
            Floor => write!(f, " "),
            Empty => write!(f, "⬯"),
            Occupied => write!(f, "⬮"),
        }
    }
}

impl std::str::FromStr for Emplacement {
    type Err = std::convert::Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Emplacement::*;
        match s {
            "." => Ok(Floor),
            "L" => Ok(Empty),
            "#" => Ok(Occupied),
            c => panic!(
                "expect a valid character ('.', 'L' or '#') for a boat emplacement but got '{}'",
                c
            ),
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
struct Boat {
    emplacements: Vec<Vec<Emplacement>>,
}

impl std::fmt::Debug for Boat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        writeln!(f)?;
        for emplacements_line in &self.emplacements {
            for emplacement in emplacements_line {
                write!(f, "{:?}", emplacement)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl<I> std::convert::From<I> for Boat
where
    I: Iterator<Item = &'static str>,
{
    fn from(stream: I) -> Self {
        let emplacements = stream.fold(Vec::<Vec<Emplacement>>::new(), |mut emplacements, line| {
            let emplacements_line = line
                .chars()
                .map(|cell| {
                    cell.to_string()
                        .parse()
                        .expect("expect line to be parseable as Cell")
                })
                .collect::<Vec<Emplacement>>();
            if emplacements.len() != 0 && emplacements_line.len() != emplacements[0].len() {
                panic!("expect every emplacement's line in the boat to be of the same length");
            }
            emplacements.push(emplacements_line);
            emplacements
        });
        Self { emplacements }
    }
}

enum Direction {
    Up,
    Down,
    None,
}
impl Direction {
    fn from(&self, value: usize) -> usize {
        match self {
            Direction::Up => value + 1,
            Direction::Down => value - 1,
            Direction::None => value,
        }
    }
}
impl Boat {
    fn direction_seat(
        &self,
        position: (usize, usize),
        direction: (Direction, Direction),
    ) -> Option<Emplacement> {
        let height = self.emplacements.len();
        let width = self.emplacements[0].len();
        let x = direction.0.from(position.0);
        let y = direction.1.from(position.1);
        if x == 0 || x > width || y == 0 || y > height {
            None
        } else {
            let emplacement = self.emplacements[y - 1][x - 1];
            #[cfg(not(feature = "sight"))]
            {
                Some(emplacement)
            }
            #[cfg(feature = "sight")]
            if emplacement == Emplacement::Floor {
                {
                    self.direction_seat((x, y), direction)
                }
            } else {
                Some(emplacement)
            }
        }
    }
    fn in_sight(&self, x: usize, y: usize) -> impl Iterator<Item = Emplacement> + '_ {
        use Direction::*;
        vec![
            (Down, Down),
            (Down, None),
            (Down, Up),
            (None, Up),
            (Up, Up),
            (Up, None),
            (Up, Down),
            (None, Down),
        ]
        .into_iter()
        .filter_map(move |direction| self.direction_seat((x, y), direction))
    }
    fn round(&mut self) {
        let height = self.emplacements.len();
        let width = self.emplacements[0].len();
        let mut emplacements = Vec::new();
        for y in 1..=height {
            let mut emplacements_line = Vec::new();
            for x in 1..=width {
                let occupied = self
                    .in_sight(x, y)
                    .filter(|&e| e == Emplacement::Occupied)
                    .count();
                match (self.emplacements[y - 1][x - 1], occupied) {
                    (Emplacement::Floor, _) => emplacements_line.push(Emplacement::Floor),
                    (Emplacement::Empty, 0) => emplacements_line.push(Emplacement::Occupied),
                    (Emplacement::Occupied, o) if o >= OCCUPIED_LIMIT => {
                        emplacements_line.push(Emplacement::Empty)
                    }
                    (e, _) => emplacements_line.push(e),
                }
            }
            emplacements.push(emplacements_line);
        }
        std::mem::swap(&mut self.emplacements, &mut emplacements);
    }
    fn stabilize(&mut self) {
        loop {
            let previous_boat = self.clone();
            self.round();
            if self == &previous_boat {
                return;
            }
        }
    }
    fn occupied(&self) -> usize {
        self.emplacements
            .iter()
            .flatten()
            .filter(|&&emplacement| emplacement == Emplacement::Occupied)
            .count()
    }
}

fn main() {
    let mut boat = Boat::from(include_str!("../boat.txt").trim().split('\n'));
    boat.stabilize();
    println!("There is {} occupied seats", boat.occupied());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(not(feature = "sight"))]
    #[test]
    fn boat() {
        let boat0 = Boat::from(
            r#"L.LL.LL.LL
LLLLLLL.LL
L.L.L..L..
LLLL.LL.LL
L.LL.LL.LL
L.LLLLL.LL
..L.L.....
LLLLLLLLLL
L.LLLLLL.L
L.LLLLL.LL"#
                .split('\n'),
        );
        let boat1 = Boat::from(
            r#"#.##.##.##
#######.##
#.#.#..#..
####.##.##
#.##.##.##
#.#####.##
..#.#.....
##########
#.######.#
#.#####.##"#
                .split('\n'),
        );
        let boat2 = Boat::from(
            r#"#.LL.L#.##
#LLLLLL.L#
L.L.L..L..
#LLL.LL.L#
#.LL.LL.LL
#.LLLL#.##
..L.L.....
#LLLLLLLL#
#.LLLLLL.L
#.#LLLL.##"#
                .split('\n'),
        );
        let boat3 = Boat::from(
            r#"#.##.L#.##
#L###LL.L#
L.#.#..#..
#L##.##.L#
#.##.LL.LL
#.###L#.##
..#.#.....
#L######L#
#.LL###L.L
#.#L###.##"#
                .split('\n'),
        );
        let boat4 = Boat::from(
            r#"#.#L.L#.##
#LLL#LL.L#
L.L.L..#..
#LLL.##.L#
#.LL.LL.LL
#.LL#L#.##
..L.L.....
#L#LLLL#L#
#.LLLLLL.L
#.#L#L#.##"#
                .split('\n'),
        );
        let boat5 = Boat::from(
            r#"#.#L.L#.##
#LLL#LL.L#
L.#.L..#..
#L##.##.L#
#.#L.LL.LL
#.#L#L#.##
..L.L.....
#L#L##L#L#
#.LLLLLL.L
#.#L#L#.##"#
                .split('\n'),
        );
        let mut boat = boat0.clone();
        boat.round();
        assert_eq!(boat1, boat, "different after 1 iteration");
        boat.round();
        assert_eq!(boat2, boat, "different after 2 iterations");
        boat.round();
        assert_eq!(boat3, boat, "different after 3 iterations");
        boat.round();
        assert_eq!(boat4, boat, "different after 4 iterations");
        boat.round();
        assert_eq!(boat5, boat, "different after 5 iterations");
        assert_eq!(37, boat.occupied());
        let mut boat = boat0.clone();
        boat.stabilize();
        assert_eq!(37, boat.occupied());
    }

    #[cfg(feature = "sight")]
    #[test]
    fn boat() {
        let boat0 = Boat::from(
            r#"L.LL.LL.LL
LLLLLLL.LL
L.L.L..L..
LLLL.LL.LL
L.LL.LL.LL
L.LLLLL.LL
..L.L.....
LLLLLLLLLL
L.LLLLLL.L
L.LLLLL.LL"#
                .split('\n'),
        );
        let boat1 = Boat::from(
            r#"#.##.##.##
#######.##
#.#.#..#..
####.##.##
#.##.##.##
#.#####.##
..#.#.....
##########
#.######.#
#.#####.##"#
                .split('\n'),
        );
        let boat2 = Boat::from(
            r#"#.LL.LL.L#
#LLLLLL.LL
L.L.L..L..
LLLL.LL.LL
L.LL.LL.LL
L.LLLLL.LL
..L.L.....
LLLLLLLLL#
#.LLLLLL.L
#.LLLLL.L#"#
                .split('\n'),
        );
        let boat3 = Boat::from(
            r#"#.L#.##.L#
#L#####.LL
L.#.#..#..
##L#.##.##
#.##.#L.##
#.#####.#L
..#.#.....
LLL####LL#
#.L#####.L
#.L####.L#"#
                .split('\n'),
        );
        let boat4 = Boat::from(
            r#"#.L#.L#.L#
#LLLLLL.LL
L.L.L..#..
##LL.LL.L#
L.LL.LL.L#
#.LLLLL.LL
..L.L.....
LLLLLLLLL#
#.LLLLL#.L
#.L#LL#.L#"#
                .split('\n'),
        );
        let boat5 = Boat::from(
            r#"#.L#.L#.L#
#LLLLLL.LL
L.L.L..#..
##L#.#L.L#
L.L#.#L.L#
#.L####.LL
..#.#.....
LLL###LLL#
#.LLLLL#.L
#.L#LL#.L#"#
                .split('\n'),
        );
        let boat6 = Boat::from(
            r#"#.L#.L#.L#
#LLLLLL.LL
L.L.L..#..
##L#.#L.L#
L.L#.LL.L#
#.LLLL#.LL
..#.L.....
LLL###LLL#
#.LLLLL#.L
#.L#LL#.L#"#
                .split('\n'),
        );
        let mut boat = boat0.clone();
        boat.round();
        assert_eq!(boat1, boat, "different after 1 iteration");
        boat.round();
        assert_eq!(boat2, boat, "different after 2 iterations");
        boat.round();
        assert_eq!(boat3, boat, "different after 3 iterations");
        boat.round();
        assert_eq!(boat4, boat, "different after 4 iterations");
        boat.round();
        assert_eq!(boat5, boat, "different after 5 iterations");
        boat.round();
        assert_eq!(boat6, boat, "different after 6 iterations");
        assert_eq!(26, boat.occupied());
        let mut boat = boat0.clone();
        boat.stabilize();
        assert_eq!(26, boat.occupied());
    }
}
