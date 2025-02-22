use std::{env, fs, process, sync::OnceLock};

static CACHE: OnceLock<Vec<fs::DirEntry>> = OnceLock::new();
static COMMANDS: [&str; 5] = ["exit", "echo", "type", "pwd", "cd"];

pub struct State {
    pwd: String,
}

pub enum Commands<'a> {
    Unknown(&'a str),
    Exit(i32),
    Echo(&'a str),
    Type(&'a str),
    PWD,
    CD(String),
    External { command: &'a str, args: &'a str },
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

impl<'a> Commands<'a> {
    pub fn parse(input_raw: &'a str) -> Self {
        let command: &str;
        let mut args_raw: Option<&str> = None;

        if let Some(index) = input_raw.find(' ') {
            command = &input_raw[..index];
            args_raw = input_raw.get(index + 1..);
        } else {
            command = &input_raw;
        }

        match command {
            "exit" => Self::Exit(args_raw.unwrap_or(&"0").parse::<i32>().unwrap()),
            "echo" => Self::Echo(args_raw.unwrap_or("")),
            "type" => Self::Type(args_raw.unwrap_or("type")),
            "pwd" => Self::PWD,
            "cd" => match args_raw {
                Some(path) => Self::CD(path.trim_end().to_string()),
                None => Self::CD(env::var("HOME").unwrap()),
            },
            input => {
                if Self::find_ext_command(command).is_some() {
                    Self::External {
                        command,
                        args: args_raw.unwrap_or("").trim_end(),
                    }
                } else {
                    Self::Unknown(input)
                }
            }
        }
    }

    pub fn exec(self, state: &mut State) {
        match self {
            Self::Unknown(cmd) => println!("{}: command not found", cmd.trim_end()),
            Self::Exit(code) => process::exit(code),
            Self::Echo(text) => println!("{}", text),
            Self::Type(cmd) => {
                if COMMANDS.contains(&cmd.trim_start()) {
                    println!("{} is a shell builtin", cmd);
                } else if let Some(entry) = Self::find_ext_command(cmd) {
                    println!("{} is {}", cmd, entry.path().to_str().unwrap());
                } else {
                    println!("{}: not found", cmd);
                }
            }
            Self::External { command, args } => {
                let args = if args.chars().count() == 0 {
                    None
                } else {
                    Some(args)
                };
                let output = process::Command::new(command)
                    .args(args)
                    .output()
                    .expect("Failed to execute command");

                let stdout = String::from_utf8(output.stdout).expect("Failed to read output");
                print!("{}", stdout);
            }
            Self::PWD => println!("{}", state.pwd),
            Self::CD(path) => {
                let home = env::var("HOME").unwrap();
                let path_parts;

                match path.chars().next() {
                    Some('~') => {
                        path_parts = home.split('/').chain(path.split('/'));
                    }
                    Some('/') => path_parts = path.split('/').chain("".split('/')), // The chain is unnecessary but Rust type system is strict and I do not want to use Box<dyn Iterator<Item = &str>>,
                    Some(_) => path_parts = state.pwd.split('/').chain(path.split('/')),
                    None => unreachable!(),
                }

                let mut resolved_path = Vec::<&str>::new();

                for path_part in path_parts {
                    match path_part {
                        "." => {} // just skip
                        ".." => {
                            if resolved_path.len() > 0 {
                                resolved_path.pop();
                            }
                        }
                        "" => {}
                        _ => resolved_path.push(path_part),
                    }
                }

                let path = if resolved_path.len() > 0 {
                    ["/", &resolved_path.join("/")].concat()
                } else {
                    "/".to_string()
                };

                match fs::exists(&path) {
                    Ok(true) => {
                        state.pwd = path;
                    }
                    Ok(false) => {
                        println!("cd: {}: No such file or directory", path);
                    }
                    Err(err) => eprintln!("{}", err),
                }
            }
        }
    }

    fn find_ext_command(target: &str) -> Option<&fs::DirEntry> {
        let ext_commands = CACHE.get_or_init(get_ext_commands);
        ext_commands
            .iter()
            .filter(|entry| entry.file_name().eq(target))
            .next()
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
