#[derive(Clone, PartialEq)]
enum State {
    STRING,
    SPECIAL,
    WHITESPACE,
}

pub fn tokenize(source: String) -> Vec<String> {
    let mut tokens = vec![];
    let mut string_buffer = String::new();
    let mut state = State::STRING;

    for (i, c) in source.chars().enumerate() {
        match state {
            State::SPECIAL | State::STRING => {
                string_buffer.push(c);
            },
            State::WHITESPACE => {},
        }

        match source.chars().nth(i + 1) {
            Some('#') | Some('=') | Some(';') | Some('<') | Some('>') | Some(':') => {
                if state != State::WHITESPACE {
                    tokens.push(string_buffer);
                    string_buffer = String::new();
                }
                state = State::SPECIAL;
            },
            Some(' ') | Some('\n') => {
                state = State::WHITESPACE;
                tokens.push(string_buffer);
                string_buffer = String::new();
            },
            _ => {
                if state != State::STRING && state != State::WHITESPACE {
                    tokens.push(string_buffer);
                    string_buffer = String::new();
                }
                state = State::STRING;
            },
        }
    }

    if !string_buffer.is_empty() {
        tokens.push(string_buffer);
    }

    tokens
}
