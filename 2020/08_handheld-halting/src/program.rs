#[derive(Debug, Clone, Copy)]
pub enum Instruction {
    NoOperation(isize),
    Accumulate(isize),
    Jump(isize),
}

impl std::str::FromStr for Instruction {
    type Err = std::convert::Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Instruction::*;
        let mut split = s.split_whitespace();
        let code = split.next().expect("expect at least an code instruction");
        let value = split
            .next()
            .expect(&format!("expect a parameter after the code '{}'", code))
            .parse()
            .expect("expect parameter to be parseable as an 'isize'");
        let instruction = match code {
            "acc" => Accumulate(value),
            "jmp" => Jump(value),
            "nop" => NoOperation(value),
            code => panic!("expect a valid code instruction, found '{}'", code),
        };
        Ok(instruction)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ExitCode {
    InfiniteLoop,
    OutOfBounds,
}

#[derive(Debug, Clone, Default)]
pub struct State {
    index: usize,
    pub accumulator: isize,
    pub exit_code: Option<ExitCode>,
    already_executed: std::collections::BTreeSet<usize>,
}

#[derive(Debug)]
pub struct Program {
    instructions: Vec<Instruction>,
}

impl Program {
    pub fn execute(&self) -> State {
        let mut state = State::default();
        loop {
            if state.index >= self.instructions.len() {
                state.exit_code = Some(ExitCode::OutOfBounds);
                return state;
            }
            if state.already_executed.contains(&state.index) {
                state.exit_code = Some(ExitCode::InfiniteLoop);
                return state;
            }
            let instruction = &self.instructions[state.index];
            use crate::program::Instruction::*;
            state.already_executed.insert(state.index);
            match instruction {
                NoOperation(_) => {
                    state.index += 1;
                }
                Accumulate(value) => {
                    state.accumulator += value;
                    state.index += 1;
                }
                Jump(value) => {
                    use std::convert::TryFrom;
                    if value.is_positive() {
                        state.index += usize::try_from(*value)
                            .expect("expect 'offset: isize' to be convertible to 'usize'");
                    } else {
                        state.index -= usize::try_from(
                            value.checked_neg().expect(
                                format!(
                                    "expect jump offset to not be the minimum '{}'",
                                    isize::MIN
                                )
                                .as_str(),
                            ),
                        )
                        .expect("expect '-offset: isize' to be convertible to 'usize'");
                    }
                }
            }
        }
    }

    pub fn fix(&mut self) {
        for index in 0..self.instructions.len() {
            let instruction = self.instructions[index];
            use Instruction::*;
            let new_instruction = match instruction {
                NoOperation(value) => Jump(value),
                Jump(value) => NoOperation(value),
                _ => continue,
            };
            self.instructions[index] = new_instruction;
            if let Some(ExitCode::OutOfBounds) = self.execute().exit_code {
                return;
            } else {
                // Backup the original instruction
                self.instructions[index] = instruction;
            }
        }
    }
}

impl std::iter::FromIterator<Instruction> for Program {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = Instruction>,
    {
        let instructions: Vec<Instruction> = iter.into_iter().collect();
        Program { instructions }
    }
}

impl std::iter::FromIterator<String> for Program {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = String>,
    {
        iter.into_iter()
            .map(|instruction| {
                instruction
                    .parse::<Instruction>()
                    .expect("expect correct instruction in the program")
            })
            .collect()
    }
}

impl std::convert::From<std::fs::File> for Program {
    fn from(file: std::fs::File) -> Self {
        use std::io::BufRead;
        std::io::BufReader::new(file)
            .lines()
            .map(|line| line.expect("expect line to be parseable as a String"))
            .collect()
    }
}
