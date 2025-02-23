use std::{env, fs, process, sync::OnceLock};

static CACHE: OnceLock<Vec<fs::DirEntry>> = OnceLock::new();
static COMMANDS: [&str; 5] = ["exit", "echo", "type", "pwd", "cd"];

pub struct State {
    pwd: String,
}

pub enum Commands<'a> {
    Unknown(&'a str),
    Exit(i32),
    Echo(String),
    Type(String),
    PWD(String),
    CD(String),
    External {
        command: &'a str,
        args: Vec<&'a str>,
    },
}

#[derive(PartialEq, Eq)]
enum ArgType {
    None,
    Space,
    Raw,
    Quote,
    DoubleQoute,
}

impl State {
    pub fn new() -> Self {
        let pwd = match env::current_dir() {
            Ok(path) => path.to_string_lossy().to_string(),
            Err(err) => panic!("Error getting current directory: {}", err),
        };

        Self { pwd }
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Commands<'a> {
    pub fn parse(input_raw: &'a str, state: &State) -> Self {
        let (command, args_list) = if let Some(index) = input_raw.find(' ') {
            (
                &input_raw[..index],
                Self::parse_args(input_raw.get(index + 1..).unwrap_or("")),
            )
        } else {
            (input_raw, vec![])
        };

        let resolved_args = args_list.join("");

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
            "pwd" => Self::PWD(String::from(&state.pwd)),
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
                    Some(_) => state.pwd.split('/').chain(path.split('/')).collect(),
                    None => unreachable!(),
                };

                let mut resolved_path = Vec::<&str>::new();

                for path_part in path_parts {
                    match path_part {
                        "." => {} // just skip
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
                if Self::find_ext_command(command).is_some() {
                    Self::External {
                        command,
                        args: args_list,
                    }
                } else {
                    Self::Unknown(input)
                }
            }
        }
    }

    fn parse_args(text: &str) -> Vec<&str> {
        if text.is_empty() {
            return vec![];
        }

        let mut args: Vec<&str> = Vec::new();
        let mut start_index: usize = 0;
        let mut arg_type = ArgType::None;
        let mut buf: Vec<&str> = vec![];

        let mut text_iter = text.chars().enumerate();

        while let Some((i, ch)) = text_iter.next() {
            match arg_type {
                ArgType::None => match ch {
                    ' ' => {
                        arg_type = ArgType::Space;
                        args.push(&text[i..=i]);
                    }
                    '\'' => {
                        arg_type = ArgType::Quote;
                        start_index = i + 1;
                    }
                    '"' => {
                        arg_type = ArgType::DoubleQoute;
                        start_index = i + 1;
                    }
                    _ => {
                        arg_type = ArgType::Raw;
                        start_index = i;
                    }
                },
                ArgType::Space => match ch {
                    ' ' => (),
                    '\'' => {
                        arg_type = ArgType::Quote;
                        start_index = i + 1;
                    }
                    '"' => {
                        arg_type = ArgType::DoubleQoute;
                        start_index = i + 1;
                    }
                    _ => {
                        arg_type = ArgType::Raw;
                        start_index = i;
                    }
                },
                ArgType::Raw => match ch {
                    ' ' => {
                        arg_type = ArgType::Space;
                        args.push(&text[start_index..=i]);
                    }
                    '\'' => {
                        arg_type = ArgType::Quote;
                        start_index = i + 1;
                    }
                    '"' => {
                        arg_type = ArgType::DoubleQoute;
                        start_index = i + 1;
                    }
                    _ => (),
                },
                ArgType::Quote => match ch {
                    '\'' => {
                        arg_type = ArgType::None;
                        args.push(&text[start_index..i]);
                    }
                    _ => (), // just skip
                },
                ArgType::DoubleQoute => match ch {
                    '"' => {
                        arg_type = ArgType::None;
                        args.push(&text[start_index..i]);
                    }
                    '\\' => {
                        if let Some((_, c)) = text_iter.next() {
                            static ESC_CHARS: [char; 3] = ['\\', '$', '"'];
                            if ESC_CHARS.contains(&c) {
                                buf.push(&text[start_index..i]);
                                start_index = i + 1;
                            }
                        }
                    }
                    _ => (), // just skip
                },
            }
        }

        if arg_type == ArgType::Raw {
            args.push(&text[start_index..]);
        }

        args
    }

    pub fn exec(self, state: &mut State) {
        match self {
            Self::Unknown(cmd) => println!("{}: command not found", cmd.trim_end()),
            Self::Exit(code) => process::exit(code),
            Self::Echo(text) => println!("{}", text),
            Self::Type(cmd) => {
                if COMMANDS.contains(&cmd.trim_start()) {
                    println!("{} is a shell builtin", cmd);
                } else if let Some(entry) = Self::find_ext_command(&cmd) {
                    println!("{} is {}", cmd, entry.path().to_str().unwrap());
                } else {
                    println!("{}: not found", cmd);
                }
            }
            Self::External { command, args } => {
                let args: Vec<String> = if !args.is_empty() {
                    args.iter()
                        .map(|el| el.to_string())
                        .collect::<Vec<String>>()
                } else {
                    vec![]
                };

                let output = process::Command::new(command)
                    .args(args)
                    .output()
                    .expect("Failed to execute command");

                let stdout = String::from_utf8(output.stdout).expect("Failed to read output");
                print!("{}", stdout);
            }
            Self::PWD(path) => println!("{}", path),
            Self::CD(path) => match fs::exists(&path) {
                Ok(true) => {
                    state.pwd = path;
                }
                Ok(false) => {
                    println!("cd: {}: No such file or directory", path);
                }
                Err(err) => eprintln!("{}", err),
            },
        }
    }

    fn find_ext_command(target: &str) -> Option<&fs::DirEntry> {
        let ext_commands = CACHE.get_or_init(get_ext_commands);
        ext_commands
            .iter()
            .find(|entry| entry.file_name().eq(target))
    }
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
