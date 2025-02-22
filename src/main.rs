#[allow(unused_imports)]
use std::io::{self, Write};

use codecrafters_shell::{Commands, State};

fn main() {
    let mut state = State::new();

    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let stdin = io::stdin();
        let mut input = String::new();
        stdin.read_line(&mut input).unwrap();
        input = input.trim().to_owned();

        if input.chars().count() != 0 {
            let cmd = Commands::parse(&input);
            cmd.exec(&mut state);
        }
    }
}
