#[derive(Clone, Copy, PartialEq, Eq)]
enum Operator {
    Add,
    Mul,
}

impl std::fmt::Debug for Operator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        use Operator::*;
        match self {
            Add => write!(f, "+")?,
            Mul => write!(f, "*")?,
        }
        Ok(())
    }
}

impl std::cmp::PartialOrd for Operator {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        use std::cmp::Ordering;
        #[cfg(not(feature = "add-first"))]
        match (self, other) {
            _ => Some(Ordering::Equal),
        }
        #[cfg(feature = "add-first")]
        match (self, other) {
            (Operator::Add, Operator::Add) | (Operator::Mul, Operator::Mul) => {
                Some(Ordering::Equal)
            }
            (Operator::Add, Operator::Mul) => Some(Ordering::Greater),
            (Operator::Mul, Operator::Add) => Some(Ordering::Less),
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Group {
    Opening,
    Closing,
}

#[derive(Debug, Clone, Copy)]
enum Token {
    Scalar(isize),
    Operator(Operator),
    Group(Group),
}

#[derive(Debug)]
struct Tokens<I>
where
    I: Iterator<Item = char>,
{
    iter: I,
}

impl<I> std::convert::From<I> for Tokens<I>
where
    I: Iterator<Item = char>,
{
    fn from(iter: I) -> Self {
        Self { iter }
    }
}

impl<I> std::iter::Iterator for Tokens<I>
where
    I: Iterator<Item = char>,
{
    type Item = Token;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            return match self.iter.next() {
                Some('(') => Some(Token::Group(Group::Opening)),
                Some(')') => Some(Token::Group(Group::Closing)),
                Some('+') => Some(Token::Operator(Operator::Add)),
                Some('*') => Some(Token::Operator(Operator::Mul)),
                Some(' ') => continue,
                Some(v) => Some(Token::Scalar(
                    v.to_string().parse().expect("expect char to be an integer"),
                )),
                None => None,
            };
        }
    }
}
impl<I> std::iter::DoubleEndedIterator for Tokens<I>
where
    I: Iterator<Item = char> + DoubleEndedIterator,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        loop {
            return match self.iter.next_back() {
                Some('(') => Some(Token::Group(Group::Closing)),
                Some(')') => Some(Token::Group(Group::Opening)),
                Some('+') => Some(Token::Operator(Operator::Add)),
                Some('*') => Some(Token::Operator(Operator::Mul)),
                Some(' ') => continue,
                Some(v) => Some(Token::Scalar(
                    v.to_string().parse().expect("expect char to be an integer"),
                )),
                None => None,
            };
        }
    }
}

enum Operation {
    Scalar(isize),
    Expression {
        operator: Operator,
        op1: Box<Operation>,
        op2: Box<Operation>,
    },
    Group(Box<Operation>),
}

impl std::fmt::Debug for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        use Operation::*;
        match self {
            Scalar(v) => write!(f, "{}", v)?,
            Expression { operator, op1, op2 } => write!(f, "{:?}{:?}{:?}", op1, operator, op2)?,
            Group(operation) => write!(f, "({:?})", operation)?,
        }
        Ok(())
    }
}
impl Operation {
    fn evaluate(self) -> isize {
        use Operation::*;
        match self {
            Scalar(v) => v,
            Expression { operator, op1, op2 } => match operator {
                Operator::Add => op1.evaluate() + op2.evaluate(),
                Operator::Mul => op1.evaluate() * op2.evaluate(),
            },
            Group(operation) => operation.evaluate(),
        }
    }
}

impl Operation {
    fn insert_after(self, new_operator: Operator, new_operation: Operation) -> Self {
        use Operation::*;
        match self {
            Scalar(v) => Expression {
                operator: new_operator,
                op1: Box::new(Scalar(v)),
                op2: Box::new(new_operation),
            },
            Expression { operator, op1, op2 } => {
                if new_operator > operator {
                    Expression {
                        operator,
                        op1,
                        op2: Box::new(op2.insert_after(new_operator, new_operation)),
                    }
                } else {
                    Expression {
                        operator: new_operator,
                        op1: Box::new(Expression { operator, op1, op2 }),
                        op2: Box::new(new_operation),
                    }
                }
            }
            Group(operation) => Expression {
                operator: new_operator,
                op1: Box::new(Group(operation)),
                op2: Box::new(new_operation),
            },
        }
    }
}

impl<I> std::convert::From<&mut I> for Operation
where
    I: Iterator<Item = Token>,
{
    fn from(tokens: &mut I) -> Self {
        enum State {
            Operation(Operation),
            PartialOperation(Operation, Operator),
        }
        impl std::fmt::Debug for State {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
                match self {
                    State::Operation(operation) => write!(f, "{:?}", operation)?,
                    State::PartialOperation(operation, operator) => {
                        write!(f, "{:?}{:?}", operation, operator)?
                    }
                }
                Ok(())
            }
        }
        let mut state = match tokens
            .next()
            .expect("expect at least one token in the operation")
        {
            Token::Scalar(s) => State::Operation(Operation::Scalar(s)),
            Token::Operator(_) => panic!("expect operation to not start with an operator"),
            Token::Group(Group::Opening) => {
                State::Operation(Operation::Group(Box::new(Operation::from(&mut *tokens))))
            }
            Token::Group(Group::Closing) => {
                panic!("expect operation to not start with a closing group")
            }
        };
        while let Some(token) = tokens.next() {
            state = match token {
                Token::Scalar(s) => match state {
                    State::Operation(_) => {
                        panic!("expect an operator but got 2 successive operations")
                    }
                    State::PartialOperation(operation, operator) => {
                        State::Operation(operation.insert_after(operator, Operation::Scalar(s)))
                    }
                },
                Token::Operator(operator) => match state {
                    State::Operation(operation) => State::PartialOperation(operation, operator),
                    State::PartialOperation(_, _) => {
                        panic!("expect another operation but got an operator")
                    }
                },
                Token::Group(Group::Opening) => match state {
                    State::Operation(_) => panic!("expect an operator but got an opening group"),
                    State::PartialOperation(operation, operator) => {
                        State::Operation(operation.insert_after(
                            operator,
                            Operation::Group(Box::new(Operation::from(&mut *tokens))),
                        ))
                    }
                },
                Token::Group(Group::Closing) => match state {
                    State::Operation(operation) => return operation,
                    State::PartialOperation(_, _) => {
                        panic!("expect to have an operation but got a closing group")
                    }
                },
            };
        }
        if let State::Operation(operation) = state {
            operation
        } else {
            panic!("expect to have a full operation but only got a partial one")
        }
    }
}

impl std::convert::From<&'static str> for Operation {
    fn from(line: &'static str) -> Self {
        let operation = Operation::from(&mut Tokens::from(line.chars()));
        operation
    }
}

#[derive(Debug)]
struct Operations<I>
where
    I: Iterator<Item = &'static str>,
{
    iter: I,
}

impl<I> std::convert::From<I> for Operations<I>
where
    I: Iterator<Item = &'static str>,
{
    fn from(iter: I) -> Self {
        Self { iter }
    }
}

impl<I> std::iter::Iterator for Operations<I>
where
    I: Iterator<Item = &'static str>,
{
    type Item = Operation;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(Operation::from)
    }
}

fn main() {
    let operations = Operations::from(include_str!("../operations.txt").lines());
    let sum: isize = operations.map(Operation::evaluate).sum();
    println!("Sum of all operation's results is {}", sum);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(not(feature = "add-first"))]
    fn operation_order() {
        let mut operations = Operations::from(
            r#"1 + 2 * 3 + 4 * 5 + 6
1 + (2 * 3) + (4 * (5 + 6))
2 * 3 + (4 * 5)
5 + (8 * 3 + 9 + 3 * 4 * 3)
5 * 9 * (7 * 3 * 3 + 9 * 3 + (8 + 6 * 4))
((2 + 4 * 9) * (6 + 9 * 8 + 6) + 6) + 2 + 4 * 2"#
                .lines(),
        );
        assert_eq!(71, operations.next().unwrap().evaluate());
        assert_eq!(51, operations.next().unwrap().evaluate());
        assert_eq!(26, operations.next().unwrap().evaluate());
        assert_eq!(437, operations.next().unwrap().evaluate());
        assert_eq!(12240, operations.next().unwrap().evaluate());
        assert_eq!(13632, operations.next().unwrap().evaluate());
    }

    #[test]
    #[cfg(feature = "add-first")]
    fn operation_order() {
        let mut operations = Operations::from(
            r#"1 + 2 * 3 + 4 * 5 + 6
1 + (2 * 3) + (4 * (5 + 6))
2 * 3 + (4 * 5)
5 + (8 * 3 + 9 + 3 * 4 * 3)
5 * 9 * (7 * 3 * 3 + 9 * 3 + (8 + 6 * 4))
((2 + 4 * 9) * (6 + 9 * 8 + 6) + 6) + 2 + 4 * 2"#
                .lines(),
        );
        assert_eq!(231, operations.next().unwrap().evaluate());
        assert_eq!(51, operations.next().unwrap().evaluate());
        assert_eq!(46, operations.next().unwrap().evaluate());
        assert_eq!(1445, operations.next().unwrap().evaluate());
        assert_eq!(669060, operations.next().unwrap().evaluate());
        assert_eq!(23340, operations.next().unwrap().evaluate());
    }
}
