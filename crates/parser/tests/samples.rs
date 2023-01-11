use core::panic;
use parser::parse_file;
use std::{fs, io, sync::mpsc::channel, thread, time::Duration};

const TIMEOUT: Duration = Duration::from_secs(10);

#[test]
fn run_samples() -> io::Result<()> {
    for sample_program in fs::read_dir("./tests/sample_programs")?.map(|p| p.unwrap().path()) {
        let file_name = sample_program
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_owned();

        // Spawn thread to measure time for the program to timeout
        // Used for programs that run into infinite loops because of regression
        let (sender, receiver) = channel();
        thread::spawn(move || {
            sender
                .send(parse_file(&sample_program))
                .expect("Thread couldn't send a message");
        });

        match receiver.recv_timeout(TIMEOUT) {
            Ok(program) => match program {
                Ok(_) => {
                    println!("{} compiled successfully.", file_name);
                }
                Err(_) => {
                    eprintln!("Regression found in {}.", file_name);
                    panic!();
                }
            },
            Err(_) => {
                eprintln!("{} timed out.", file_name);
                panic!();
            }
        }
    }

    Ok(())
}
