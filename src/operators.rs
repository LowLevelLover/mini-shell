use std::{
    fs::{File, OpenOptions},
    io::Write,
};

use anyhow::anyhow;

use crate::{
    command::{Command, CommandType},
    state::State,
};

#[derive(Debug)]
enum OutputType {
    StdOut,
    StdErr,
}

#[derive(Debug)]
enum RedirectType {
    Output(OutputType),
    Input,
    Append(OutputType),
}

impl RedirectType {
    fn from_str(op: &str) -> Result<Self, anyhow::Error> {
        let digits: String = op.chars().take_while(|c| c.is_digit(10)).collect();
        let mut chars = op.get(digits.len()..).unwrap_or(op).chars();

        let first_char = chars.next();

        match first_char {
            Some(c) => {
                if !['<', '>'].contains(&c) {
                    return Err(anyhow!("Failed to read operator"));
                }
            }
            None => {
                return Err(anyhow!("Failed to read operator"));
            }
        }

        let output_type = match digits.parse::<u32>() {
            Ok(1) => OutputType::StdOut,
            Ok(2) => OutputType::StdErr,
            Err(_) => OutputType::StdOut,
            _ => {
                unimplemented!()
            }
        };

        match first_char {
            Some('<') => Ok(Self::Input),
            Some('>') => {
                if let Some('>') = chars.next() {
                    Ok(Self::Append(output_type))
                } else {
                    Ok(Self::Output(output_type))
                }
            }
            Some(_) => Err(anyhow!("Failed to read operator")),
            None => unreachable!(),
        }
    }
}

#[derive(Debug)]
pub(crate) struct Redirect {
    command: Command,
    r_type: RedirectType,
    file: File,
}

impl Redirect {
    fn new(command: Command, r_type: RedirectType, file_path: &str) -> Self {
        let file = match r_type {
            RedirectType::Output(_) => match File::create(file_path) {
                Ok(f) => f,
                Err(err) => panic!("{}", err),
            },
            RedirectType::Input => match File::open(file_path) {
                Ok(f) => f,
                Err(err) => panic!("{}", err),
            },
            RedirectType::Append(_) => {
                match OpenOptions::new().append(true).create(true).open(file_path) {
                    Ok(f) => f,
                    Err(err) => panic!("{}", err),
                }
            }
        };

        Self {
            command,
            r_type,
            file,
        }
    }

    fn exec(&mut self, state: &mut State) {
        match &self.r_type {
            RedirectType::Output(output_type) => {
                self.command.exec(state);
                match output_type {
                    OutputType::StdOut => {
                        self.file
                            .write(self.command.output().unwrap_or(&String::new()).as_bytes())
                            .unwrap();
                        self.command.write_to_stderr();
                    }
                    OutputType::StdErr => {
                        self.file
                            .write(self.command.error().unwrap_or(&String::new()).as_bytes())
                            .unwrap();
                        self.command.write_to_stdout();
                    }
                }
            }
            RedirectType::Append(output_type) => {
                self.command.exec(state);
                match output_type {
                    OutputType::StdOut => {
                        self.file
                            .write(self.command.output().unwrap_or(&String::new()).as_bytes())
                            .unwrap();
                        self.command.write_to_stderr();
                    }
                    OutputType::StdErr => {
                        self.file
                            .write(self.command.error().unwrap_or(&String::new()).as_bytes())
                            .unwrap();
                        self.command.write_to_stdout();
                    }
                }
            }
            RedirectType::Input => unimplemented!(),
        }
    }
}

#[derive(Debug)]
pub enum Operators {
    Pure(Command),
    Redirect(Redirect),
}

impl Operators {
    pub fn create_queue(words: Vec<String>, state: &State) -> Vec<Self> {
        let mut ops = Vec::<Operators>::new();
        let mut buf = Vec::<&str>::new();
        let mut words_iter = words.iter().enumerate();

        while let Some((_, word)) = words_iter.next() {
            if let Ok(r_type) = RedirectType::from_str(word) {
                let command = Command::new(CommandType::parse(
                    buf.iter().map(|el| el.to_string()).collect(),
                    state,
                ));
                buf.clear();

                let file_path = if let Some((_, w)) = words_iter.next() {
                    w.as_str()
                } else {
                    panic!("redirection needs a path");
                };

                ops.push(Self::Redirect(Redirect::new(command, r_type, file_path)));
            } else {
                buf.push(word);
            }
        }

        if !buf.is_empty() {
            let command = Command::new(CommandType::parse(
                buf.iter().map(|el| el.to_string()).collect(),
                state,
            ));
            ops.push(Self::Pure(command));
        }

        ops
    }

    pub(crate) fn exec(&mut self, state: &mut State) {
        match self {
            Self::Pure(cmd) => {
                cmd.exec(state);
                cmd.write_to_stdout();
                cmd.write_to_stderr();
            }
            Self::Redirect(cmd) => cmd.exec(state),
        }
    }
}
