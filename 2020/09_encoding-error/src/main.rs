use std::{
    fs::File,
    io::{BufRead, BufReader},
};

#[derive(Debug, PartialEq, Eq)]
enum Xmas {
    Buffer(usize),
    Valid(usize, usize, usize),
    NotValid(usize),
}

#[derive(Debug)]
struct XmasIterator<I>
where
    I: Iterator<Item = String>,
{
    stream: I,
    capacity: usize,
    buffer: std::collections::VecDeque<usize>,
}

impl<I> XmasIterator<I>
where
    I: Iterator<Item = String>,
{
    fn new(stream: I, capacity: usize) -> Self {
        Self {
            stream,
            capacity,
            buffer: std::collections::VecDeque::new(),
        }
    }

    fn check(&self, number: usize) -> Option<(usize, usize)> {
        for operand1 in &self.buffer {
            for operand2 in &self.buffer {
                if operand1 != operand2 && operand1 + operand2 == number {
                    return Some((*operand1, *operand2));
                }
            }
        }
        None
    }

    fn xmas_number(self) -> (usize, Vec<usize>) {
        let mut valid_numbers = Vec::new();
        let invalid_number = self
            .filter_map(|number| match number {
                Xmas::NotValid(n) => Some(n),
                Xmas::Valid(n, _, _) | Xmas::Buffer(n) => {
                    valid_numbers.push(n);
                    None
                }
            })
            .next()
            .expect("expect to find at least one invalid number");
        for low_bound in 0..valid_numbers.len() {
            let mut sum = 0;
            for (up_bound, n) in valid_numbers[low_bound..].iter().enumerate() {
                sum += n;
                if sum == invalid_number {
                    return (
                        invalid_number,
                        valid_numbers[low_bound..=(low_bound + up_bound)].to_vec(),
                    );
                }
            }
        }
        panic!("expect to find a valid set of numbers that sums up to the first invalid number");
    }
}

impl<I> std::iter::Iterator for XmasIterator<I>
where
    I: Iterator<Item = String>,
{
    type Item = Xmas;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(next) = self.stream.next() {
            let number: usize = next
                .parse()
                .expect("expect the String to be parseable as a usize");
            if self.buffer.len() < self.capacity {
                self.buffer.push_back(number);
                // Buffer not filled up yet, just yielding the numbers
                Some(Xmas::Buffer(number))
            } else {
                if let Some((operand1, operand2)) = self.check(number) {
                    self.buffer.push_back(number);
                    self.buffer.pop_front();
                    // New valid number to yield
                    Some(Xmas::Valid(number, operand1, operand2))
                } else {
                    // No more valid number, ending the iterator
                    Some(Xmas::NotValid(number))
                }
            }
        } else {
            // No more number in the file, ending the Iterator
            None
        }
    }
}

fn main() {
    let file = File::open("xmas.txt").expect("expect file 'xmas.txt' to exist");
    let reader = BufReader::new(file);
    let xmas = XmasIterator::new(
        reader
            .lines()
            .map(|line| line.expect("expect line to be parseable as a String")),
        25,
    );
    let (first_invalid, range) = xmas.xmas_number();
    println!("First invalid number is {}", first_invalid);
    println!(
        "Sum of minimum ({}) and maximum ({}) number of the range of numbers suming up to invalid number is {}",
        range.iter().copied().min().unwrap(),
        range.iter().copied().max().unwrap(),
        range.iter().copied().min().unwrap() + range.iter().copied().max().unwrap(),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xmas_simple() {
        use Xmas::*;
        let numbers = || (1usize..=25usize).into_iter().map(|n| n.to_string());
        // 26
        let xmas = XmasIterator::new(numbers().chain(std::iter::once(26usize.to_string())), 25);
        let next = xmas.skip(25).next().unwrap();
        assert_eq!(Valid(26, 1, 25), next);
        // 49
        let xmas = XmasIterator::new(numbers().chain(std::iter::once(49usize.to_string())), 25);
        let next = xmas.skip(25).next().unwrap();
        assert_eq!(Valid(49, 24, 25), next);
        // 100
        let xmas = XmasIterator::new(numbers().chain(std::iter::once(100usize.to_string())), 25);
        let next = xmas.skip(25).next().unwrap();
        assert_eq!(NotValid(100), next);
        // 50
        let xmas = XmasIterator::new(numbers().chain(std::iter::once(50usize.to_string())), 25);
        let next = xmas.skip(25).next().unwrap();
        assert_eq!(NotValid(50), next);
    }

    #[test]
    fn xmas_missing_20() {
        use Xmas::*;
        let numbers = || {
            (1usize..20usize)
                .into_iter()
                .chain(21usize..=25usize)
                .chain(std::iter::once(45))
                .map(|n| n.to_string())
        };
        // 26
        let xmas = XmasIterator::new(numbers().chain(std::iter::once(26usize.to_string())), 25);
        let next = xmas.skip(25).next().unwrap();
        assert_eq!(Valid(26, 1, 25), next);
        // 65
        let xmas = XmasIterator::new(numbers().chain(std::iter::once(65usize.to_string())), 25);
        let next = xmas.skip(25).next().unwrap();
        assert_eq!(NotValid(65), next);
        // 64
        let xmas = XmasIterator::new(numbers().chain(std::iter::once(64usize.to_string())), 25);
        let next = xmas.skip(25).next().unwrap();
        assert_eq!(Valid(64, 19, 45), next);
        // 66
        let xmas = XmasIterator::new(numbers().chain(std::iter::once(66usize.to_string())), 25);
        let next = xmas.skip(25).next().unwrap();
        assert_eq!(Valid(66, 21, 45), next);
    }

    #[test]
    fn xmas() {
        let numbers = vec![
            35, 20, 15, 25, 47, 40, 62, 55, 65, 95, 102, 117, 150, 182, 127, 219, 299, 277, 309,
            576,
        ]
        .into_iter()
        .map(|n| n.to_string());
        let xmas = XmasIterator::new(numbers, 5);
        let (invalid_number, range) = xmas.xmas_number();
        assert_eq!(127, invalid_number);
        assert_eq!(15, range.iter().copied().min().unwrap());
        assert_eq!(47, range.iter().copied().max().unwrap());
    }
}
