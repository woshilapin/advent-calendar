use intcode::{self, Program};
use std::{
    env,
    fs::File,
    io::{self, BufRead, BufReader, Write},
    sync::mpsc,
    thread,
};

fn boost_keycode(opcodes: Vec<i64>, user_mode: i64) -> Result<i64, intcode::Error> {
    let (sender_to_thread, receiver_from_host) = mpsc::sync_channel(0);
    let (sender_to_host, receiver_from_thread) = mpsc::sync_channel(0);
    let mut program = Program::new(opcodes, receiver_from_host, sender_to_host);
    thread::spawn(move || program.run().unwrap());
    sender_to_thread.send(user_mode)?;
    receiver_from_thread.recv().map_err(intcode::Error::from)
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
    let mut buffer = String::new();
    print!("Input mode: ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut buffer)?;
    let user_mode = buffer.trim().parse()?;
    let boost_keycode = boost_keycode(program, user_mode)?;
    println!("Boost Keycode is {}", boost_keycode);
    Ok(())
}
