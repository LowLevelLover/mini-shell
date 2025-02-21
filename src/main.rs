#[allow(unused_imports)]
use std::io::{self, Write};
use std::process;

struct Command<'a> {
    command: &'a str,
    args: Option<Vec<&'a str>>,
}

impl<'a> Command<'a> {
    fn new(input: &'a str) -> Self {
        if let Some(index) = input.find(' ') {
            Self {
                command: &input[..index],
                args: Some(Vec::from_iter(input[index..].split_whitespace())),
            }
        } else {
            Self {
                command: input,
                args: None,
            }
        }
    }

    fn run(&self) {
        match self.command {
            "exit" => {
                let code = self.args.as_ref().unwrap_or(&vec!["0"])[0]
                    .parse::<i32>()
                    .unwrap();
                process::exit(code);
            }
            _ => {
                println!("{}: command not found", self.command.trim_end());
            }
        }
    }
}

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let stdin = io::stdin();
        let mut input = String::new();
        stdin.read_line(&mut input).unwrap();
        input = input.trim().to_owned();

        if input.len() != 0 {
            let cmd = Command::new(&input);
            cmd.run();
        }
    }
}
