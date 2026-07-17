#[derive(Debug, PartialEq, Eq)]
enum ParserState {
    None,
    ReadToken,
    ReadString,
    ReadByte,
    Comment,
}

#[derive(Debug, PartialEq, Eq)]
pub enum RawToken {
    Token(String),
    StringToken(String),
    CharToken(String),
    EndLine,
}

pub fn tokenize<'a>(data: impl Iterator<Item = &'a u8>) -> Vec<RawToken> {
    let mut state = ParserState::None;
    let mut raw_tokens: Vec<RawToken> = Vec::new();
    let mut current_token = String::new();
    for byte in data {
        if state == ParserState::None {
            if *byte == b';' {
                state = ParserState::Comment;
                continue;
            }
            if *byte == b'\r' || *byte == b'\n' {
                state = ParserState::None;
                raw_tokens.push(RawToken::EndLine);
                continue;
            }
            if byte.is_ascii_whitespace() || *byte == b',' {
                continue;
            }
            if *byte == b'"' {
                state = ParserState::ReadString;
                continue;
            }
            if *byte == b'\'' {
                state = ParserState::ReadByte;
                continue;
            }
            state = ParserState::ReadToken;
            current_token.push(*byte as char);
        } else if state == ParserState::ReadToken {
            if *byte == b';' {
                state = ParserState::Comment;
                raw_tokens.push(RawToken::Token(current_token.clone()));
                current_token.clear();
                continue;
            }
            if *byte == b' ' || *byte == b'\t' {
                state = ParserState::None;
                raw_tokens.push(RawToken::Token(current_token.clone()));
                current_token.clear();
                continue;
            }
            if *byte == b',' {
                raw_tokens.push(RawToken::Token(current_token.clone()));
                current_token.clear();
                continue;
            }
            if *byte == b'\r' || *byte == b'\n' {
                state = ParserState::None;
                raw_tokens.push(RawToken::Token(current_token.clone()));
                raw_tokens.push(RawToken::EndLine);
                current_token.clear();
                continue;
            }
            current_token.push(*byte as char);
        } else if state == ParserState::Comment && (*byte == b'\r' || *byte == b'\n') {
            state = ParserState::None;
            raw_tokens.push(RawToken::EndLine);
            continue;
        } else if state == ParserState::ReadString {
            if *byte == b'"' {
                state = ParserState::None;
                raw_tokens.push(RawToken::StringToken(current_token.clone()));
                current_token.clear();
                continue;
            }
            current_token.push(*byte as char);
        } else if state == ParserState::ReadByte {
            if *byte == b'\'' {
                state = ParserState::None;
                raw_tokens.push(RawToken::CharToken(current_token.clone()));
                current_token.clear();
                continue;
            }
            current_token.push(*byte as char);
        }
    }
    if !current_token.is_empty() {
        raw_tokens.push(RawToken::Token(current_token));
    }

    raw_tokens
}

#[cfg(test)]
mod tests {
    use crate::compiler::tokenizer::{RawToken, tokenize};

    #[test]
    fn test_tokenizer() {
        let program = "
        ; Tokenizer test
        ; Test
        lea data  ; load binary data
        mov dx,ax ; store on data register
        data:
            db 'H'
            db \"ello\",\",\"
            db \" world!\",0

        ";
        let stream = program.as_bytes().iter();
        assert_eq!(
            tokenize(stream),
            vec![
                RawToken::EndLine,
                RawToken::EndLine,
                RawToken::EndLine,
                RawToken::Token("lea".to_string()),
                RawToken::Token("data".to_string()),
                RawToken::EndLine,
                RawToken::Token("mov".to_string()),
                RawToken::Token("dx".to_string()),
                RawToken::Token("ax".to_string()),
                RawToken::EndLine,
                RawToken::Token("data:".to_string()),
                RawToken::EndLine,
                RawToken::Token("db".to_string()),
                RawToken::CharToken("H".to_string()),
                RawToken::EndLine,
                RawToken::Token("db".to_string()),
                RawToken::StringToken("ello".to_string()),
                RawToken::StringToken(",".to_string()),
                RawToken::EndLine,
                RawToken::Token("db".to_string()),
                RawToken::StringToken(" world!".to_string()),
                RawToken::Token("0".to_string()),
                RawToken::EndLine,
                RawToken::EndLine,
            ]
        );
    }
}
