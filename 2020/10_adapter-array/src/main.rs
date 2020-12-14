type Adapter = usize;

#[derive(Debug)]
struct Adapters {
    adapters: Vec<Adapter>,
}

impl<I> std::convert::From<I> for Adapters
where
    I: Iterator<Item = &'static str>,
{
    fn from(stream: I) -> Self {
        let mut adapters: Vec<Adapter> = stream
            .map(|adapter| {
                adapter
                    .parse()
                    .expect("expect adapters to be parseable as Adapter")
            })
            .collect();
        adapters.sort();
        // Add the outlet charger
        adapters.insert(0, 0);
        // Add the final in-device adapter
        adapters.push(adapters.last().expect("expect at least 1 adapter") + 3);
        Self { adapters }
    }
}

impl Adapters {
    fn differences(&self) -> (usize, usize, usize) {
        let mut one = 0;
        let mut two = 0;
        let mut three = 0;
        for window in self.adapters.windows(2) {
            match window[1] - window[0] {
                0 => panic!("expect to have different adapters"),
                1 => one += 1,
                2 => two += 1,
                3 => three += 1,
                _ => panic!("expect adapters to complete the chain"),
            }
        }
        (one, two, three)
    }

    // Every sequence of successive adapters (1-2-3 or 41-42-43), the middle one
    // can be removed or not (2 possibilities). For each group of these
    // successive numbers, you can multiply by 2 the number of
    // possibilities... but there is a but.
    //
    // If you have (1-2-3-4-5), each of 2, 3, and 4 are toggeable... but
    // actually, 1-5 is not valid. So when you have more than 2 successive
    // numbers, you need to remove one extra combination from the current
    // successive sequence.
    fn arrangements(&self) -> usize {
        let mut arrangements = 1;
        let mut successive = 1;
        let mut extra = 0;
        for window in self.adapters.windows(3) {
            if window[1] - window[0] == 1 && window[2] - window[1] == 1 {
                successive *= 2;
                if successive >= 8 {
                    extra += 1;
                }
            } else {
                arrangements *= successive - extra;
                successive = 1;
                extra = 0;
            }
        }
        arrangements
    }
}

fn main() {
    let adapters = Adapters::from(include_str!("../adapters.txt").trim().split('\n'));
    let (one, two, three) = adapters.differences();
    println!(
        "There is respectively {}, {} and {} 1-2-3 differences (1 * 3 -> {})",
        one,
        two,
        three,
        one * three
    );
    println!(
        "There is {} possible adapters arrangements",
        adapters.arrangements()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn few_adapters() {
        let adapters = Adapters::from(r#"16 10 15 5 1 11 7 19 6 12 4"#.split_whitespace());
        let (one, two, three) = adapters.differences();
        assert_eq!(7, one);
        assert_eq!(0, two);
        assert_eq!(5, three);
        assert_eq!(8, adapters.arrangements());
    }

    #[test]
    fn more_adapters() {
        let adapters = Adapters::from(r#"28 33 18 42 31 14 46 20 48 47 24 23 49 45 19 38 39 11 1 32 25 35 8 17 7 9 4 2 34 10 3"#.split_whitespace());
        let (one, two, three) = adapters.differences();
        assert_eq!(22, one);
        assert_eq!(0, two);
        assert_eq!(10, three);
        assert_eq!(19208, adapters.arrangements());
    }

    #[test]
    fn successive_adapters() {
        let adapters = Adapters::from(r#"1 2 3 4"#.split_whitespace());
        // (0) -> 1 -> 2 -> 3 -> 4 -> (7)
        // (0) -> 2 -> 3 -> 4 -> (7)
        // (0) -> 2 -> 4 -> (7)
        // (0) -> 1 -> 3 -> 4 -> (7)
        // (0) -> 3 -> 4 -> (7)
        // (0) -> 1 -> 4 -> (7)
        // (0) -> 1 -> 2 -> 4 -> (7)
        let (one, two, three) = adapters.differences();
        assert_eq!(4, one);
        assert_eq!(0, two);
        assert_eq!(1, three);
        assert_eq!(7, adapters.arrangements());
    }
}
