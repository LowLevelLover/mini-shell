#[allow(unused_imports)]
mod command;
mod operators;
mod parser;
mod state;

use std::io::{self, Write};

use operators::Operators;
use parser::WordParser;
use state::State;

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
            let words = WordParser::split(&input);
            let mut operators = Operators::create_queue(words, &state);
            operators.iter_mut().for_each(|op| op.exec(&mut state));
        }
    }
}
