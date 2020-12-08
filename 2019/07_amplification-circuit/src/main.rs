use intcode::{self, Program};
use std::{
    collections::HashSet,
    env,
    fs::File,
    io::{BufRead, BufReader},
    sync::mpsc,
    thread,
};

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

fn optimize_thrusters(program: Vec<i64>) -> Result<i64, intcode::Error> {
    let mut max_thrusting = 0;
    #[cfg(not(feature = "loopback"))]
    let phase_combinations = phase_settings_combinations(vec![0, 1, 2, 3, 4]);
    #[cfg(feature = "loopback")]
    let phase_combinations = phase_settings_combinations(vec![5, 6, 7, 8, 9]);
    for phase_settings in phase_combinations {
        let mut threads = Vec::new();
        let (mut sender, mut receiver) = mpsc::sync_channel(1);
        let init_sender = sender.clone();
        for phase_setting in phase_settings {
            sender.send(phase_setting)?;
            let channel = mpsc::sync_channel(1);
            sender = channel.0;
            let next_receiver = channel.1;
            let sender_for_thread = sender.clone();
            let mut program_for_thread = Program::new(program.clone(), receiver, sender_for_thread);
            let thread = thread::Builder::new()
                .name(phase_setting.to_string())
                .spawn(move || {
                    program_for_thread.run().unwrap();
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

fn main() -> Result<(), intcode::Error> {
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
        .map_err(intcode::Error::from)?;
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
    fn process_opcodes() -> Result<(), intcode::Error> {
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
    fn process_opcodes() -> Result<(), intcode::Error> {
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
