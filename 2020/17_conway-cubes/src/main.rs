#[cfg(not(feature = "hypercube"))]
type Coordinates = (isize, isize, isize);
#[cfg(feature = "hypercube")]
type Coordinates = (isize, isize, isize, isize);
type ConwayCubesInner = std::collections::HashSet<Coordinates>;
#[derive(Default)]
struct ConwayCubes {
    inner: ConwayCubesInner,
}

impl std::ops::Deref for ConwayCubes {
    type Target = ConwayCubesInner;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl std::ops::DerefMut for ConwayCubes {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl std::convert::From<&'static str> for ConwayCubes {
    fn from(map: &'static str) -> Self {
        let mut conway_cubes = Self::default();
        for (line_index, line) in map.lines().enumerate() {
            for (row_index, c) in line.chars().enumerate() {
                match c {
                    '#' => {
                        #[cfg(not(feature = "hypercube"))]
                        conway_cubes.insert((row_index as isize, line_index as isize, 0));
                        #[cfg(feature = "hypercube")]
                        conway_cubes.insert((row_index as isize, line_index as isize, 0, 0));
                    }
                    _ => continue,
                }
            }
        }
        conway_cubes
    }
}

impl std::iter::FromIterator<Coordinates> for ConwayCubes {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = Coordinates>,
    {
        let inner: ConwayCubesInner = iter.into_iter().collect();
        Self { inner }
    }
}

impl std::iter::IntoIterator for ConwayCubes {
    type Item = Coordinates;
    type IntoIter = std::collections::hash_set::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

#[cfg(not(feature = "hypercube"))]
impl std::fmt::Debug for ConwayCubes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        let z_min = self.iter().map(|&(_, _, z)| z).min().unwrap();
        let z_max = self.iter().map(|&(_, _, z)| z).max().unwrap();
        let y_min = self.iter().map(|&(_, y, _)| y).min().unwrap();
        let y_max = self.iter().map(|&(_, y, _)| y).max().unwrap();
        let x_min = self.iter().map(|&(x, _, _)| x).min().unwrap();
        let x_max = self.iter().map(|&(x, _, _)| x).max().unwrap();
        for z in z_min..=z_max {
            writeln!(f, "\n[z={}]", z)?;
            let mut layer: Vec<Coordinates> =
                self.iter().filter(|&&(_, _, v)| z == v).copied().collect();
            layer.sort_by(
                |&(x1, y1, _), &(x2, y2, _)| match y1.partial_cmp(&y2).unwrap() {
                    std::cmp::Ordering::Equal => x1.partial_cmp(&x2).unwrap(),
                    ordering => ordering,
                },
            );
            for y in y_min..=y_max {
                let mut line: Vec<Coordinates> =
                    layer.iter().filter(|&&(_, v, _)| y == v).copied().collect();
                line.sort_by_key(|&(x, _, _)| x);
                for x in x_min..=x_max {
                    if line.contains(&(x, y, z)) {
                        write!(f, "#")?;
                    } else {
                        write!(f, "·")?;
                    }
                }
                writeln!(f)?;
            }
        }
        writeln!(f)
    }
}

impl ConwayCubes {
    fn actives(&self) -> usize {
        self.inner.len()
    }
    fn cycle(self) -> Self {
        fn neighbors(c: Coordinates) -> Vec<Coordinates> {
            let mut neighbors = Vec::new();
            for dx in -1..=1 {
                for dy in -1..=1 {
                    for dz in -1..=1 {
                        #[cfg(not(feature = "hypercube"))]
                        if dx != 0 || dy != 0 || dz != 0 {
                            neighbors.push((c.0 + dx, c.1 + dy, c.2 + dz));
                        }
                        #[cfg(feature = "hypercube")]
                        for dd in -1..=1 {
                            if dx != 0 || dy != 0 || dz != 0 || dd != 0 {
                                neighbors.push((c.0 + dx, c.1 + dy, c.2 + dz, c.3 + dd));
                            }
                        }
                    }
                }
            }
            neighbors
        }
        let next_cubes: ConwayCubes = self
            .iter()
            .copied()
            .chain(self.iter().flat_map(|&coordinates| neighbors(coordinates)))
            .collect();
        let mut output = Self::default();
        for cube in next_cubes {
            let active_neighbors = neighbors(cube)
                .into_iter()
                .filter(|c| self.contains(c))
                .count();
            if (self.contains(&cube) && (active_neighbors == 2 || active_neighbors == 3))
                || (!self.contains(&cube) && active_neighbors == 3)
            {
                output.insert(cube);
            }
        }
        output
    }
}

fn main() {
    let mut conway_cubes = ConwayCubes::from(include_str!("../initial.txt"));
    for _ in 0..6 {
        conway_cubes = conway_cubes.cycle();
    }
    println!("{} active cubes after 6 cycles", conway_cubes.actives());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(not(feature = "hypercube"))]
    #[test]
    fn conway_cubes() {
        let initial = r#".#.
..#
###"#;
        // Cycle 0
        let conway_cubes = ConwayCubes::from(initial);
        assert_eq!(5, conway_cubes.actives());
        assert!(conway_cubes.contains(&(1, 0, 0)));
        assert!(conway_cubes.contains(&(2, 1, 0)));
        assert!(conway_cubes.contains(&(0, 2, 0)));
        assert!(conway_cubes.contains(&(1, 2, 0)));
        assert!(conway_cubes.contains(&(2, 2, 0)));

        // Cycle 1
        let conway_cubes = conway_cubes.cycle();
        assert_eq!(11, conway_cubes.actives());
        assert!(conway_cubes.contains(&(0, 1, -1)));
        assert!(conway_cubes.contains(&(2, 2, -1)));
        assert!(conway_cubes.contains(&(1, 3, -1)));
        assert!(conway_cubes.contains(&(0, 1, 0)));
        assert!(conway_cubes.contains(&(2, 1, 0)));
        assert!(conway_cubes.contains(&(1, 2, 0)));
        assert!(conway_cubes.contains(&(2, 2, 0)));
        assert!(conway_cubes.contains(&(1, 3, 0)));
        assert!(conway_cubes.contains(&(0, 1, 1)));
        assert!(conway_cubes.contains(&(2, 2, 1)));
        assert!(conway_cubes.contains(&(1, 3, 1)));

        // Cycle 2
        let conway_cubes = conway_cubes.cycle();
        assert_eq!(21, conway_cubes.actives());

        // Cycle 3
        let conway_cubes = conway_cubes.cycle();
        assert_eq!(38, conway_cubes.actives());
    }

    #[cfg(feature = "hypercube")]
    #[test]
    fn conway_cubes() {
        let initial = r#".#.
..#
###"#;
        let mut conway_cubes = ConwayCubes::from(initial);
        assert_eq!(5, conway_cubes.actives());
        for _ in 0..6 {
            conway_cubes = conway_cubes.cycle();
        }
        assert_eq!(848, conway_cubes.actives());
    }
}
