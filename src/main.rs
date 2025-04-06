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
    println!("Shell is starting..."); // the following line is necessary to initialize stdout properly in docker container

    let stdin = &io::stdin();
    let mut stdout = stdout().into_raw_mode().expect("Failed to enter raw mode");

    let mut state = State::new();
    let mut trie = TrieNode::default();

    let mut multi_tab: Option<Vec<String>> = None;

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
        let max_cols = termion::terminal_size().unwrap().0 as usize;
        let max_rows = termion::terminal_size().unwrap().1 - 1;

        let mut correct_history: Vec<&str> = vec![];

        for line in history.iter() {
            let line_rows = (line.len() as f32 / max_cols as f32).ceil() as usize;
            for i in 0..line_rows {
                let start = i * max_cols;
                let end = line.len().min((i + 1) * max_cols);

                correct_history.push(&line[start..end]);
            }
        }

        let start_row: usize =
            0isize.max(correct_history.len() as isize - max_rows as isize) as usize;

        for (i, line) in correct_history[start_row..].iter().enumerate() {
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

        let input_row = (correct_history.len() + 1) as u16;
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

        if multi_tab.is_some() && c != Key::Char('\t') {
            multi_tab = None;
        }

        match c {
            Key::Char('\t') => {
                if let Some(words) = trie.get_completed_word(&current_input) {
                    if words.len() == 1 {
                        current_input = words[0].to_string();
                        cursor_pos = current_input.chars().count() + 2;
                    } else {
                        match &multi_tab {
                            Some(w) => {
                                history.push("$ ".to_string() + &current_input);
                                history.push(w.join("  "));
                            }
                            None => {
                                multi_tab = Some(words.to_vec());
                            }
                        }
                    }
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
