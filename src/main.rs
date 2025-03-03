#[allow(unused_imports)]
mod autocomplete;
mod command;
mod operators;
mod parser;
mod state;

use std::io::{self, stdout, Write};

use termion::{event::Key, input::TermRead, raw::IntoRawMode};

use autocomplete::TrieNode;
use command::{CommandType, CACHE, COMMANDS};
use operators::Operators;
use parser::WordParser;
use state::State;

const BELL: char = '\u{0007}';

fn main() {
    let stdin = &io::stdin();
    let mut stdout = stdout().into_raw_mode().expect("Failed to enter raw mode");

    let mut state = State::new();
    let mut trie = TrieNode::default();

    for word in COMMANDS.iter() {
        trie.insert(word);
    }

    let ext_commands = CACHE.get_or_init(CommandType::get_ext_commands);
    for path in ext_commands.iter() {
        if let Some(file_name) = path.file_name().to_str() {
            trie.insert(file_name);
        }
    }

    let mut history = Vec::<String>::new();
    let mut current_input = String::new();
    let mut cursor_pos = 2;

    write!(
        stdout,
        "{}{}$ ",
        termion::clear::All,
        termion::cursor::Goto(1, 1),
    )
    .unwrap();

    loop {
        let max_rows = termion::terminal_size().unwrap().1 as usize - 1;
        let start_row = if history.len() >= max_rows {
            history.len() - max_rows
        } else {
            0
        };

        for (i, line) in history[start_row..].iter().enumerate() {
            let row = (i + 1) as u16;
            write!(
                stdout,
                "{}{}{}",
                termion::cursor::Goto(1, row),
                termion::clear::CurrentLine,
                line
            )
            .unwrap();
        }

        let input_row = (history.len() + 1) as u16;
        write!(
            stdout,
            "{}{}$ {}",
            termion::cursor::Goto(1, input_row),
            termion::clear::CurrentLine,
            current_input,
        )
        .unwrap();

        stdout.flush().unwrap();

        let c = stdin.keys().next().unwrap().unwrap();

        match c {
            Key::Char('\t') => {
                if let Some(word) = trie.get_completed_word(&current_input) {
                    current_input = word;
                    cursor_pos = current_input.chars().count() + 2;
                } else {
                    write!(stdout, "{}", BELL).unwrap();
                    stdout.flush().unwrap();
                }
            }
            Key::Backspace => {
                if cursor_pos > 2 {
                    current_input.pop();
                    cursor_pos -= 1;
                }
            }
            Key::Char('\n') => {
                let input = current_input.trim().to_string();

                history.push(format!("$ {}", current_input));

                if input.chars().count() != 0 {
                    let words = WordParser::split(&input);
                    let mut operators = Operators::create_queue(words, &state);
                    operators.iter_mut().for_each(|op| op.exec(&mut state));
                }

                let output = state.flush_stdout();
                let error = state.flush_stderr();

                for line in output.split('\n') {
                    if !line.is_empty() {
                        history.push(line.to_string());
                    }
                }

                for line in error.split('\n') {
                    if !line.is_empty() {
                        history.push(line.to_string());
                    }
                }

                current_input.clear();
                cursor_pos = 2;
            }
            Key::Char(c) => {
                current_input.push(c);
                cursor_pos += 1;
            }
            _ => {}
        }
    }
}
