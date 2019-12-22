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
    time::Duration,
};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Tile {
    Empty,
    Wall,
    Block,
    Paddle,
    Ball,
}

impl From<i64> for Tile {
    fn from(int_tile: i64) -> Self {
        match int_tile {
            0 => Tile::Empty,
            1 => Tile::Wall,
            2 => Tile::Block,
            3 => Tile::Paddle,
            4 => Tile::Ball,
            _ => Tile::Empty,
        }
    }
}

impl Display for Tile {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        use self::Tile::*;
        match self {
            Empty => write!(f, " "),
            Wall => write!(f, "█"),
            Block => write!(f, "░"),
            Paddle => write!(f, "▂"),
            Ball => write!(f, "●"),
        }
    }
}

type Position = (i64, i64);
type Tiles = HashMap<Position, Tile>;
#[derive(Debug, Default)]
struct Game {
    tiles: Tiles,
    score: i64,
}
impl Deref for Game {
    type Target = Tiles;
    fn deref(&self) -> &Self::Target {
        &self.tiles
    }
}
impl DerefMut for Game {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.tiles
    }
}

impl Display for Game {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        if self.tiles.is_empty() {
            return Ok(());
        }
        let min_x = self.tiles.keys().map(|tile| tile.0).min().unwrap();
        let max_x = self.tiles.keys().map(|tile| tile.0).max().unwrap();
        let min_y = self.tiles.keys().map(|tile| tile.1).min().unwrap();
        let max_y = self.tiles.keys().map(|tile| tile.1).max().unwrap();
        write!(f, "Score - {}\n", self.score)?;
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let tile = self.tiles.get(&(x, y)).cloned().unwrap_or(Tile::Empty);
                write!(f, "{}", tile)?;
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}

fn arcade_cabinet(opcodes: Vec<i64>) -> Result<Game, intcode::Error> {
    #[cfg(not(feature = "free-game"))]
    let (_, receiver_from_host) = mpsc::sync_channel(0);
    #[cfg(feature = "free-game")]
    let (sender_to_thread, receiver_from_host) = mpsc::sync_channel(0);
    let (sender_to_host, receiver_from_thread) = mpsc::sync_channel(0);
    let mut program = Program::new(opcodes, receiver_from_host, sender_to_host);
    thread::spawn(move || program.run().unwrap());
    let mut game = Game::default();
    loop {
        while let (Ok(x), Ok(y)) = (
            receiver_from_thread.recv_timeout(Duration::from_millis(1000 / 50)),
            receiver_from_thread.recv_timeout(Duration::from_millis(1000 / 50)),
        ) {
            let position = (x, y);
            if position == (-1, 0) {
                game.score = receiver_from_thread.recv()?;
            } else {
                let tile = Tile::from(receiver_from_thread.recv()?);
                game.insert(position, tile);
            }
        }
        println!("{}", game);
        #[cfg(not(feature = "free-game"))]
        break;
        #[cfg(feature = "free-game")]
        {
            let ((ball_x, _), _) = game
                .tiles
                .iter()
                .find(|(_, &tile)| tile == Tile::Ball)
                .unwrap();
            let ((paddle_x, _), _) = game
                .tiles
                .iter()
                .find(|(_, &tile)| tile == Tile::Paddle)
                .unwrap();
            let send = if paddle_x < ball_x {
                sender_to_thread.try_send(1)
            } else if paddle_x > ball_x {
                sender_to_thread.try_send(-1)
            } else {
                sender_to_thread.try_send(0)
            };
            match send {
                Ok(_) => continue,
                Err(_) => break,
            }
        }
    }
    Ok(game)
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
    #[cfg(feature = "free-game")]
    let program_str = program_str.replacen("1,", "2,", 1);
    let program: Vec<i64> = program_str
        .trim()
        .split(",")
        .map(str::parse)
        .collect::<Result<_, _>>()
        .map_err(intcode::Error::from)?;
    #[cfg(not(feature = "free-game"))]
    {
        let game = arcade_cabinet(program)?;
        let block_tiles_count = game
            .tiles
            .values()
            .filter(|&tile| *tile == Tile::Block)
            .count();
        println!("Number of block tiles is {}", block_tiles_count);
    }
    #[cfg(feature = "free-game")]
    arcade_cabinet(program)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn process_opcodes() -> Result<(), intcode::Error> {
        let tiles = arcade_cabinet(vec![104, 1, 104, 2, 104, 3, 104, 6, 104, 5, 104, 4, 99])?;
        assert_eq!(
            0,
            tiles.values().filter(|&tile| *tile == Tile::Block).count()
        );
        let tiles = arcade_cabinet(vec![104, 1, 104, 2, 104, 2, 99])?;
        assert_eq!(
            1,
            tiles.values().filter(|&tile| *tile == Tile::Block).count()
        );
        Ok(())
    }
}
