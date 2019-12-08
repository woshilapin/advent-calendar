use std::{
    cmp,
    collections::HashSet,
    convert::{From, TryFrom, TryInto},
    env,
    fs::File,
    io::{BufRead, BufReader},
    iter::FromIterator,
    ops::{Deref, DerefMut},
    sync::mpsc::{self, Receiver, Sender},
    thread,
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
    #[error("The program doesn't contain any instruction")]
    ProgramEmpty,
    #[error("Trying to access cell {0} in a array of length {1}")]
    OutOfBound(usize, usize),
    #[error("An input was expected but all inputs have already been consumed")]
    MissingInput(#[from] std::sync::mpsc::RecvError),
    #[error("Failed to send the output")]
    MissingOutput(#[from] std::sync::mpsc::SendError<i64>),
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
    JumpIf(Mode, Mode),
    JumpIfNot(Mode, Mode),
    LessThan(Mode, Mode, Mode),
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
            5 => Ok(Instruction::JumpIf(
                Mode::try_from(intcode.get(0..1).unwrap_or("0"))?,
                Mode::try_from(intcode.get(1..2).unwrap_or("0"))?,
            )),
            6 => Ok(Instruction::JumpIfNot(
                Mode::try_from(intcode.get(0..1).unwrap_or("0"))?,
                Mode::try_from(intcode.get(1..2).unwrap_or("0"))?,
            )),
            7 => Ok(Instruction::LessThan(
                Mode::try_from(intcode.get(0..1).unwrap_or("0"))?,
                Mode::try_from(intcode.get(1..2).unwrap_or("0"))?,
                Mode::try_from(intcode.get(2..3).unwrap_or("0"))?,
            )),
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

fn process_opcode(
    mut program: Vec<i64>,
    sender: Sender<i64>,
    receiver: Receiver<i64>,
) -> Result<(), MyError> {
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
                let input = receiver.recv()?;
                program[input_offset.0] = input;
            }
            Output(output_mode) => {
                let output_offset = offset_from_mode!(index, output_mode)?;
                index += 1;
                sender.send(program[output_offset.0])?;
            }
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
    Ok(())
}

fn phase_settings_combinations(settings: Vec<i64>) -> HashSet<Vec<i64>> {
    let mut combinations = HashSet::new();
    if settings.len() == 1 {
        combinations.insert(vec![settings[0]]);
        return combinations;
    }
    for setting in settings.clone() {
        let mut remaining_settings = settings.clone();
        remaining_settings.retain(|s| *s != setting);
        for combination in phase_settings_combinations(remaining_settings) {
            let mut new_combination = vec![setting];
            new_combination.extend(combination);
            combinations.insert(new_combination);
        }
    }
    combinations
}

fn optimize_thrusters(program: Vec<i64>) -> Result<i64, MyError> {
    let mut max_thrusting = 0;
    #[cfg(not(feature = "loopback"))]
    let phase_combinations = phase_settings_combinations(vec![0, 1, 2, 3, 4]);
    #[cfg(feature = "loopback")]
    let phase_combinations = phase_settings_combinations(vec![5, 6, 7, 8, 9]);
    for phase_settings in phase_combinations {
        let mut threads = Vec::new();
        let (mut sender, mut receiver) = mpsc::channel();
        let init_sender = sender.clone();
        for phase_setting in phase_settings {
            sender.send(phase_setting)?;
            let channel = mpsc::channel();
            sender = channel.0;
            let next_receiver = channel.1;
            let sender_for_thread = sender.clone();
            let program_for_thread = program.clone();
            let thread = thread::Builder::new()
                .name(phase_setting.to_string())
                .spawn(move || {
                    process_opcode(program_for_thread, sender_for_thread, receiver).unwrap();
                });
            threads.push(thread);
            receiver = next_receiver;
        }
        init_sender.send(0)?;
        #[cfg(not(feature = "loopback"))]
        let output = receiver.recv()?;
        #[cfg(feature = "loopback")]
        let output = {
            let mut output = receiver.recv()?;
            loop {
                if let Err(_) = init_sender.send(output) {
                    break;
                };
                output = match receiver.recv() {
                    Ok(o) => o,
                    Err(_) => break,
                };
            }
            output
        };
        if output > max_thrusting {
            max_thrusting = output;
        }
    }
    Ok(max_thrusting)
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
    let max_thrusting = optimize_thrusters(program)?;
    println!("Max thrusting is {}", max_thrusting);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn combinations() {
        assert!(phase_settings_combinations(vec![0]).contains(&vec![0]));

        assert!(phase_settings_combinations(vec![0, 1]).contains(&vec![0, 1]));
        assert!(phase_settings_combinations(vec![0, 1]).contains(&vec![1, 0]));

        assert!(phase_settings_combinations(vec![0, 1, 2]).contains(&vec![0, 1, 2]));
        assert!(phase_settings_combinations(vec![0, 1, 2]).contains(&vec![0, 2, 1]));
        assert!(phase_settings_combinations(vec![0, 1, 2]).contains(&vec![1, 0, 2]));
        assert!(phase_settings_combinations(vec![0, 1, 2]).contains(&vec![1, 2, 0]));
        assert!(phase_settings_combinations(vec![0, 1, 2]).contains(&vec![2, 0, 1]));
        assert!(phase_settings_combinations(vec![0, 1, 2]).contains(&vec![2, 1, 0]));
    }

    #[cfg(not(feature = "loopback"))]
    #[test]
    fn process_opcodes() -> Result<(), MyError> {
        assert_eq!(
            43210,
            optimize_thrusters(vec![
                3, 15, 3, 16, 1002, 16, 10, 16, 1, 16, 15, 15, 4, 15, 99, 0, 0
            ])?
        );
        assert_eq!(
            54321,
            optimize_thrusters(vec![
                3, 23, 3, 24, 1002, 24, 10, 24, 1002, 23, -1, 23, 101, 5, 23, 23, 1, 24, 23, 23, 4,
                23, 99, 0, 0
            ])?
        );
        assert_eq!(
            65210,
            optimize_thrusters(vec![
                3, 31, 3, 32, 1002, 32, 10, 32, 1001, 31, -2, 31, 1007, 31, 0, 33, 1002, 33, 7, 33,
                1, 33, 31, 31, 1, 32, 31, 31, 4, 31, 99, 0, 0, 0
            ])?
        );
        Ok(())
    }

    #[cfg(feature = "loopback")]
    #[test]
    fn process_opcodes() -> Result<(), MyError> {
        assert_eq!(
            139629729,
            optimize_thrusters(vec![
                3, 26, 1001, 26, -4, 26, 3, 27, 1002, 27, 2, 27, 1, 27, 26, 27, 4, 27, 1001, 28,
                -1, 28, 1005, 28, 6, 99, 0, 0, 5
            ])?
        );
        assert_eq!(
            18216,
            optimize_thrusters(vec![
                3, 52, 1001, 52, -5, 52, 3, 53, 1, 52, 56, 54, 1007, 54, 5, 55, 1005, 55, 26, 1001,
                54, -5, 54, 1105, 1, 12, 1, 53, 54, 53, 1008, 54, 0, 55, 1001, 55, 1, 55, 2, 53,
                55, 53, 4, 53, 1001, 56, -1, 56, 1005, 56, 6, 99, 0, 0, 0, 0, 10
            ])?
        );
        Ok(())
    }
}
