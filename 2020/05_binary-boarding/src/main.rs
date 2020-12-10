use std::{
    convert::Infallible,
    env,
    fs::File,
    io::{BufRead, BufReader},
};

#[derive(Debug)]
enum RowMove {
    Front,
    Back,
}
impl RowMove {
    fn from_char(c: char) -> Self {
        match c {
            'F' => RowMove::Front,
            'B' => RowMove::Back,
            _ => panic!("expect 'F' or 'B' for row move"),
        }
    }
}

#[derive(Debug)]
enum ColumnMove {
    Right,
    Left,
}
impl ColumnMove {
    fn from_char(c: char) -> Self {
        match c {
            'R' => ColumnMove::Right,
            'L' => ColumnMove::Left,
            _ => panic!("expect 'R' or 'L' for column move"),
        }
    }
}

#[derive(Debug)]
struct BoardingPass {
    row: usize,
    column: usize,
}

impl BoardingPass {
    fn id(&self) -> usize {
        self.row * 8 + self.column
    }
}

impl std::str::FromStr for BoardingPass {
    type Err = Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 10 {
            panic!("expect boarding pass code to be 10 characters long");
        }
        let mut row_min = 0;
        let mut row_max = 127;
        let mut window = 64;
        let chars: Vec<char> = s.chars().collect();
        for c in &chars[0..7] {
            let row_move = RowMove::from_char(*c);
            use RowMove::*;
            match row_move {
                Front => row_max -= window,
                Back => row_min += window,
            }
            window >>= 1;
        }
        debug_assert_eq!(row_min, row_max);
        let row = row_min;
        let mut column_min = 0;
        let mut column_max = 7;
        let mut window = 4;
        for c in &chars[7..10] {
            let column_move = ColumnMove::from_char(*c);
            use ColumnMove::*;
            match column_move {
                Left => column_max -= window,
                Right => column_min += window,
            }
            window >>= 1;
        }
        debug_assert_eq!(column_min, column_max);
        let column = column_min;
        let boarding_pass = BoardingPass { row, column };
        Ok(boarding_pass)
    }
}

struct BoardingPasses<I>
where
    I: Iterator<Item = String>,
{
    stream: I,
}

impl<I> BoardingPasses<I>
where
    I: Iterator<Item = String>,
{
    fn new(stream: I) -> Self {
        Self { stream }
    }
}

impl<I> Iterator for BoardingPasses<I>
where
    I: Iterator<Item = String>,
{
    type Item = BoardingPass;
    fn next(&mut self) -> Option<Self::Item> {
        self.stream
            .next()
            .map(|code| code.parse())
            .transpose()
            .expect("expect String to be parseable as BoardingPass")
    }
}

fn find_seat(mut ids: Vec<usize>) -> usize {
    ids.sort();
    for (id, next_id) in ids.iter().take(ids.len() - 1).zip(ids.iter().skip(1)) {
        if next_id - id != 1 {
            return id + 1;
        }
    }
    panic!("failed to find a seat")
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("Only one argument is accepted, received {:#?}", args);
    }
    let file = File::open(&args[1]).expect(format!("expect file '{}' to exist", args[1]).as_str());
    let reader = BufReader::new(file);
    let boarding_passes = BoardingPasses::new(
        reader
            .lines()
            .map(|line| line.expect("expect line to be parseable as a String")),
    );
    let ids = boarding_passes
        .map(|boarding_pass: BoardingPass| boarding_pass.id())
        .collect::<Vec<_>>();
    println!("Greater boarding pass ID is {}", ids.iter().max().unwrap());
    println!("Your ID seat is {}", find_seat(ids));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boarding_pass() {
        let boarding_pass: BoardingPass = "FBFBBFFRLR".parse().unwrap();
        assert_eq!(357, boarding_pass.id());
    }

    #[test]
    fn boarding_passes() {
        let boarding_passes = r#"BFFFBBFRRR
FFFBBBFRRR
BBFFBBFRLL"#;
        let mut boarding_passes = BoardingPasses::new(
            boarding_passes
                .split('\n')
                .map(std::borrow::ToOwned::to_owned),
        );
        let boarding_pass = boarding_passes.next().unwrap();
        assert_eq!(567, boarding_pass.id());
        let boarding_pass = boarding_passes.next().unwrap();
        assert_eq!(119, boarding_pass.id());
        let boarding_pass = boarding_passes.next().unwrap();
        assert_eq!(820, boarding_pass.id());
        assert!(boarding_passes.next().is_none());
    }

    #[test]
    fn find_seat() {
        let ids = vec![8, 4, 5, 7];
        assert_eq!(6, super::find_seat(ids));
    }
}
