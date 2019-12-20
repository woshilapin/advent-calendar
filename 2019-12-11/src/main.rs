use intcode::{self, Program};
use std::{
    collections::HashMap,
    env,
    fmt::{self, Display, Formatter},
    fs::File,
    io::{BufRead, BufReader},
    ops::Add,
    sync::mpsc,
    thread,
};

#[derive(Debug, Clone)]
enum Color {
    Black,
    White,
}

impl From<i64> for Color {
    fn from(int_color: i64) -> Self {
        match int_color {
            0 => Color::Black,
            1 => Color::White,
            _ => Color::Black,
        }
    }
}

impl Into<i64> for &mut Color {
    fn into(self) -> i64 {
        match self {
            Color::Black => 0,
            Color::White => 1,
        }
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        use self::Color::*;
        match self {
            Black => write!(f, " "),
            White => write!(f, "█"),
        }
    }
}

#[derive(Debug)]
enum Turn {
    Left,
    Right,
}

impl From<i64> for Turn {
    fn from(int_dir: i64) -> Self {
        match int_dir {
            0 => Turn::Left,
            1 => Turn::Right,
            dir => panic!("Unknown direction {}", dir),
        }
    }
}

#[derive(Debug)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Add<Turn> for Direction {
    type Output = Self;
    fn add(self, rhs: Turn) -> Self::Output {
        use self::Direction::*;
        match (self, rhs) {
            (Up, Turn::Left) => Left,
            (Up, Turn::Right) => Right,
            (Left, Turn::Left) => Down,
            (Left, Turn::Right) => Up,
            (Down, Turn::Left) => Right,
            (Down, Turn::Right) => Left,
            (Right, Turn::Left) => Up,
            (Right, Turn::Right) => Down,
        }
    }
}

impl Direction {
    fn move_forward(&self, position: Position) -> Position {
        use self::Direction::*;
        match self {
            Up => (position.0, position.1 + 1),
            Left => (position.0 - 1, position.1),
            Down => (position.0, position.1 - 1),
            Right => (position.0 + 1, position.1),
        }
    }
}
type Position = (i64, i64);
type Tiles = HashMap<Position, Color>;

fn painting_robot(opcodes: Vec<i64>) -> Result<Tiles, intcode::Error> {
    let (sender_to_thread, receiver_from_host) = mpsc::sync_channel(0);
    let (sender_to_host, receiver_from_thread) = mpsc::sync_channel(0);
    let mut program = Program::new(opcodes, receiver_from_host, sender_to_host);
    thread::spawn(move || program.run().unwrap());
    let mut tiles = Tiles::new();
    #[cfg(feature = "start-white")]
    tiles.insert((0, 0), Color::White);
    let mut position = (0, 0);
    let mut direction = Direction::Up;
    loop {
        let color = tiles.entry(position.clone()).or_insert(Color::Black);
        if let Err(_) = sender_to_thread.send(color.into()) {
            break;
        }
        *color = Color::from(receiver_from_thread.recv()?);
        direction = direction + Turn::from(receiver_from_thread.recv()?);
        position = direction.move_forward(position);
    }
    Ok(tiles)
}

fn print_tiles(tiles: &Tiles) {
    if tiles.is_empty() {
        return;
    }
    let min_x = tiles.keys().map(|tile| tile.0).min().unwrap();
    let max_x = tiles.keys().map(|tile| tile.0).max().unwrap();
    let min_y = tiles.keys().map(|tile| tile.1).min().unwrap();
    let max_y = tiles.keys().map(|tile| tile.1).max().unwrap();
    for y in (min_y..=max_y).rev() {
        for x in min_x..=max_x {
            let color = tiles.get(&(x, y)).cloned().unwrap_or(Color::Black);
            print!("{}", color);
        }
        println!();
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
    let tiles = painting_robot(program)?;
    print_tiles(&tiles);
    println!("Number of painted tiles is {}", tiles.len());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn process_opcodes() -> Result<(), intcode::Error> {
        let tiles = painting_robot(vec![
            103, 0, 104, 1, 104, 0, 103, 0, 104, 0, 104, 0, 103, 0, 104, 1, 104, 0, 103, 0, 104, 1,
            104, 0, 103, 1, 104, 0, 104, 1, 103, 0, 104, 1, 104, 0, 103, 0, 104, 1, 104, 0, 99,
        ])?;
        assert_eq!(7, tiles.len());
        Ok(())
    }
}
