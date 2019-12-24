use intcode::{self, Program};
use std::{
    collections::HashMap,
    env,
    fmt::{self, Display, Formatter},
    fs::File,
    io::{BufRead, BufReader},
    ops::{Deref, DerefMut},
    sync::mpsc,
    thread,
};

type Position = (i64, i64);

#[derive(Debug, Clone)]
enum Tile {
    Empty(usize),
    Wall,
    Oxygen(usize),
}

impl From<i64> for Tile {
    fn from(int_tile: i64) -> Self {
        match int_tile {
            0 => Tile::Wall,
            1 => Tile::Empty(999),
            2 => Tile::Oxygen(999),
            _ => Tile::Empty(999),
        }
    }
}

impl Display for Tile {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        use self::Tile::*;
        match self {
            Empty(i) if i == &999 => write!(f, "   "),
            Empty(i) => write!(f, "{:^3}", i),
            Wall => write!(f, "███"),
            Oxygen(_) => write!(f, "oOo"),
        }
    }
}

#[derive(Debug)]
enum Command {
    North,
    South,
    West,
    East,
}

impl Into<i64> for Command {
    fn into(self) -> i64 {
        use self::Command::*;
        match self {
            North => 1,
            South => 2,
            West => 3,
            East => 4,
        }
    }
}

impl Command {
    fn next(&self, position: &Position) -> Position {
        use self::Command::*;
        match self {
            North => (position.0, position.1 + 1),
            South => (position.0, position.1 - 1),
            West => (position.0 - 1, position.1),
            East => (position.0 + 1, position.1),
        }
    }
}

type Tiles = HashMap<Position, Tile>;
#[derive(Debug, Default)]
struct Map {
    tiles: Tiles,
    position: Position,
}
impl Deref for Map {
    type Target = Tiles;
    fn deref(&self) -> &Self::Target {
        &self.tiles
    }
}
impl DerefMut for Map {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.tiles
    }
}

impl Display for Map {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        if self.tiles.is_empty() {
            return Ok(());
        }
        let min_x = self.keys().map(|position| position.0).min().unwrap();
        let max_x = self.keys().map(|position| position.0).max().unwrap();
        let min_y = self.keys().map(|position| position.1).min().unwrap();
        let max_y = self.keys().map(|position| position.1).max().unwrap();
        for y in (min_y..=max_y).rev() {
            for x in min_x..=max_x {
                if self.position.0 == x && self.position.1 == y {
                    write!(f, " x ")?;
                } else {
                    let tile = self.tiles.get(&(x, y)).cloned().unwrap_or(Tile::Empty(999));
                    write!(f, "{}", tile)?;
                }
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}

impl Map {
    fn time_to_oxygenize(&mut self) -> usize {
        let mut minutes = 0;
        let mut oxygen_positions: Vec<Position> = self
            .iter()
            .filter(|(_, tile)| match tile {
                Tile::Oxygen(_) => true,
                _ => false,
            })
            .map(|(position, _)| position.clone())
            .collect();
        loop {
            let mut new_oxygen_positions = Vec::new();
            for oxygen_position in oxygen_positions.drain(..) {
                let neighbour_positions: Vec<Position> =
                    vec![Command::North, Command::South, Command::West, Command::East]
                        .iter()
                        .map(|command| command.next(&oxygen_position))
                        .collect();
                for neighbour_position in neighbour_positions {
                    if let Some(neighbour) = self.get_mut(&neighbour_position) {
                        if let Tile::Empty(distance) = neighbour {
                            *neighbour = Tile::Oxygen(*distance);
                            new_oxygen_positions.push(neighbour_position);
                        }
                    }
                }
            }
            if new_oxygen_positions.is_empty() {
                return minutes;
            }
            oxygen_positions = new_oxygen_positions;
            minutes += 1;
        }
    }
}

fn find_oxygen(opcodes: Vec<i64>) -> Result<Map, intcode::Error> {
    let (sender_to_thread, receiver_from_host) = mpsc::sync_channel(0);
    let (sender_to_host, receiver_from_thread) = mpsc::sync_channel(0);
    let mut program = Program::new(opcodes, receiver_from_host, sender_to_host);
    thread::spawn(move || program.run().unwrap());
    let mut map = Map::default();
    map.position = (0, 0);
    let mut current_distance = 0;
    map.insert((0, 0), Tile::Empty(current_distance));
    'outer: loop {
        println!("{}", map);
        for command in vec![Command::North, Command::South, Command::West, Command::East] {
            std::thread::sleep(std::time::Duration::from_millis(10));
            let next_position = command.next(&map.position);
            if let Some(tile) = map.get(&next_position) {
                if let Tile::Empty(distance) = tile {
                    let diff = if current_distance > *distance {
                        current_distance - *distance
                    } else {
                        *distance - current_distance
                    };
                    if diff < 2 {
                        continue;
                    }
                } else {
                    continue;
                }
            }
            sender_to_thread.send(command.into())?;
            let new_tile = Tile::from(receiver_from_thread.recv()?);
            match new_tile {
                Tile::Empty(_) => {
                    current_distance += 1;
                    map.insert(next_position.clone(), Tile::Empty(current_distance));
                    map.position = next_position;
                    continue 'outer;
                }
                Tile::Wall => {
                    map.insert(next_position, Tile::Wall);
                    continue 'outer;
                }
                Tile::Oxygen(_) => {
                    current_distance += 1;
                    map.insert(next_position.clone(), Tile::Oxygen(current_distance));
                    map.position = next_position;
                    continue 'outer;
                }
            }
        }
        // If all around is explored, move backward
        for command in vec![Command::East, Command::West, Command::South, Command::North] {
            let next_position = command.next(&map.position);
            if let Some(Tile::Empty(distance)) = map.get(&next_position) {
                if *distance <= current_distance {
                    sender_to_thread.send(command.into())?;
                    receiver_from_thread.recv()?;
                    current_distance = *distance;
                    map.position = next_position;
                    if map.position == (0, 0) {
                        return Ok(map);
                    }
                    break;
                }
            }
        }
    }
}

fn main() -> Result<(), intcode::Error> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("Only one argument is accepted, received {:#?}", args);
    }
    let file = File::open(&args[1])?;
    let mut reader = BufReader::new(file);
    let mut program_str = String::new();
    reader.read_line(&mut program_str)?;
    let program: Vec<i64> = program_str
        .trim()
        .split(",")
        .map(str::parse)
        .collect::<Result<_, _>>()
        .map_err(intcode::Error::from)?;
    let mut map = find_oxygen(program)?;
    if let Some(Tile::Oxygen(distance)) = map.values().find(|tile| match tile {
        Tile::Oxygen(_) => true,
        _ => false,
    }) {
        let distance = *distance;
        println!("{}", map);
        let time = map.time_to_oxygenize();
        println!("The distance to oxygen system is {}", distance);
        println!("It took {} minutes to reoxygenize", time);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small() {
        let mut map = Map::default();
        map.insert((0, 0), Tile::Empty(0));
        map.insert((0, 1), Tile::Wall);
        map.insert((1, 0), Tile::Empty(1));
        map.insert((1, 1), Tile::Wall);
        map.insert((1, -1), Tile::Wall);
        map.insert((2, 0), Tile::Wall);
        map.insert((-1, 0), Tile::Wall);
        map.insert((0, -1), Tile::Empty(1));
        map.insert((0, -2), Tile::Wall);
        map.insert((-1, -1), Tile::Oxygen(2));
        map.insert((-2, -1), Tile::Wall);
        map.insert((-1, -2), Tile::Wall);
        assert_eq!(3, map.time_to_oxygenize());
    }

    #[test]
    fn bigger() {
        let mut map = Map::default();
        map.insert((1, 4), Tile::Wall);
        map.insert((2, 4), Tile::Wall);
        map.insert((0, 3), Tile::Wall);
        map.insert((1, 3), Tile::Empty(3));
        map.insert((2, 3), Tile::Empty(4));
        map.insert((3, 3), Tile::Wall);
        map.insert((4, 3), Tile::Wall);
        map.insert((0, 2), Tile::Wall);
        map.insert((1, 2), Tile::Empty(2));
        map.insert((2, 2), Tile::Wall);
        map.insert((3, 2), Tile::Empty(2));
        map.insert((4, 2), Tile::Empty(3));
        map.insert((5, 2), Tile::Wall);
        map.insert((0, 1), Tile::Wall);
        map.insert((1, 1), Tile::Empty(1));
        map.insert((2, 1), Tile::Oxygen(1));
        map.insert((3, 1), Tile::Empty(1));
        map.insert((4, 1), Tile::Wall);
        map.insert((1, 0), Tile::Wall);
        map.insert((2, 0), Tile::Wall);
        map.insert((3, 0), Tile::Wall);
        assert_eq!(4, map.time_to_oxygenize());
    }
}
