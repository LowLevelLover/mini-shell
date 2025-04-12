use std::{env, fs, os::unix::fs::PermissionsExt, path::Path, process, sync::OnceLock};

use crate::state::State;

pub static CACHE: OnceLock<Vec<fs::DirEntry>> = OnceLock::new();
pub static COMMANDS: [&str; 5] = ["exit", "echo", "type", "pwd", "cd"];

#[derive(Debug)]
pub struct Command {
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
        self.exec_cmd(state);
    }

    pub fn output(&self) -> Option<&String> {
        match &self.output {
            Some(o) => Some(o),
            None => None,
        }
    }

    pub fn error(&self) -> Option<&String> {
        match &self.error {
            Some(e) => Some(e),
            None => None,
        }
    }

    fn write_output(&mut self, text: &str) {
        self.output = Some(text.to_string());
    }

    fn write_error(&mut self, text: &str) {
        self.error = Some(text.to_string());
    }

    fn exec_cmd(&mut self, state: &mut State) {
        match &self.cmd {
            CommandType::Unknown(cmd) => {
                self.write_error(&format!("{}: command not found\n", cmd.trim_end()))
            }
            CommandType::Exit(code) => process::exit(*code),
            CommandType::Echo(text) => self.write_output(&format!("{}\n", text)),
            CommandType::Type(cmd) => {
                if COMMANDS.contains(&cmd.trim_start()) {
                    self.write_output(&format!("{} is a shell builtin\n", cmd));
                } else if let Some(entry) = CommandType::find_ext_command(cmd) {
                    self.write_output(&format!("{} is {}\n", cmd, entry.path().to_str().unwrap()));
                } else {
                    self.write_error(&format!("{}: not found\n", cmd));
                }
            }
            CommandType::External { command, args } => {
                let output = process::Command::new(command)
                    .args(args)
                    .output()
                    .expect("Failed to execute command");

                let stdout = String::from_utf8(output.stdout).expect("Failed to read stdout");
                let stderr = String::from_utf8(output.stderr).expect("Failed to read stderr");

                self.write_output(&stdout.to_string());
                self.write_error(&stderr.to_string());
            }
            CommandType::Pwd(path) => self.write_output(&format!("{}\n", path)),
            CommandType::Cd(path) => match fs::exists(path) {
                Ok(true) => {
                    state.set_pwd(path);
                }
                Ok(false) => {
                    self.write_error(&format!("cd: {}: No such file or directory\n", path));
                }
                Err(err) => self.write_error(&format!("{}\n", err)),
            },
        }
    }
}

#[derive(Debug)]
pub enum CommandType {
    Unknown(String),
    Exit(i32),
    Echo(String),
    Type(String),
    Pwd(String),
    Cd(String),
    External { command: String, args: Vec<String> },
}

impl CommandType {
    pub fn parse(parsed_input: Vec<String>, state: &State) -> Self {
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
            "pwd" => Self::Pwd(String::from(state.pwd())),
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

                Self::Cd(path)
            }
            input => {
                if Self::find_ext_command(command).is_some() {
                    Self::External {
                        command: command.to_string(),
                        args: args_list,
                    }
                } else {
                    Self::Unknown(input.to_owned())
                }
            }
        }
    }

    pub fn find_ext_command(target: &str) -> Option<&fs::DirEntry> {
        let ext_commands = CACHE.get_or_init(Self::get_ext_commands);
        ext_commands
            .iter()
            .find(|entry| entry.file_name().eq(target))
    }

    fn is_executable<P: AsRef<Path>>(path: P) -> std::io::Result<bool> {
        let metadata = fs::metadata(&path)?;

        // Check if any executable bit is set (owner: 0o100, group: 0o010, others: 0o001)
        Ok(metadata.permissions().mode() & 0o111 != 0)
    }

    pub fn get_ext_commands() -> Vec<fs::DirEntry> {
        let pathes = env::var("PATH").unwrap();
        let dirs: Vec<String> = pathes.split(":").map(|el| el.to_string()).collect();

        let mut commands = Vec::new();

        for dir in dirs.iter() {
            if let Ok(entries) = fs::read_dir(dir) {
                commands.extend(
                    entries
                        .flatten()
                        .filter(|el| Self::is_executable(el.path()).unwrap_or(false)),
                );
            }
        }

        commands
    }
}
