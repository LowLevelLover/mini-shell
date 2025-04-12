# Mini-Shell

Welcome to **Mini-Shell**, a lightweight POSIX-compliant Unix shell built as part of the [CodeCrafters "Build Your Own Shell" challenge](https://codecrafters.io). This project is written in Rust and demonstrates the core functionality of a Unix shell, including command parsing, executing external programs, and handling built-in commands like `cd`, `pwd`, `echo`, and more.

## Features

- **Command Parsing**: Interprets and splits user input into shell words for execution.
- **Built-in Commands**: Supports essential commands such as:
  - `cd` - Change directory
  - `pwd` - Print working directory
  - `echo` - Display text
- **External Programs**: Executes external commands with proper argument handling.
- **Redirection and Operators**: Supports redirection (e.g., `2>` for stderr) and other operators for flexible command execution.
- **REPL**: Provides an interactive Read-Eval-Print Loop for continuous user input.

## Project Structure

The repository is organized as follows:

- **`main.rs`**: Handles user input and orchestrates command execution.
- **`state.rs`**: Manages shell state, including current working directory (`pwd`), output, and error streams.
- **`parser.rs`**: Parses raw input into a vector of shell words (see [Word Splitting](https://www.gnu.org/software/bash/manual/html_node/Word-Splitting.html)).
- **`command.rs`**: Processes shell words to create built-in or external commands, each executable via an `exec()` method.
- **`operator.rs`**: Defines operators (e.g., redirection) and their behavior, including handling commands and data flow. Commands without operators are treated as `Pure` commands.

Example of redirection:
```bash
cat somefile otherfile 2> /tmp/fsw/file.txt
```
This redirects stderr from `cat somefile otherfile` to `/tmp/fsw/file.txt`.

## Getting Started

To try Mini-Shell, follow these steps:

1. **Clone the Repository**:
   ```bash
   git clone https://github.com/LowLevelLover/mini-shell.git
   cd mini-shell
   ```

2. **Install Rust**:
   Ensure you have [Rust](https://www.rust-lang.org/tools/install) installed. You can install it using:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

3. **Build and Run**:
   ```bash
   cargo build
   cargo run
   ```

4. **Interact with the Shell**:
   Once running, you can enter commands like:
   ```bash
   pwd
   cd /tmp
   echo Hello, World!
   ls -l
   ```

## Contributing

Contributions are welcome! If you'd like to improve Mini-Shell, feel free to:

- Open an issue to report bugs or suggest features.
- Submit a pull request with your changes.

Please ensure your code follows the existing style and includes appropriate tests.

## About

This project is part of the CodeCrafters "Build Your Own Shell" challenge, designed to teach shell command parsing, REPL implementation, and command execution. For more details, visit [CodeCrafters](https://codecrafters.io).

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

---
