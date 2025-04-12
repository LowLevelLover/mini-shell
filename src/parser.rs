#[derive(PartialEq, Eq)]
enum WordState {
    None,
    Space,
    Raw,
    Quote,
    DoubleQoute,
    RawBackSlash,
}

pub struct WordParser;

impl WordParser {
    pub fn split(text: &str) -> Vec<String> {
        let mut args = Vec::<String>::new();
        let mut buf = Vec::<&str>::new();
        let mut start_index = 0usize;
        let mut arg_type = WordState::None;
        let mut text_iter = text.chars().enumerate();

        while let Some((i, ch)) = text_iter.next() {
            match arg_type {
                WordState::None => match ch {
                    ' ' => {
                        arg_type = WordState::Space;
                        if !buf.is_empty() {
                            args.push(buf.join(""));
                            buf.clear();
                        }
                    }
                    '\'' => {
                        arg_type = WordState::Quote;
                        start_index = i + 1;
                    }
                    '"' => {
                        arg_type = WordState::DoubleQoute;
                        start_index = i + 1;
                    }
                    '\\' => {
                        arg_type = WordState::RawBackSlash;
                    }
                    _ => {
                        arg_type = WordState::Raw;
                        start_index = i;
                    }
                },
                WordState::Space => match ch {
                    ' ' => (),
                    '\'' => {
                        arg_type = WordState::Quote;
                        start_index = i + 1;
                    }
                    '"' => {
                        arg_type = WordState::DoubleQoute;
                        start_index = i + 1;
                    }
                    '\\' => {
                        arg_type = WordState::RawBackSlash;
                    }
                    _ => {
                        arg_type = WordState::Raw;
                        start_index = i;
                    }
                },
                WordState::Raw => {
                    static DELIM_CHARS: [char; 4] = ['\'', ' ', '"', '\\'];
                    if DELIM_CHARS.contains(&ch) {
                        buf.push(&text[start_index..i]);
                    }

                    match ch {
                        ' ' => {
                            arg_type = WordState::Space;
                            args.push(buf.join(""));
                            buf.clear();
                        }
                        '\'' => {
                            arg_type = WordState::Quote;
                            start_index = i + 1;
                        }
                        '"' => {
                            arg_type = WordState::DoubleQoute;
                            start_index = i + 1;
                        }
                        '\\' => {
                            arg_type = WordState::RawBackSlash;
                        }
                        _ => (),
                    }
                }
                WordState::Quote => {
                    if ch == '\'' {
                        arg_type = WordState::None;
                        buf.push(&text[start_index..i]);
                    }
                }
                WordState::DoubleQoute => match ch {
                    '"' => {
                        arg_type = WordState::None;
                        buf.push(&text[start_index..i]);
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
                    _ => (),
                },
                WordState::RawBackSlash => {
                    arg_type = WordState::Raw;
                    buf.push(&text[i..=i]);
                    start_index = i + 1;
                }
            }
        }

        if arg_type == WordState::Raw {
            buf.push(&text[start_index..]);
            args.push(buf.join(""));
            buf.clear();
        } else if !buf.is_empty() {
            args.push(buf.join(""));
        }

        args
    }
}
