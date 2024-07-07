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
    let mut i = 0;
    let source_chars = source.chars().collect::<Vec<_>>();

    while i < source_chars.len() - 1 {
        match state {
            State::SPECIAL | State::STRING => {
                string_buffer.push(source_chars[i]);
            },
            State::WHITESPACE => {},
        }

        match source_chars[i + 1] {
            '#' | '=' | ';' | '<' | '>' | ':' | '{' | '}' => {
                if state != State::WHITESPACE {
                    tokens.push(string_buffer);
                    string_buffer = String::new();
                }
                state = State::SPECIAL;
            },
            ' ' | '\n' => {
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

        i += 1;
    }

    if !string_buffer.is_empty() {
        tokens.push(string_buffer);
    }

    tokens
}
