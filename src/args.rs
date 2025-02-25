#[derive(PartialEq, Eq)]
pub enum ArgType {
    None,
    Space,
    Raw,
    Quote,
    DoubleQoute,
    RawBackSlash,
}

impl ArgType {
    pub fn parse_args(text: &str) -> Vec<String> {
        let mut args = Vec::<String>::new();
        let mut buf = Vec::<&str>::new();
        let mut start_index = 0usize;
        let mut arg_type = Self::None;
        let mut text_iter = text.chars().enumerate();

        while let Some((i, ch)) = text_iter.next() {
            match arg_type {
                Self::None => match ch {
                    ' ' => {
                        arg_type = Self::Space;
                        if !buf.is_empty() {
                            args.push(buf.join(""));
                            buf.clear();
                        }
                    }
                    '\'' => {
                        arg_type = Self::Quote;
                        start_index = i + 1;
                    }
                    '"' => {
                        arg_type = Self::DoubleQoute;
                        start_index = i + 1;
                    }
                    '\\' => {
                        arg_type = Self::RawBackSlash;
                    }
                    _ => {
                        arg_type = Self::Raw;
                        start_index = i;
                    }
                },
                Self::Space => match ch {
                    ' ' => (),
                    '\'' => {
                        arg_type = Self::Quote;
                        start_index = i + 1;
                    }
                    '"' => {
                        arg_type = Self::DoubleQoute;
                        start_index = i + 1;
                    }
                    '\\' => {
                        arg_type = Self::RawBackSlash;
                    }
                    _ => {
                        arg_type = Self::Raw;
                        start_index = i;
                    }
                },
                Self::Raw => {
                    static DELIM_CHARS: [char; 4] = ['\'', ' ', '"', '\\'];
                    if DELIM_CHARS.contains(&ch) {
                        buf.push(&text[start_index..i]);
                    }

                    match ch {
                        ' ' => {
                            arg_type = Self::Space;
                            args.push(buf.join(""));
                            buf.clear();
                        }
                        '\'' => {
                            arg_type = Self::Quote;
                            start_index = i + 1;
                        }
                        '"' => {
                            arg_type = Self::DoubleQoute;
                            start_index = i + 1;
                        }
                        '\\' => {
                            arg_type = Self::RawBackSlash;
                        }
                        _ => (),
                    }
                }
                Self::Quote => {
                    if ch == '\'' {
                        arg_type = Self::None;
                        buf.push(&text[start_index..i]);
                    }
                }
                Self::DoubleQoute => match ch {
                    '"' => {
                        arg_type = Self::None;
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
                Self::RawBackSlash => {
                    arg_type = Self::Raw;
                    buf.push(&text[i..=i]);
                }
            }
        }

        if arg_type == Self::Raw {
            buf.push(&text[start_index..]);
            args.push(buf.join(""));
            buf.clear();
        }

        if !buf.is_empty() {
            args.push(buf.join(""));
        }

        args
    }
}
