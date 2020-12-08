use std::{
    cmp,
    convert::{From, TryFrom, TryInto},
    iter::FromIterator,
    ops::{Deref, DerefMut},
    sync::mpsc::{Receiver, SyncSender},
};
use thiserror;

#[derive(Debug, thiserror::Error)]
pub enum Error {
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
    type Error = Error;
    fn try_from(value: i64) -> Result<Self, Self::Error> {
        usize::try_from(value)
            .map(Offset::from)
            .map_err(Error::from)
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
    ModifyBase(Mode),
    Halt,
}
impl TryFrom<i64> for Instruction {
    type Error = Error;
    fn try_from(intcode: i64) -> Result<Self, Self::Error> {
        let mut intcode = intcode.to_string();
        let length = intcode.len();
        let instruction_code = intcode
            .get((length - cmp::min(length, 2))..)
            .ok_or_else(|| Error::InvalidInstruction(intcode.clone()))?
            .to_string();
        intcode.truncate(length - instruction_code.len());
        let intcode = String::from_iter(intcode.chars().rev());
        let instruction_code = instruction_code
            .parse::<usize>()
            .map_err(|_| Error::InvalidInstruction(intcode.clone()))?;
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
            9 => Ok(Instruction::ModifyBase(Mode::try_from(
                intcode.get(0..1).unwrap_or("0"),
            )?)),
            99 => Ok(Instruction::Halt),
            code => Err(Error::InvalidInstruction(code.to_string())),
        }
    }
}

#[derive(Debug)]
enum Mode {
    Immediate,
    Position,
    Relative,
}
impl TryFrom<&str> for Mode {
    type Error = Error;
    fn try_from(code: &str) -> Result<Self, Self::Error> {
        match code {
            "0" => Ok(Mode::Position),
            "1" => Ok(Mode::Immediate),
            "2" => Ok(Mode::Relative),
            code => Err(Error::InvalidMode(code.to_string())),
        }
    }
}

pub struct Program {
    opcodes: Vec<i64>,
    inputs: Receiver<i64>,
    outputs: SyncSender<i64>,
    base: usize,
}

impl Program {
    pub fn new(opcodes: Vec<i64>, inputs: Receiver<i64>, outputs: SyncSender<i64>) -> Self {
        Program {
            opcodes,
            inputs,
            outputs,
            base: 0,
        }
    }

    fn offset_from_mode(&mut self, index: usize, mode: Mode) -> Result<Offset, Error> {
        let offset = match mode {
            Mode::Position => Offset::try_from(self.opcodes[index]),
            Mode::Immediate => Ok(Offset::from(index)),
            Mode::Relative => Offset::try_from(self.opcodes[index] + self.base as i64),
        };
        if let Ok(offset) = &offset {
            if offset.0 >= self.opcodes.len() {
                for _ in self.opcodes.len()..=offset.0 {
                    self.opcodes.push(0);
                }
            }
        }
        offset
    }

    pub fn run(&mut self) -> Result<(), Error> {
        if self.opcodes.is_empty() {
            return Err(Error::ProgramEmpty);
        }
        let mut index = 0;
        while index < self.opcodes.len() {
            use self::Instruction::*;
            let instruction = Instruction::try_from(self.opcodes[index])?;
            index += 1;
            match instruction {
                Add(op1_mode, op2_mode, result_mode) => {
                    let op1_offset = self.offset_from_mode(index, op1_mode)?;
                    index += 1;
                    let op2_offset = self.offset_from_mode(index, op2_mode)?;
                    index += 1;
                    let result_offset = self.offset_from_mode(index, result_mode)?;
                    index += 1;
                    self.opcodes[result_offset.0] =
                        self.opcodes[op1_offset.0] + self.opcodes[op2_offset.0];
                }
                Multiply(op1_mode, op2_mode, result_mode) => {
                    let op1_offset = self.offset_from_mode(index, op1_mode)?;
                    index += 1;
                    let op2_offset = self.offset_from_mode(index, op2_mode)?;
                    index += 1;
                    let result_offset = self.offset_from_mode(index, result_mode)?;
                    index += 1;
                    self.opcodes[result_offset.0] =
                        self.opcodes[op1_offset.0] * self.opcodes[op2_offset.0];
                }
                Input(input_mode) => {
                    let input_offset = self.offset_from_mode(index, input_mode)?;
                    index += 1;
                    let input = self.inputs.recv()?;
                    self.opcodes[input_offset.0] = input;
                }
                Output(output_mode) => {
                    let output_offset = self.offset_from_mode(index, output_mode)?;
                    index += 1;
                    self.outputs.send(self.opcodes[output_offset.0])?;
                }
                JumpIf(condition_mode, pointer_mode) => {
                    let condition_offset = self.offset_from_mode(index, condition_mode)?;
                    index += 1;
                    if self.opcodes[condition_offset.0] != 0 {
                        let pointer_offset = self.offset_from_mode(index, pointer_mode)?;
                        index = self.opcodes[pointer_offset.0].try_into()?;
                    } else {
                        index += 1;
                    }
                }
                JumpIfNot(condition_mode, pointer_mode) => {
                    let condition_offset = self.offset_from_mode(index, condition_mode)?;
                    index += 1;
                    if self.opcodes[condition_offset.0] == 0 {
                        let pointer_offset = self.offset_from_mode(index, pointer_mode)?;
                        index = self.opcodes[pointer_offset.0].try_into()?;
                    } else {
                        index += 1;
                    }
                }
                LessThan(op1_mode, op2_mode, result_mode) => {
                    let op1_offset = self.offset_from_mode(index, op1_mode)?;
                    index += 1;
                    let op2_offset = self.offset_from_mode(index, op2_mode)?;
                    index += 1;
                    let result_offset = self.offset_from_mode(index, result_mode)?;
                    index += 1;
                    self.opcodes[result_offset.0] =
                        if self.opcodes[op1_offset.0] < self.opcodes[op2_offset.0] {
                            1
                        } else {
                            0
                        };
                }
                Equals(op1_mode, op2_mode, result_mode) => {
                    let op1_offset = self.offset_from_mode(index, op1_mode)?;
                    index += 1;
                    let op2_offset = self.offset_from_mode(index, op2_mode)?;
                    index += 1;
                    let result_offset = self.offset_from_mode(index, result_mode)?;
                    index += 1;
                    self.opcodes[result_offset.0] =
                        if self.opcodes[op1_offset.0] == self.opcodes[op2_offset.0] {
                            1
                        } else {
                            0
                        };
                }
                ModifyBase(base_mode) => {
                    let base_offset = self.offset_from_mode(index, base_mode)?;
                    index += 1;
                    let new_base = self.opcodes[base_offset.0] + self.base as i64;
                    self.base = new_base.try_into()?;
                }
                Halt => {
                    break;
                }
            }
        }
        Ok(())
    }
}
