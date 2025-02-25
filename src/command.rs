use std::{
    env, fs,
    io::{self, Write},
    process,
    sync::OnceLock,
};

use crate::{operators::Operators, parser::WordParser, state::State};

static CACHE: OnceLock<Vec<fs::DirEntry>> = OnceLock::new();
static COMMANDS: [&str; 5] = ["exit", "echo", "type", "pwd", "cd"];

#[derive(Debug)]
pub(crate) struct Command {
    cmd: CommandType,
    output: Option<String>,
    error: Option<String>,
}

impl Command {
    pub fn new(cmd: CommandType) -> Self {
        Self {
            cmd,
            output: None,
            error: None,
        }
    }

    pub fn exec(&mut self, state: &mut State) {
        self.cmd.exec(state);
        self.output = Some(state.flush_output_buf());
        self.error = Some(state.flush_error_buf());
    }

    pub fn output(&self) -> Option<&String> {
        match &self.output {
            Some(o) => Some(&o),
            None => None,
        }
    }

    pub fn error(&self) -> Option<&String> {
        match &self.error {
            Some(e) => Some(&e),
            None => None,
        }
    }

    pub fn write_to_stdout(&self) {
        let mut stdout = io::stdout();
        if let Some(text) = self.output() {
            stdout.write(text.as_bytes()).unwrap();
        }
    }

    pub fn write_to_stderr(&self) {
        let mut stderr = io::stderr();
        if let Some(text) = self.error() {
            stderr.write(text.as_bytes()).unwrap();
        }
    }
}

#[derive(Debug)]
pub(crate) enum CommandType {
    Unknown(String),
    Exit(i32),
    Echo(String),
    Type(String),
    PWD(String),
    CD(String),
    External { command: String, args: Vec<String> },
}

impl CommandType {
    pub(crate) fn parse(parsed_input: Vec<String>, state: &State) -> Self {
        let (command, args_list) = (
            parsed_input[0].as_str(),
            parsed_input.get(1..).unwrap_or(&[]).to_owned(),
        );

        let resolved_args = args_list.join(" ");

        match command {
            "exit" => {
                if args_list.is_empty() {
                    Self::Exit(0)
                } else {
                    Self::Exit(args_list[0].parse::<i32>().unwrap())
                }
            }
            "echo" => Self::Echo(resolved_args),
            "type" => {
                if resolved_args.is_empty() {
                    Self::Type("type".to_string())
                } else {
                    Self::Type(resolved_args)
                }
            }
            "pwd" => Self::PWD(String::from(state.pwd())),
            "cd" => {
                let path = if resolved_args.is_empty() {
                    env::var("HOME").unwrap()
                } else {
                    resolved_args.trim_end().to_string()
                };

                let home = env::var("HOME").unwrap();

                let path_parts: Vec<&str> = match path.chars().next() {
                    Some('~') => {
                        if let Some(p) = path.get(2..) {
                            home.split('/').chain(p.split('/')).collect()
                        } else {
                            home.split('/').collect()
                        }
                    }
                    Some('/') => path.split('/').collect(),
                    Some(_) => state.pwd().split('/').chain(path.split('/')).collect(),
                    None => unreachable!(),
                };

                let mut resolved_path = Vec::<&str>::new();

                for path_part in path_parts {
                    match path_part {
                        "." => {}
                        ".." => {
                            if resolved_path.len() > 1 {
                                resolved_path.pop();
                            }
                        }
                        "" => {
                            if resolved_path.is_empty() {
                                resolved_path.push(path_part);
                            }
                        }
                        _ => resolved_path.push(path_part),
                    }
                }

                let path = if resolved_path.len() > 1 {
                    resolved_path.join("/")
                } else {
                    "/".to_string()
                };

                Self::CD(path)
            }
            input => {
                if Self::find_ext_command(&command).is_some() {
                    Self::External {
                        command: command.to_owned(),
                        args: args_list.to_owned(),
                    }
                } else {
                    Self::Unknown(input.to_owned())
                }
            }
        }
    }

    pub fn exec(&self, state: &mut State) {
        match self {
            Self::Unknown(cmd) => {
                state.write_error(&format!("{}: command not found\n", cmd.trim_end()))
            }
            Self::Exit(code) => process::exit(*code),
            Self::Echo(text) => state.write_output(&format!("{}\n", text)),
            Self::Type(cmd) => {
                if COMMANDS.contains(&cmd.trim_start()) {
                    state.write_output(&format!("{} is a shell builtin\n", cmd));
                } else if let Some(entry) = Self::find_ext_command(&cmd) {
                    state.write_output(&format!("{} is {}\n", cmd, entry.path().to_str().unwrap()));
                } else {
                    state.write_error(&format!("{}: not found\n", cmd));
                }
            }
            Self::External { command, args } => {
                let output = process::Command::new(command)
                    .args(args)
                    .output()
                    .expect("Failed to execute command");

                let stdout = String::from_utf8(output.stdout).expect("Failed to read stdout");
                let stderr = String::from_utf8(output.stderr).expect("Failed to read stderr");

                state.write_output(&format!("{}", stdout));
                state.write_error(&format!("{}", stderr));
            }
            Self::PWD(path) => state.write_output(&format!("{}\n", path)),
            Self::CD(path) => match fs::exists(&path) {
                Ok(true) => {
                    state.set_pwd(&path);
                }
                Ok(false) => {
                    state.write_error(&format!("cd: {}: No such file or directory\n", path));
                }
                Err(err) => state.write_error(&format!("{}\n", err)),
            },
        }
    }

    fn find_ext_command(target: &str) -> Option<&fs::DirEntry> {
        let ext_commands = CACHE.get_or_init(Self::get_ext_commands);
        ext_commands
            .iter()
            .find(|entry| entry.file_name().eq(target))
    }

    fn get_ext_commands() -> Vec<fs::DirEntry> {
        let pathes = env::var("PATH").unwrap();
        let dirs: Vec<String> = pathes.split(":").map(|el| el.to_string()).collect();

        let mut commands = Vec::new();

        for dir in dirs.iter() {
            if let Ok(entries) = fs::read_dir(dir) {
                commands.extend(entries.flatten());
            }
        }

        commands
    }
}
