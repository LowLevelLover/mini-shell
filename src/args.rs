#[derive(PartialEq, Eq)]
pub enum ArgType {
    None,
    Space,
    Raw,
    Quote,
    DoubleQoute,
    BackSlash,
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
                        args.push(text[i..=i].to_string());
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
                        arg_type = Self::BackSlash;
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
                    _ => {
                        arg_type = Self::Raw;
                        start_index = i;
                    }
                },
                Self::Raw => match ch {
                    ' ' => {
                        arg_type = Self::Space;
                        args.push(text[start_index..=i].to_string());
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
                        arg_type = Self::BackSlash;
                        args.push(text[start_index..i].to_string());
                    }
                    _ => (),
                },
                Self::Quote => {
                    if ch == '\'' {
                        arg_type = Self::None;
                        args.push(text[start_index..i].to_string());
                    }
                }
                Self::DoubleQoute => match ch {
                    '"' => {
                        arg_type = Self::None;
                        buf.push(&text[start_index..i]);
                        args.push(buf.join(""));
                        buf.clear();
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
                Self::BackSlash => {
                    arg_type = Self::None;
                    args.push(text[i..=i].to_string());
                }
            }
        }

        if arg_type == Self::Raw {
            args.push(text[start_index..].to_string());
        }

        args
    }
}
