use std::{
    env,
    fs::File,
    io::{BufRead, BufReader, Result},
};

fn process_opcode(mut program: Vec<usize>) -> Vec<usize> {
    fn binary_operator<F>(program: &mut [usize], offset: usize, operator: F)
    where
        F: Fn(usize, usize) -> usize,
    {
        let ensure_offset = |offset: usize| -> () {
            assert!(offset < program.len(), "Out of bound reference");
        };
        let offset_a = program[offset + 1];
        ensure_offset(offset_a);
        let offset_b = program[offset + 2];
        ensure_offset(offset_b);
        let a = program[offset_a];
        let b = program[offset_b];
        let output_offset = program[offset + 3];
        ensure_offset(output_offset);
        program[output_offset] = operator(a, b);
    }
    let mut index = 0;
    while index < program.len() && program[index] != 99 {
        match program[index] {
            1 => {
                binary_operator(&mut program, index, |a, b| a + b);
                index += 4;
            }
            2 => {
                binary_operator(&mut program, index, |a, b| a * b);
                index += 4;
            }
            _ => {
                index += 1;
            }
        }
    }
    program
}

fn init_opcode(mut program: Vec<usize>, first: usize, second: usize) -> Vec<usize> {
    program[1] = first;
    program[2] = second;
    program
}

#[cfg(not(feature = "noun-verb"))]
fn run(program: Vec<usize>) {
    let program = init_opcode(program, 12, 2);
    let program = process_opcode(program);
    println!("Position [0] contains '{}'", program[0]);
}

#[cfg(feature = "noun-verb")]
fn run(program: Vec<usize>) {
    for noun in 0..=99 {
        for verb in 0..=99 {
            let test_program = init_opcode(program.clone(), noun, verb);
            let test_program = process_opcode(test_program);
            if test_program[0] == 19690720 {
                println!(
                    "Noun is '{}' and verb is '{}' (100 * noun + verb = {})",
                    noun,
                    verb,
                    100 * noun + verb
                );
            }
        }
    }
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("Only one argument is accepted, received {:#?}", args);
    }
    let file = File::open(&args[1])?;
    let mut reader = BufReader::new(file);
    let mut program_str = String::new();
    reader.read_line(&mut program_str)?;
    let program: Vec<usize> = program_str
        .trim()
        .split(",")
        .map(str::parse)
        .map(std::result::Result::unwrap)
        .collect();
    run(program);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn process() {
        assert_eq!(
            vec![3500, 9, 10, 70, 2, 3, 11, 0, 99, 30, 40, 50],
            process_opcode(vec![1, 9, 10, 3, 2, 3, 11, 0, 99, 30, 40, 50])
        );
        assert_eq!(vec![2, 0, 0, 0, 99], process_opcode(vec![1, 0, 0, 0, 99]));
        assert_eq!(vec![2, 3, 0, 6, 99], process_opcode(vec![2, 3, 0, 3, 99]));
        assert_eq!(
            vec![2, 4, 4, 5, 99, 9801],
            process_opcode(vec![2, 4, 4, 5, 99, 0])
        );
        assert_eq!(
            vec![30, 1, 1, 4, 2, 5, 6, 0, 99],
            process_opcode(vec![1, 1, 1, 4, 99, 5, 6, 0, 99])
        );
    }
}
