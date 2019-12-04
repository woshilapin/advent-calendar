use std::{convert::TryFrom, env};
use thiserror::Error;

#[derive(Error, Debug)]
enum MyError {
    #[error("Needs to integer as arguments")]
    WrongNumberOfArguments,
    #[error("Input arguments must be integer")]
    InvalidArgument(#[from] std::num::ParseIntError),
    #[error("Number {0} is not between 100000 and 999999")]
    InvalidBound(usize),
}

struct PasswordIterator {
    current: usize,
    end: usize,
}

impl TryFrom<(usize, usize)> for PasswordIterator {
    type Error = MyError;
    fn try_from((start, end): (usize, usize)) -> Result<Self, Self::Error> {
        let check_number = |num| num >= 100000 && num <= 999999;
        if !check_number(start) {
            return Err(MyError::InvalidBound(start));
        }
        if !check_number(end) {
            return Err(MyError::InvalidBound(end));
        }
        if start < end {
            Ok(PasswordIterator {
                current: start - 1,
                end,
            })
        } else {
            Ok(PasswordIterator {
                current: end - 1,
                end: start,
            })
        }
    }
}

#[cfg(not(feature = "no-group"))]
impl Iterator for PasswordIterator {
    type Item = usize;
    fn next(&mut self) -> Option<Self::Item> {
        'outer: while self.current < self.end {
            self.current += 1;

            let mut previous_char = None;
            let mut has_double = false;
            for c in self.current.to_string().chars() {
                if previous_char.map(|pc| c < pc).unwrap_or(false) {
                    continue 'outer;
                }
                if previous_char.map(|pc| pc == c).unwrap_or(false) {
                    has_double = true;
                }
                previous_char = Some(c);
            }
            if has_double {
                return Some(self.current);
            }
        }
        None
    }
}

#[cfg(feature = "no-group")]
impl Iterator for PasswordIterator {
    type Item = usize;
    fn next(&mut self) -> Option<Self::Item> {
        'outer: while self.current < self.end {
            self.current += 1;

            let mut previous_char = None;
            let mut double_size = 1;
            let mut has_double = false;
            for c in self.current.to_string().chars() {
                if previous_char.map(|pc| c < pc).unwrap_or(false) {
                    continue 'outer;
                }
                if previous_char.map(|pc| pc == c).unwrap_or(false) {
                    double_size += 1;
                } else {
                    if double_size == 2 {
                        has_double = true;
                    }
                    double_size = 1;
                }
                previous_char = Some(c);
            }
            if has_double || double_size == 2 {
                return Some(self.current);
            }
        }
        None
    }
}

fn valid_passwords(start: usize, end: usize) -> Result<PasswordIterator, MyError> {
    PasswordIterator::try_from((start, end))
}

fn main() -> Result<(), MyError> {
    let bounds: Vec<usize> = env::args()
        .skip(1)
        .take(2)
        .map(|arg| arg.parse())
        .collect::<Result<_, _>>()?;
    if bounds.len() != 2 {
        return Err(MyError::WrongNumberOfArguments);
    }
    let valid_passwords = valid_passwords(bounds[0], bounds[1])?;
    println!("There is {} valid passwords", valid_passwords.count());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(not(feature = "no-group"))]
    #[test]
    fn is_valid_password() -> Result<(), MyError> {
        assert_eq!(1, valid_passwords(111111, 111111)?.count());
        assert_eq!(0, valid_passwords(223450, 223454)?.count());
        assert_eq!(0, valid_passwords(123789, 123798)?.count());
        assert_eq!(1, valid_passwords(123456, 123466)?.count());
        assert_eq!(1, valid_passwords(112233, 112233)?.count());
        assert_eq!(1, valid_passwords(123444, 123444)?.count());
        assert_eq!(1, valid_passwords(111122, 111122)?.count());
        assert_eq!(2, valid_passwords(111123, 111124)?.count());
        assert_eq!(2, valid_passwords(122223, 122224)?.count());
        assert_eq!(2, valid_passwords(122233, 122234)?.count());
        assert_eq!(1, valid_passwords(122333, 122333)?.count());
        Ok(())
    }

    #[cfg(feature = "no-group")]
    #[test]
    fn is_valid_password() -> Result<(), MyError> {
        assert_eq!(0, valid_passwords(111111, 111111)?.count());
        assert_eq!(0, valid_passwords(223450, 223454)?.count());
        assert_eq!(0, valid_passwords(123789, 123798)?.count());
        assert_eq!(1, valid_passwords(123456, 123466)?.count());
        assert_eq!(1, valid_passwords(112233, 112233)?.count());
        assert_eq!(0, valid_passwords(123444, 123444)?.count());
        assert_eq!(1, valid_passwords(111122, 111122)?.count());
        assert_eq!(0, valid_passwords(111123, 111124)?.count());
        assert_eq!(0, valid_passwords(122223, 122224)?.count());
        assert_eq!(1, valid_passwords(122233, 122234)?.count());
        assert_eq!(1, valid_passwords(122333, 122333)?.count());
        Ok(())
    }
}
