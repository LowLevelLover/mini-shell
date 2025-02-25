use std::env;

pub struct State {
    pwd: String,
    output_buf: String,
    error_buf: String,
}

impl State {
    pub fn new() -> Self {
        let pwd = match env::current_dir() {
            Ok(path) => path.to_string_lossy().to_string(),
            Err(err) => panic!("Error getting current directory: {}", err),
        };

        Self {
            pwd,
            output_buf: String::new(),
            error_buf: String::new(),
        }
    }

    pub fn pwd(&self) -> &str {
        &self.pwd
    }

    pub fn set_pwd(&mut self, pwd: &str) {
        self.pwd = pwd.to_string();
    }

    pub fn flush_output_buf(&mut self) -> String {
        let output = self.output_buf.clone();
        self.output_buf.clear();

        output
    }

    pub fn write_output(&mut self, text: &str) {
        self.output_buf.push_str(text);
    }

    pub fn flush_error_buf(&mut self) -> String {
        let error = self.error_buf.clone();
        self.error_buf.clear();

        error
    }

    pub fn write_error(&mut self, text: &str) {
        self.error_buf.push_str(text);
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}
