use std::io::Read;
use std::io::{self};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

mod utils;

use utils::*;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let elf_file = if args.len() >= 2 {
        args[1].clone()
    } else {
        "".to_string()
    };

    let mut coredump: Vec<u8> = Vec::new();
    let mut state = State::Wait;
    let mut utf8fixer = Utf8Fixer::new();
    let mut pipe = Tokker::new(vec!["@COREDUMP\n".to_string(), "@ENDCOREDUMP".to_string()]);
    let mut pushed_back = None;

    let receiver = spawn_stdin_stream();
    loop {
        if let Ok(v) = receiver.try_recv() {
            utf8fixer.push(&v);
            pipe.push(&utf8fixer.poll());
        }

        let received = pipe.poll();

        match received {
            TokkerPollResult::None => (),
            TokkerPollResult::Data(to_print) => match state {
                State::Wait => {
                    print!("{to_print}");
                }
                State::Done => (),
                State::Receiving => {
                    let to_convert = if let Some(pushed_back) = pushed_back {
                        format!("{}{}", pushed_back as char, to_print)
                    } else {
                        to_print
                    };
                    pushed_back = None;

                    for b in to_convert.chars().collect::<Vec<char>>().chunks(2) {
                        if b.len() != 2 {
                            pushed_back = Some(b[0]);
                            break;
                        }

                        let b =
                            u8::from_str_radix(&format!("{}{}", b[0] as char, b[1] as char), 16)
                                .unwrap();
                        coredump.push(b);
                    }
                }
                State::Idle => (),
            },
            TokkerPollResult::Token(token) => {
                if token == "@COREDUMP\n" {
                    state = State::Receiving;
                    println!("\n\nReceiving coredump ...");
                } else if token == "@ENDCOREDUMP" {
                    state = State::Done;
                    println!("Got coredump");
                }
            }
        }

        if matches!(state, State::Done) {
            state = State::Idle;

            std::fs::write("./coredump.elf", &coredump).unwrap();
            if coredump[0] != 0x7f || coredump[1] != 0x45 || coredump[2] != 0x4c {
                println!("Coredump corrupted");
            }

            if elf_file != "" {
                let gdb = if elf_file.contains("-esp32-") {
                    "xtensa-esp32-elf-gdb"
                } else if elf_file.contains("-esp32s2-") {
                    "xtensa-esp32s2-elf-gdb"
                } else if elf_file.contains("-esp32s3-") {
                    "xtensa-esp32s3-elf-gdb"
                } else {
                    "riscv32-esp-elf-gdb"
                };

                println!("Run `{} {} coredump.elf`", gdb, elf_file);
            } else {
                println!("Use `riscv32-esp-elf-gdb` or `xtensa-esp32[s2/s3]-elf-gdb` to make use of the coredump.");
            }
        }
    }
}

#[derive(Debug, PartialEq)]
enum State {
    Wait,
    Receiving,
    Done,
    Idle,
}

pub fn spawn_stdin_stream() -> Receiver<Vec<u8>> {
    let (tx, rx): (Sender<Vec<u8>>, Receiver<Vec<u8>>) = mpsc::channel();

    thread::spawn(move || {
        let stdin = io::stdin();
        let mut stdin_lock = stdin.lock();

        loop {
            let mut buffer = [0u8; 1024];
            match stdin_lock.read(&mut buffer) {
                Ok(len) => {
                    let _ = tx.send(Vec::from(&buffer[..len]));
                }
                Err(_) => (), // Read failure
            }
        }
    });

    rx
}
