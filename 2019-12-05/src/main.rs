use std::{
    cmp,
    convert::{From, TryFrom, TryInto},
    env,
    fs::File,
    io::{self, BufRead, BufReader, Write},
    iter::FromIterator,
    ops::{Deref, DerefMut},
};
use thiserror::Error;

#[derive(Debug, Error)]
enum MyError {
    #[error("Failed to read user input argument")]
    FailedUserInput(#[from] std::io::Error),
    #[error("Failed to convert to an Offset from an integer")]
    InvalidIntOffset(#[from] std::num::TryFromIntError),
    #[error("Failed to convert to an Offset from a String")]
    InvalidStringOffset(#[from] std::num::ParseIntError),
    #[error("Failed to convert '{0}' to a Mode")]
    InvalidMode(String),
    #[error("Failed to convert '{0}' to an Instruction")]
    InvalidInstruction(String),
    #[error("Diagnostic has an error code of '{0}'")]
    InvalidDiagnostic(i64),
    #[error("The program doesn't contain any instruction")]
    ProgramEmpty,
    #[error("Trying to access cell {0} in a array of length {1}")]
    OutOfBound(usize, usize),
}

#[derive(Debug)]
struct Offset(usize);
impl From<usize> for Offset {
    fn from(value: usize) -> Self {
        Offset(value)
    }
}
impl TryFrom<i64> for Offset {
    type Error = MyError;
    fn try_from(value: i64) -> Result<Self, Self::Error> {
        usize::try_from(value)
            .map(Offset::from)
            .map_err(MyError::from)
    }
}
impl Deref for Offset {
    type Target = usize;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for Offset {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug)]
enum Instruction {
    Add(Mode, Mode, Mode),
    Multiply(Mode, Mode, Mode),
    Input(Mode),
    Output(Mode),
    #[cfg(feature = "test-jump")]
    JumpIf(Mode, Mode),
    #[cfg(feature = "test-jump")]
    JumpIfNot(Mode, Mode),
    #[cfg(feature = "test-jump")]
    LessThan(Mode, Mode, Mode),
    #[cfg(feature = "test-jump")]
    Equals(Mode, Mode, Mode),
    Halt,
}
impl TryFrom<i64> for Instruction {
    type Error = MyError;
    fn try_from(intcode: i64) -> Result<Self, Self::Error> {
        let mut intcode = intcode.to_string();
        let length = intcode.len();
        let instruction_code = intcode
            .get((length - cmp::min(length, 2))..)
            .ok_or_else(|| MyError::InvalidInstruction(intcode.clone()))?
            .to_string();
        intcode.truncate(length - instruction_code.len());
        let intcode = String::from_iter(intcode.chars().rev());
        let instruction_code = instruction_code
            .parse::<usize>()
            .map_err(|_| MyError::InvalidInstruction(intcode.clone()))?;
        match instruction_code {
            1 => {
                let op1_mode = Mode::try_from(intcode.get(0..1).unwrap_or("0"))?;
                let op2_mode = Mode::try_from(intcode.get(1..2).unwrap_or("0"))?;
                let result_mode = Mode::try_from(intcode.get(2..3).unwrap_or("0"))?;
                Ok(Instruction::Add(op1_mode, op2_mode, result_mode))
            }
            2 => {
                let op1_mode = Mode::try_from(intcode.get(0..1).unwrap_or("0"))?;
                let op2_mode = Mode::try_from(intcode.get(1..2).unwrap_or("0"))?;
                let result_mode = Mode::try_from(intcode.get(2..3).unwrap_or("0"))?;
                Ok(Instruction::Multiply(op1_mode, op2_mode, result_mode))
            }
            3 => Ok(Instruction::Input(Mode::try_from(
                intcode.get(0..1).unwrap_or("0"),
            )?)),
            4 => Ok(Instruction::Output(Mode::try_from(
                intcode.get(0..1).unwrap_or("0"),
            )?)),
            #[cfg(feature = "test-jump")]
            5 => Ok(Instruction::JumpIf(
                Mode::try_from(intcode.get(0..1).unwrap_or("0"))?,
                Mode::try_from(intcode.get(1..2).unwrap_or("0"))?,
            )),
            #[cfg(feature = "test-jump")]
            6 => Ok(Instruction::JumpIfNot(
                Mode::try_from(intcode.get(0..1).unwrap_or("0"))?,
                Mode::try_from(intcode.get(1..2).unwrap_or("0"))?,
            )),
            #[cfg(feature = "test-jump")]
            7 => Ok(Instruction::LessThan(
                Mode::try_from(intcode.get(0..1).unwrap_or("0"))?,
                Mode::try_from(intcode.get(1..2).unwrap_or("0"))?,
                Mode::try_from(intcode.get(2..3).unwrap_or("0"))?,
            )),
            #[cfg(feature = "test-jump")]
            8 => Ok(Instruction::Equals(
                Mode::try_from(intcode.get(0..1).unwrap_or("0"))?,
                Mode::try_from(intcode.get(1..2).unwrap_or("0"))?,
                Mode::try_from(intcode.get(2..3).unwrap_or("0"))?,
            )),
            99 => Ok(Instruction::Halt),
            code => Err(MyError::InvalidInstruction(code.to_string())),
        }
    }
}

#[derive(Debug)]
enum Mode {
    Immediate,
    Position,
}
impl TryFrom<&str> for Mode {
    type Error = MyError;
    fn try_from(code: &str) -> Result<Self, Self::Error> {
        match code {
            "0" => Ok(Mode::Position),
            "1" => Ok(Mode::Immediate),
            code => Err(MyError::InvalidMode(code.to_string())),
        }
    }
}

fn process_opcode(mut program: Vec<i64>) -> Result<Vec<i64>, MyError> {
    macro_rules! offset_from_mode {
        ($index:expr, $mode:ident) => {
            match $mode {
                Mode::Immediate => Ok(Offset::from($index)),
                Mode::Position => Offset::try_from(program[$index]),
            }
            .and_then(|offset| {
                if offset.0 < program.len() {
                    Ok(offset)
                } else {
                    Err(MyError::OutOfBound(offset.0, program.len()))
                }
            })
        };
    }

    if program.is_empty() {
        return Err(MyError::ProgramEmpty);
    }
    let mut index = 0;
    let mut output_codes = Vec::new();
    while index < program.len() {
        use self::Instruction::*;
        let instruction = Instruction::try_from(program[index])?;
        index += 1;
        match instruction {
            Add(op1_mode, op2_mode, result_mode) => {
                let op1_offset = offset_from_mode!(index, op1_mode)?;
                index += 1;
                let op2_offset = offset_from_mode!(index, op2_mode)?;
                index += 1;
                let result_offset = offset_from_mode!(index, result_mode)?;
                index += 1;
                program[result_offset.0] = program[op1_offset.0] + program[op2_offset.0];
            }
            Multiply(op1_mode, op2_mode, result_mode) => {
                let op1_offset = offset_from_mode!(index, op1_mode)?;
                index += 1;
                let op2_offset = offset_from_mode!(index, op2_mode)?;
                index += 1;
                let result_offset = offset_from_mode!(index, result_mode)?;
                index += 1;
                program[result_offset.0] = program[op1_offset.0] * program[op2_offset.0];
            }
            Input(input_mode) => {
                let input_offset = offset_from_mode!(index, input_mode)?;
                index += 1;
                let mut buffer = String::new();
                print!("User ID: ");
                io::stdout().flush()?;
                io::stdin().read_line(&mut buffer)?;
                program[input_offset.0] = buffer.trim().parse()?;
            }
            Output(output_mode) => {
                let output_offset = offset_from_mode!(index, output_mode)?;
                index += 1;
                output_codes.push(program[output_offset.0]);
            }
            #[cfg(feature = "test-jump")]
            JumpIf(condition_mode, pointer_mode) => {
                let condition_offset = offset_from_mode!(index, condition_mode)?;
                index += 1;
                if program[condition_offset.0] != 0 {
                    let pointer_offset = offset_from_mode!(index, pointer_mode)?;
                    index = program[pointer_offset.0].try_into()?;
                } else {
                    index += 1;
                }
            }
            #[cfg(feature = "test-jump")]
            JumpIfNot(condition_mode, pointer_mode) => {
                let condition_offset = offset_from_mode!(index, condition_mode)?;
                index += 1;
                if program[condition_offset.0] == 0 {
                    let pointer_offset = offset_from_mode!(index, pointer_mode)?;
                    index = program[pointer_offset.0].try_into()?;
                } else {
                    index += 1;
                }
            }
            #[cfg(feature = "test-jump")]
            LessThan(op1_mode, op2_mode, result_mode) => {
                let op1_offset = offset_from_mode!(index, op1_mode)?;
                index += 1;
                let op2_offset = offset_from_mode!(index, op2_mode)?;
                index += 1;
                let result_offset = offset_from_mode!(index, result_mode)?;
                index += 1;
                program[result_offset.0] = if program[op1_offset.0] < program[op2_offset.0] {
                    1
                } else {
                    0
                };
            }
            #[cfg(feature = "test-jump")]
            Equals(op1_mode, op2_mode, result_mode) => {
                let op1_offset = offset_from_mode!(index, op1_mode)?;
                index += 1;
                let op2_offset = offset_from_mode!(index, op2_mode)?;
                index += 1;
                let result_offset = offset_from_mode!(index, result_mode)?;
                index += 1;
                program[result_offset.0] = if program[op1_offset.0] == program[op2_offset.0] {
                    1
                } else {
                    0
                };
            }
            Halt => {
                break;
            }
        }
    }
    Ok(output_codes)
}

fn check_diagnostics(diagnostics: Vec<i64>) -> Result<i64, MyError> {
    let length = diagnostics.len();
    let final_diagnostic = diagnostics[length - 1];
    for diagnostic in diagnostics.into_iter().take(length - 1) {
        if diagnostic != 0 {
            return Err(MyError::InvalidDiagnostic(diagnostic));
        }
    }
    Ok(final_diagnostic)
}

fn main() -> Result<(), MyError> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("Only one argument is accepted, received {:#?}", args);
    }
    let file = File::open(&args[1])?;
    let mut reader = BufReader::new(file);
    let mut program_str = String::new();
    reader.read_line(&mut program_str)?;
    let program: Vec<i64> = program_str
        .trim()
        .split(",")
        .map(str::parse)
        .collect::<Result<_, _>>()
        .map_err(MyError::from)?;
    let diagnostics = process_opcode(program)?;
    let diagnostic = check_diagnostics(diagnostics)?;
    println!("Diagnostic is {}", diagnostic);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn process_opcodes() -> Result<(), MyError> {
        assert_eq!(vec![19], process_opcode(vec![1101, 9, 10, 3, 4, 3])?);
        assert_eq!(vec![5], process_opcode(vec![1101, 9, 10, 3, 4, 5])?);
        Ok(())
    }

    #[cfg(feature = "test-jump")]
    #[test]
    fn process_opcodes_with_tests_and_jumps() -> Result<(), MyError> {
        assert_eq!(
            vec![0],
            process_opcode(vec![1101, 9, 10, 3, 1008, 3, 3, 3, 4, 3])?
        );
        Ok(())
    }
}
