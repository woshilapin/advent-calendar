use std::collections::BTreeMap;

#[derive(Debug)]
struct Recitation<I>
where
    I: Iterator<Item = &'static str>,
{
    iter: I,
    turn: usize,
    last_number: Option<usize>,
    cache: BTreeMap<usize, usize>,
}

impl<I> std::convert::From<I> for Recitation<I>
where
    I: Iterator<Item = &'static str>,
{
    fn from(iter: I) -> Self {
        Self {
            iter,
            turn: 1,
            last_number: None,
            cache: BTreeMap::new(),
        }
    }
}

impl<I> std::iter::Iterator for Recitation<I>
where
    I: Iterator<Item = &'static str>,
{
    type Item = usize;
    fn next(&mut self) -> Option<Self::Item> {
        let number = if let Some(starting_number) = self.iter.next() {
            starting_number
                .parse()
                .expect("expect starting number to be parseable as an 'usize'")
        } else {
            if let Some(last_number) = self.last_number {
                if let Some(&spoken_turn) = self.cache.get(&last_number) {
                    self.turn - spoken_turn - 1
                } else {
                    0
                }
            } else {
                unreachable!(
                    "expect at least one starting number so there should always be a last_number"
                );
            }
        };
        if let Some(last_number) = self.last_number {
            self.cache.insert(last_number, self.turn - 1);
        }
        self.turn += 1;
        self.last_number = Some(number);
        Some(number)
    }
}

fn main() {
    let starting_numbers = "8,11,0,19,1,2";
    let years_number = Recitation::from(starting_numbers.split(','))
        .skip(2020 - 1)
        .next()
        .expect("expect a 2020th number");
    println!("2020th number is {}", years_number);
    // Rust is fast enough, let it run for a few seconds/minutes and it'll work!
    let big_number = Recitation::from(starting_numbers.split(','))
        .skip(30000000 - 1)
        .next()
        .expect("expect a 30000000th number");
    println!("30000000th number is {}", big_number);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rambunctious_recitation_2020() {
        let starting_numbers = "0,3,6";
        let mut recitation = Recitation::from(starting_numbers.split(','));
        assert_eq!(0, recitation.next().unwrap());
        assert_eq!(3, recitation.next().unwrap());
        assert_eq!(6, recitation.next().unwrap());
        assert_eq!(0, recitation.next().unwrap());
        assert_eq!(3, recitation.next().unwrap());
        assert_eq!(3, recitation.next().unwrap());
        assert_eq!(1, recitation.next().unwrap());
        assert_eq!(0, recitation.next().unwrap());
        assert_eq!(4, recitation.next().unwrap());
        assert_eq!(0, recitation.next().unwrap());
        let years_number = Recitation::from(starting_numbers.split(','))
            .skip(2019)
            .next()
            .unwrap();
        assert_eq!(436, years_number);
    }

    // This test run for several minutes but it works
    #[test]
    fn rambunctious_recitation_30000000() {
        assert_eq!(
            175594,
            Recitation::from("0,3,6".split(','))
                .skip(30000000 - 1)
                .next()
                .unwrap()
        );
        assert_eq!(
            2578,
            Recitation::from("1,3,2".split(','))
                .skip(30000000 - 1)
                .next()
                .unwrap()
        );
        assert_eq!(
            3544142,
            Recitation::from("2,1,3".split(','))
                .skip(30000000 - 1)
                .next()
                .unwrap()
        );
        assert_eq!(
            261214,
            Recitation::from("1,2,3".split(','))
                .skip(30000000 - 1)
                .next()
                .unwrap()
        );
        assert_eq!(
            6895259,
            Recitation::from("2,3,1".split(','))
                .skip(30000000 - 1)
                .next()
                .unwrap()
        );
        assert_eq!(
            18,
            Recitation::from("3,2,1".split(','))
                .skip(30000000 - 1)
                .next()
                .unwrap()
        );
        assert_eq!(
            362,
            Recitation::from("3,1,2".split(','))
                .skip(30000000 - 1)
                .next()
                .unwrap()
        );
    }
}
