use std::env;

pub struct State {
    pwd: String,
    stdout: String,
    stderr: String,
}

impl State {
    pub fn new() -> Self {
        let pwd = match env::current_dir() {
            Ok(path) => path.to_string_lossy().to_string(),
            Err(err) => panic!("Error getting current directory: {}", err),
        };

        Self {
            pwd,
            stdout: String::new(),
            stderr: String::new(),
        }
    }

    pub fn pwd(&self) -> &str {
        &self.pwd
    }

    pub fn set_pwd(&mut self, pwd: &str) {
        self.pwd = pwd.to_string();
    }

    pub fn flush_stdout(&mut self) -> String {
        let output = self.stdout.clone();
        self.stdout.clear();

        output
    }

    pub fn write_stdout(&mut self, text: &str) {
        self.stdout.push_str(text);
    }

    pub fn flush_stderr(&mut self) -> String {
        let error = self.stderr.clone();
        self.stderr.clear();

        error
    }

    pub fn write_stderr(&mut self, text: &str) {
        self.stderr.push_str(text);
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}
