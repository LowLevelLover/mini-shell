#[allow(unused_imports)]
use std::io::{self, Write};

use codecrafters_shell::Commands;

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let stdin = io::stdin();
        let mut input = String::new();
        stdin.read_line(&mut input).unwrap();
        input = input.trim().to_owned();

        if input.chars().count() != 0 {
            let cmd = Commands::parse(&input);
            cmd.exec();
        }
    }
}
