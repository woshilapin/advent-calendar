use std::{
    cmp::{Ord, Ordering, PartialOrd},
    collections::{BTreeSet, HashSet},
    env,
    fmt::{Display, Error, Formatter},
    fs::File,
    io::{BufRead, BufReader},
    iter::{FromIterator, IntoIterator},
};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Asteroid {
    x: usize,
    y: usize,
}

impl From<(usize, usize)> for Asteroid {
    fn from((x, y): (usize, usize)) -> Self {
        Asteroid { x, y }
    }
}

impl Into<(usize, usize)> for Asteroid {
    fn into(self) -> (usize, usize) {
        (self.x, self.y)
    }
}

#[derive(Debug, PartialEq, Eq)]
struct LaserRay<'a> {
    from: &'a Asteroid,
    to: &'a Asteroid,
}

impl LaserRay<'_> {
    fn vector(&self) -> (i64, i64) {
        (
            self.to.x as i64 - self.from.x as i64,
            self.to.y as i64 - self.from.y as i64,
        )
    }
}

impl<'a> From<(&'a Asteroid, &'a Asteroid)> for LaserRay<'a> {
    fn from((from, to): (&'a Asteroid, &'a Asteroid)) -> Self {
        LaserRay { from, to }
    }
}

impl PartialOrd for LaserRay<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for LaserRay<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        let self_v = self.vector();
        let other_v = other.vector();
        if self_v.0 >= 0 {
            if other_v.0 < 0 {
                return Ordering::Less;
            }
            if self_v.1 <= 0 {
                if other_v.1 > 0 {
                    return Ordering::Less;
                }
            }
        } else {
            if other_v.0 >= 0 {
                return Ordering::Greater;
            }
            if self_v.1 >= 0 {
                if other_v.1 < 0 {
                    return Ordering::Less;
                }
            }
        }
        let dydx = -self_v.1 * other_v.0;
        let dydx2 = -other_v.1 * self_v.0;
        if dydx > dydx2 {
            Ordering::Less
        } else if dydx < dydx2 {
            Ordering::Greater
        } else {
            let d = self_v.0 * self_v.0 + self_v.1 * self_v.1;
            let d2 = other_v.0 * other_v.0 + other_v.1 * other_v.1;
            if d < d2 {
                Ordering::Less
            } else if d > d2 {
                Ordering::Greater
            } else {
                Ordering::Equal
            }
        }
    }
}

#[derive(Debug, Default)]
struct Map {
    asteroids: HashSet<Asteroid>,
    width: usize,
    height: usize,
    monitoring_station: Option<Asteroid>,
}

impl From<String> for Map {
    fn from(string_map: String) -> Self {
        let mut map = Map::default();
        for (y, line) in string_map.lines().enumerate() {
            map.height = y + 1;
            for (x, cell) in line.chars().enumerate() {
                map.width = x + 1;
                match cell {
                    '#' => {
                        map.asteroids.insert(Asteroid { x, y });
                    }
                    _ => continue,
                }
            }
        }
        map
    }
}

impl FromIterator<Asteroid> for Map {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = Asteroid>,
    {
        let asteroids: HashSet<Asteroid> = iter.into_iter().collect();
        let mut width = 0;
        let mut height = 0;
        for asteroid in &asteroids {
            if asteroid.x > width {
                width = asteroid.x;
            }
            if asteroid.y > height {
                height = asteroid.y;
            }
        }
        Map {
            asteroids,
            width,
            height,
            monitoring_station: None,
        }
    }
}

impl Display for Map {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        let mut asteroids = Vec::from_iter(self.asteroids.clone());
        asteroids.sort_by_key(|asteroid| asteroid.x);
        asteroids.sort_by_key(|asteroid| asteroid.y);
        let mut asteroid_index = 0;
        write!(f, "+")?;
        for _ in 0..self.width {
            write!(f, "-")?;
        }
        write!(f, "+\n")?;
        for y in 0..self.height {
            write!(f, "|")?;
            for x in 0..self.width {
                let c = asteroids
                    .get(asteroid_index)
                    .and_then(|asteroid| {
                        if asteroid.x == x && asteroid.y == y {
                            asteroid_index += 1;
                            if self
                                .monitoring_station
                                .as_ref()
                                .map(|ms| ms.x == asteroid.x && ms.y == asteroid.y)
                                .unwrap_or(false)
                            {
                                Some('●')
                            } else {
                                Some('○')
                            }
                        } else {
                            None
                        }
                    })
                    .unwrap_or(' ');
                write!(f, "{}", c)?;
            }
            write!(f, "|\n")?;
        }
        write!(f, "+")?;
        for _ in 0..self.width {
            write!(f, "-")?;
        }
        write!(f, "+\n")?;
        Ok(())
    }
}

// Based on https://stackoverflow.com/a/11908158/7447059
fn is_asteroid_between((origin, destination): (&Asteroid, &Asteroid), asteroid: &Asteroid) -> bool {
    if origin == destination || asteroid == origin || asteroid == destination {
        return false;
    }
    type Vector = (i64, i64);
    let vector = |a: &Asteroid, b: &Asteroid| (b.x as i64 - a.x as i64, b.y as i64 - a.y as i64);
    let cross_product = |v1: Vector, v2: Vector| v1.0 * v2.1 - v1.1 * v2.0;
    let from_origin = vector(origin, asteroid);
    let origin_to_destination = vector(origin, destination);
    let is_colinear = cross_product(from_origin, origin_to_destination) == 0;
    if !is_colinear {
        return false;
    }
    if origin_to_destination.0.abs() >= origin_to_destination.1.abs() {
        if origin_to_destination.0 > 0 {
            origin.x <= asteroid.x && asteroid.x <= destination.x
        } else {
            destination.x <= asteroid.x && asteroid.x <= origin.x
        }
    } else {
        if origin_to_destination.1 > 0 {
            origin.y <= asteroid.y && asteroid.y <= destination.y
        } else {
            destination.y <= asteroid.y && asteroid.y <= origin.y
        }
    }
}

fn visible_asteroids<'a>(origin: &Asteroid, map: &'a Map) -> HashSet<&'a Asteroid> {
    let mut obstructed_asteroids = HashSet::new();
    for asteroid_to_check in &map.asteroids {
        for asteroid in &map.asteroids {
            let is_obstructed = is_asteroid_between((origin, asteroid_to_check), asteroid);
            if is_obstructed {
                obstructed_asteroids.insert(asteroid_to_check);
            }
        }
    }
    let visible_asteroids = map
        .asteroids
        .iter()
        .filter(|&asteroid| asteroid != origin)
        .filter(|&asteroid| !obstructed_asteroids.contains(asteroid))
        .collect::<HashSet<_>>();
    visible_asteroids
}

impl Map {
    fn find_monitoring_station(&mut self) {
        let mut max_visible = 0;
        for asteroid in &self.asteroids {
            let visible_count = visible_asteroids(asteroid, &self).len();
            if visible_count > max_visible {
                self.monitoring_station = Some(asteroid.clone());
                max_visible = visible_count;
            }
        }
    }

    fn destroy_asteroids<'a>(&'a self) -> Vec<&'a Asteroid> {
        let mut destroyed_asteroids = Vec::new();
        let mut next_round = HashSet::new();
        if let Some(monitoring_station) = &self.monitoring_station {
            let laser_rays: BTreeSet<LaserRay> = self
                .asteroids
                .iter()
                .filter(|asteroid| asteroid != &monitoring_station)
                .map(|asteroid| LaserRay::from((monitoring_station, asteroid)))
                .collect();
            let mut last_ray = None;
            for laser_ray in &laser_rays {
                let vec = laser_ray.vector();
                let last_vec = last_ray.map(LaserRay::vector);
                let dydx = last_vec.map(|v| -vec.1 * v.0);
                let last_dydx = last_vec.map(|v| -v.1 * vec.0);
                match (dydx, last_dydx) {
                    (Some(dd), Some(last_dd)) if dd == last_dd => {
                        next_round.insert(laser_ray.to.clone());
                    }
                    _ => {
                        destroyed_asteroids.push(laser_ray.to);
                    }
                }
                last_ray = Some(laser_ray);
            }
        }
        if !next_round.is_empty() {
            let map = Map {
                asteroids: next_round,
                width: self.width,
                height: self.height,
                monitoring_station: self.monitoring_station.clone(),
            };
            let next_round_destroyed = map.destroy_asteroids();
            for next_round_asteroid in next_round_destroyed {
                for asteroid in &self.asteroids {
                    if next_round_asteroid == asteroid {
                        destroyed_asteroids.push(asteroid);
                    }
                }
            }
        }
        destroyed_asteroids
    }
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("One file argument is needed, received {:#?}", args);
    }
    let file = File::open(&args[1])?;
    let reader = BufReader::new(file);
    let mut buffer = String::new();
    for line in reader.lines() {
        buffer = format!("{}\n{}", buffer.trim(), line?);
    }
    let mut map = Map::from(buffer);
    map.find_monitoring_station();
    println!("{}", map);
    if let Some(monitoring_station) = &map.monitoring_station {
        let visible_asteroids = visible_asteroids(&monitoring_station, &map);
        let max_visible_asteroids = visible_asteroids.len();
        println!(
            "Maximum number of visible asteroids is {}",
            max_visible_asteroids
        );
        let destroyed_asteroids = map.destroy_asteroids();
        if destroyed_asteroids.len() >= 200 {
            let asteroid = destroyed_asteroids[199];
            println!(
                "200th asteroid has coordinates ({},{}) [{}]",
                asteroid.x,
                asteroid.y,
                100 * asteroid.x + asteroid.y
            );
        } else {
            println!("Less than 200 asteroids have been destroyed");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn calculate_visible_asteroids() {
        let map = vec![(0, 0), (1, 1), (2, 2), (2, 1)]
            .into_iter()
            .map(Asteroid::from)
            .collect();
        assert_eq!(2, visible_asteroids(&Asteroid { x: 0, y: 0 }, &map).len());

        assert_eq!((1i64 - 1).signum(), ((-1i64) - (-1)).signum());
    }

    #[test]
    fn calculate_best_asteroid() {
        let mut map: Map = vec![(0, 0), (0, 1), (1, 1), (2, 2), (2, 1)]
            .into_iter()
            .map(Asteroid::from)
            .collect();
        map.find_monitoring_station();
        assert_eq!((1, 1), map.monitoring_station.unwrap().into());
        let mut map = Map::from(
            r#".#..#
.....
#####
....#
...##"#
                .to_string(),
        );
        map.find_monitoring_station();
        assert_eq!((3, 4), map.monitoring_station.unwrap().into());

        let mut map = Map::from(
            r#"......#.#.
#..#.#....
..#######.
.#.#.###..
.#..#.....
..#....#.#
#..#....#.
.##.#..###
##...#..#.
.#....####"#
                .to_string(),
        );
        map.find_monitoring_station();
        assert_eq!((5, 8), map.monitoring_station.unwrap().into());

        let mut map = Map::from(
            r#"#.#...#.#.
.###....#.
.#....#...
##.#.#.#.#
....#.#.#.
.##..###.#
..#...##..
..##....##
......#...
.####.###."#
                .to_string(),
        );
        map.find_monitoring_station();
        assert_eq!((1, 2), map.monitoring_station.unwrap().into());

        let mut map = Map::from(
            r#".#..#..###
####.###.#
....###.#.
..###.##.#
##.##.#.#.
....###..#
..#.#..#.#
#..#.#.###
.##...##.#
.....#.#.."#
                .to_string(),
        );
        map.find_monitoring_station();
        assert_eq!((6, 3), map.monitoring_station.unwrap().into());

        let mut map = Map::from(
            r#"#..##.###...#######
##.############..##.
.#.######.########.#
.###.#######.####.#.
#####.##.#.##.###.##
..#####..#.#########
####################
#.####....###.#.#.##
##.#################
#####.##.###..####..
..######..##.#######
####.##.####...##..#
.#####..#.######.###
##...#.##########...
#.##########.#######
.####.#.###.###.#.##
....##.##.###..#####
.#.#.###########.###
#.#.#.#####.####.###
###.##.####.##.#..##"#
                .to_string(),
        );
        map.find_monitoring_station();
        assert_eq!((11, 13), map.monitoring_station.unwrap().into());
    }

    mod laser_ray_ordering {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn equal() {
            let monitoring_station = Asteroid::from((0, 0));
            let asteroid = Asteroid::from((1, 1));
            assert_eq!(
                Ordering::Equal,
                LaserRay::from((&monitoring_station, &asteroid))
                    .cmp(&LaserRay::from((&monitoring_station, &asteroid)))
            );
        }

        #[test]
        fn order() {
            let monitoring_station = Asteroid::from((2, 2));
            let asteroids: Vec<Asteroid> = vec![
                (2, 1),
                (3, 0),
                (3, 1),
                (4, 1),
                (3, 2),
                (4, 3),
                (3, 3),
                (3, 4),
                (2, 3),
                (1, 4),
                (1, 3),
                (0, 3),
                (1, 2),
                (0, 1),
                (1, 1),
                (1, 0),
            ]
            .into_iter()
            .map(Asteroid::from)
            .collect();
            let laser_rays: Vec<LaserRay> = asteroids
                .iter()
                .map(|asteroid| LaserRay::from((&monitoring_station, asteroid)))
                .collect();
            for i in 0..(laser_rays.len() - 1) {
                let lr1 = &laser_rays[i];
                let lr2 = &laser_rays[i + 1];
                assert_eq!(Ordering::Less, lr1.cmp(lr2));
                assert_eq!(Ordering::Greater, lr2.cmp(lr1));
            }
        }

        #[test]
        fn limit() {
            let monitoring_station = Asteroid::from((2, 2));
            let asteroid1 = Asteroid::from((2, 1));
            let asteroid2 = Asteroid::from((1, 0));
            assert_eq!(
                Ordering::Less,
                LaserRay::from((&monitoring_station, &asteroid1))
                    .cmp(&LaserRay::from((&monitoring_station, &asteroid2)))
            );
        }

        #[test]
        fn distance() {
            let monitoring_station = Asteroid::from((0, 0));
            let asteroid1 = Asteroid::from((1, 1));
            let asteroid2 = Asteroid::from((2, 2));
            assert_eq!(
                Ordering::Less,
                LaserRay::from((&monitoring_station, &asteroid1))
                    .cmp(&LaserRay::from((&monitoring_station, &asteroid2)))
            );
        }
    }

    #[test]
    fn laser_destroy() {
        let mut map = Map::from(
            r#".#....#####...#..
##...##.#####..##
##...#...#.#####.
..#.....#...###..
..#.#.....#....##"#
                .to_string(),
        );
        map.find_monitoring_station();
        let destroyed_asteroids = map.destroy_asteroids();
        let expected: Vec<Asteroid> = vec![
            (8, 1),
            (9, 0),
            (9, 1),
            (10, 0),
            (9, 2),
            (11, 1),
            (12, 1),
            (11, 2),
            (15, 1),
            (12, 2),
            (13, 2),
            (14, 2),
            (15, 2),
            (12, 3),
            (16, 4),
            (15, 4),
            (10, 4),
            (4, 4),
            (2, 4),
            (2, 3),
            (0, 2),
            (1, 2),
            (0, 1),
            (1, 1),
            (5, 2),
            (1, 0),
            (5, 1),
            (6, 1),
            (6, 0),
            (7, 0),
            (8, 0),
            (10, 1),
            (14, 0),
            (16, 1),
            (13, 3),
            (14, 3),
        ]
        .into_iter()
        .map(Asteroid::from)
        .collect();
        let expected_ref: Vec<&Asteroid> = expected.iter().collect();
        assert_eq!(expected_ref, destroyed_asteroids);
    }
}
