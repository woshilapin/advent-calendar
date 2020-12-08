use std::{
    env,
    fs::File,
    io::{BufRead, BufReader},
};

type IsTree = bool;
struct Map {
    width: usize,
    height: usize,
    map: Vec<Vec<IsTree>>,
}

impl Map {
    fn from_lines(lines: impl Iterator<Item = String>) -> Self {
        let mut width = 0;
        let mut height = 0;
        let mut map = Vec::new();
        for line in lines {
            height += 1;
            if line.len() != width {
                if width == 0 {
                    width = line.len();
                } else {
                    panic!("2 lines in the map were not the same length");
                }
            }
            let mut map_line = Vec::new();
            for c in line.chars() {
                let is_tree: IsTree = match c {
                    '.' => false,
                    '#' => true,
                    _ => panic!("expect '.' for free cell and '#' for trees"),
                };
                map_line.push(is_tree);
            }
            map.push(map_line);
        }
        Map { width, height, map }
    }

    fn slide(&self, slope: (usize, usize)) -> MapSlider<'_> {
        MapSlider {
            map: self,
            slope,
            position: (0, 0),
        }
    }
}

struct MapSlider<'map> {
    map: &'map Map,
    slope: (usize, usize),
    position: (usize, usize),
}

impl std::iter::Iterator for MapSlider<'_> {
    type Item = IsTree;
    fn next(&mut self) -> Option<Self::Item> {
        if self.position.1 >= self.map.height {
            return None;
        }
        let item = self.map.map[self.position.1][self.position.0 % self.map.width];
        self.position.0 += self.slope.0;
        self.position.1 += self.slope.1;
        return Some(item);
    }
}

impl MapSlider<'_> {
    fn count_trees(&mut self) -> usize {
        self.filter(|is_tree| *is_tree).count()
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("Only one argument is accepted, received {:#?}", args);
    }
    let file = File::open(&args[1]).expect(format!("expect file '{}' to exist", args[1]).as_str());
    let reader = BufReader::new(file);
    let map = Map::from_lines(
        reader
            .lines()
            .map(|line| line.expect("expect each line to be parseable as a String")),
    );
    let mut product = 1;
    for slope in vec![(1, 1), (3, 1), (5, 1), (7, 1), (1, 2)] {
        let count = map.slide(slope).count_trees();
        println!(
            "Total of encountered trees with slope {:?} is {}",
            slope, count
        );
        product *= count;
    }
    println!("Product of all encountered trees is {}", product);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toboggan_trajectory() {
        let map = r#"..##.......
#...#...#..
.#....#..#.
..#.#...#.#
.#...##..#.
..#.##.....
.#.#.#....#
.#........#
#.##...#...
#...##....#
.#..#...#.#"#;
        let map: Map = Map::from_lines(map.split('\n').map(std::borrow::ToOwned::to_owned));
        assert_eq!(2, map.slide((1, 1)).count_trees());
        assert_eq!(7, map.slide((3, 1)).count_trees());
        assert_eq!(3, map.slide((5, 1)).count_trees());
        assert_eq!(4, map.slide((7, 1)).count_trees());
        assert_eq!(2, map.slide((1, 2)).count_trees());
    }
}
