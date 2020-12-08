use intcode::{self, Program};
use std::{
    env,
    fs::File,
    io::{self, BufRead, BufReader, Write},
    sync::mpsc,
    thread,
};
use thiserror;

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("Failed to read user input argument")]
    FailedSystemInput(#[from] std::io::Error),
    #[error("Failed to convert to an Offset from a String")]
    InvalidStringOffset(#[from] std::num::ParseIntError),
    #[error("An error occured in program execution")]
    ProgramError(#[from] intcode::Error),
    #[error("Failed to send a message to the program")]
    SendError(#[from] mpsc::SendError<i64>),
    #[error("Diagnostic has an error code of '{0}'")]
    InvalidDiagnostic(i64),
}

fn run_diagnostics(program: Vec<i64>, system_id: i64) -> Result<Vec<i64>, Error> {
    let (sender_to_thread, receiver_from_host) = mpsc::sync_channel(0);
    let (sender_to_host, receiver_from_thread) = mpsc::sync_channel(0);
    let mut program = Program::new(program, receiver_from_host, sender_to_host);
    thread::spawn(move || program.run().unwrap());
    let mut diagnostics = Vec::new();
    sender_to_thread.send(system_id)?;
    loop {
        match receiver_from_thread.recv() {
            Ok(diagnostic) => diagnostics.push(diagnostic),
            Err(_) => return Ok(diagnostics),
        }
    }
}

fn check_diagnostics(diagnostics: Vec<i64>) -> Result<i64, Error> {
    let length = diagnostics.len();
    let final_diagnostic = diagnostics[length - 1];
    for diagnostic in diagnostics.into_iter().take(length - 1) {
        if diagnostic != 0 {
            return Err(Error::InvalidDiagnostic(diagnostic));
        }
    }
    Ok(final_diagnostic)
}

fn main() -> Result<(), Error> {
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
        .map_err(Error::from)?;
    let mut buffer = String::new();
    print!("System ID: ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut buffer)?;
    let system_id = buffer.trim().parse()?;
    let diagnostics = run_diagnostics(program, system_id)?;
    let diagnostic = check_diagnostics(diagnostics)?;
    println!("Diagnostic is {}", diagnostic);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn process_opcodes() -> Result<(), Error> {
        assert_eq!(
            vec![7, 42],
            run_diagnostics(vec![103, 0, 104, 7, 104, 42, 99], 1)?
        );
        Ok(())
    }

    #[test]
    fn process_opcodes_with_tests_and_jumps() -> Result<(), Error> {
        assert_eq!(
            vec![0],
            run_diagnostics(vec![103, 0, 1101, 9, 10, 3, 1008, 3, 3, 3, 4, 3, 99], 1)?
        );
        Ok(())
    }
}
