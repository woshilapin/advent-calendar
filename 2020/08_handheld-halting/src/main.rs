mod program;

use program::Program;
use std::{env, fs::File};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("Only one argument is accepted, received {:#?}", args);
    }
    let file = File::open(&args[1]).expect(format!("expect file '{}' to exist", args[1]).as_str());
    let mut program = Program::from(file);
    let state = program.execute();
    println!("State of the accumulator after loop {}", state.accumulator);
    program.fix();
    let state = program.execute();
    println!(
        "State of the accumulator with normal exit {}",
        state.accumulator
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn infinite_loop() {
        let instructions = r#"nop +0
acc +1
jmp +4
acc +3
jmp -3
acc -99
acc +1
jmp -4
acc +6"#;
        let program: Program = instructions
            .split('\n')
            .map(std::borrow::ToOwned::to_owned)
            .collect();
        let state = program.execute();
        assert_eq!(5, state.accumulator);
    }

    #[test]
    fn fix_program() {
        let instructions = r#"nop +0
acc +1
jmp +4
acc +3
jmp -3
acc -99
acc +1
jmp -4
acc +6"#;
        let mut program: Program = instructions
            .split('\n')
            .map(std::borrow::ToOwned::to_owned)
            .collect();
        program.fix();
        let state = program.execute();
        assert_eq!(8, state.accumulator);
    }
}
