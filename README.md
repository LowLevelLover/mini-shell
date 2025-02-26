[![progress-banner](https://backend.codecrafters.io/progress/shell/3136a6d6-b9d0-4b3f-ac06-e50e58a6de69)](https://app.codecrafters.io/users/codecrafters-bot?r=2qF)

This is a starting point for Rust solutions to the
["Build Your Own Shell" Challenge](https://app.codecrafters.io/courses/shell/overview).

In this challenge, you'll build your own POSIX compliant shell that's capable of
interpreting shell commands, running external programs and builtin commands like
cd, pwd, echo and more. Along the way, you'll learn about shell command parsing,
REPLs, builtin commands, and more.

**Note**: If you're viewing this repo on GitHub, head over to
[codecrafters.io](https://codecrafters.io) to try the challenge.

# Project Structure

**Main**: get input from user and run commands.

**State**: Keep shell states like `pwm`, `output`, `error` to flush them to _stdout_ or _file_.

**Parser**: Parse raw input and split it to a vector of shell words. See [Word Splitting](https://www.gnu.org/software/bash/manual/html_node/Word-Splitting.html)

**Command**: Parse shell words and create Builtin or external command with arguments. Each comand could be execute with `exec()` method.

**Operator**: Operators has their own structure and could contain one or two commands to run and some words to handle data. Commands which do not have any relation with any operators are `Pure` commands. e.g: `cat somefile otherfile 2> /tmp/fsw/file.txt` -> this redirection operator will redirect Stderr from `cat somefile otherfile` to the `/tmp/fsw/file.txt` file.
