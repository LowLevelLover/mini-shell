use std::{env, fs, process, sync::OnceLock};

static CACHE: OnceLock<Vec<fs::DirEntry>> = OnceLock::new();
static COMMANDS: [&str; 3] = ["exit", "echo", "type"];

pub enum Commands<'a> {
    Unknown(&'a str),
    Exit(i32),
    Echo(&'a str),
    Type(&'a str),
    External { command: &'a str, args: &'a str },
    Pwd(String),
}

impl<'a> Commands<'a> {
    pub fn parse(input_raw: &'a str) -> Self {
        let command: &str;
        let mut args_raw: Option<&str> = None;

        if let Some(index) = input_raw.find(' ') {
            command = &input_raw[..index];
            args_raw = Some(&input_raw[index..]);
        } else {
            command = &input_raw;
        }

        match command {
            "exit" => Self::Exit(
                args_raw
                    .unwrap_or(&"0")
                    .trim_start()
                    .parse::<i32>()
                    .unwrap(),
            ),
            "echo" => Self::Echo(args_raw.unwrap_or("").trim_start()),
            "type" => Self::Type(args_raw.unwrap_or("type")),
            "pwd" => match env::current_dir() {
                Ok(path) => Self::Pwd(path.to_str().unwrap().to_string()),
                Err(err) => panic!("Error getting current directory: {}", err),
            },
            input => {
                if Self::find_ext_command(command).is_some() {
                    Self::External {
                        command,
                        args: args_raw.unwrap_or("").trim(),
                    }
                } else {
                    Self::Unknown(input)
                }
            }
        }
    }

    pub fn exec(self) {
        match self {
            Self::Unknown(cmd) => println!("{}: command not found", cmd.trim_end()),
            Self::Exit(code) => process::exit(code),
            Self::Echo(text) => println!("{}", text),
            Self::Type(cmd) => {
                if COMMANDS.contains(&cmd) {
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
            Self::Pwd(current_path) => println!("{}", current_path),
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
