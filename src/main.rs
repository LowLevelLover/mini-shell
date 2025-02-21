#[allow(unused_imports)]
use std::io::{self, Write};
use std::{env, fs, path::PathBuf, process};

struct Command<'a> {
    command: &'a str,
    args: Option<&'a str>,
}

impl<'a> Command<'a> {
    fn new(input: &'a str) -> Self {
        if let Some(index) = input.find(' ') {
            Self {
                command: &input[..index],
                args: Some(&input[index + 1..]),
            }
        } else {
            Self {
                command: input,
                args: None,
            }
        }
    }

    fn run(&self) {
        match self.command {
            "exit" => {
                let code = if let Some(arg) = self.args {
                    arg.parse::<i32>().unwrap()
                } else {
                    0
                };
                process::exit(code);
            }
            "echo" => {
                if let Some(text) = self.args {
                    println!("{}", text);
                } else {
                    println!();
                }
            }
            "type" => {
                static COMMANDS: [&str; 3] = ["exit", "echo", "type"];
                if let Some(text) = self.args {
                    if COMMANDS.contains(&text) {
                        println!("{} is a shell builtin", text);
                    } else {
                        if let Some(path) = Command::find_ex_command(text) {
                            println!("{} is {}", self.args.unwrap(), path.to_str().unwrap());
                        } else {
                            println!("{}: not found", self.args.unwrap())
                        }
                    }
                }
            }
            _ => {
                if Command::find_ex_command(self.command).is_some() {
                    let output = process::Command::new(self.command)
                        .args(self.args)
                        .output()
                        .expect("Failed to execute command");

                    let stdout = String::from_utf8(output.stdout).expect("Failed to read output");
                    print!("{}", stdout);
                } else {
                    println!("{}: command not found", self.command.trim_end());
                }
            }
        }
    }

    fn find_ex_command(target: &str) -> Option<PathBuf> {
        let path = env::var("PATH").unwrap();
        let dirs: Vec<&str> = path.split(":").collect();

        for dir in dirs.iter() {
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries {
                    let entry = entry.unwrap();
                    if entry.file_name().eq(target) {
                        return Some(entry.path());
                    }
                }
            }
        }
        None
    }
}

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let stdin = io::stdin();
        let mut input = String::new();
        stdin.read_line(&mut input).unwrap();
        input = input.trim().to_owned();

        if input.len() != 0 {
            let cmd = Command::new(&input);
            cmd.run();
        }
    }
}
