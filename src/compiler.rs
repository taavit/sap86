use std::collections::HashMap;

#[derive(Debug, Default)]
struct LabelMatch {
    target: Option<u8>,
    places: Vec<u8>,
}

fn decode_register(r: &str) -> u8 {
    match r.trim() {
        "ax" => 0,
        "cx" => 1,
        "dx" => 2,
        "bx" => 3,
        "sp" => 4,
        "bp" => 5,
        "si" => 6,
        "di" => 7,
        _ => panic!("Invalid register"),
    }
}

fn parse_argument(arg: &str) -> u8 {
    let arg = arg.trim();
    if let Some(arg) = arg.strip_suffix('h') {
        u8::from_str_radix(arg, 16).unwrap()
    } else {
        arg.parse().unwrap()
    }
}

pub fn compile_program(program: &str) -> Vec<u8> {
    let mut res = Vec::new();
    let mut labels: HashMap<String, LabelMatch> = HashMap::new();
    let mut parsed = tokenize(program.as_bytes().iter()).into_iter();
    while let Some(next) = parsed.next() {
        match next {
            RawToken::EndLine => {
                continue;
            }
            RawToken::Token(token) => {
                if let Some(l) = token.strip_suffix(':') {
                    labels.entry(l.to_string()).or_default().target = Some(res.len() as u8);
                    continue;
                }
                match token.as_str() {
                    "lea" => {
                        let Some(RawToken::Token(dest)) = parsed.next() else {
                            panic!("Missing argument");
                        };
                        res.push(0x11);
                        let entry = labels.entry(dest.trim().to_string()).or_default();
                        entry.places.push(res.len() as u8);
                        res.push(0);
                    }
                    "mov" => {
                        let Some(RawToken::Token(reg1)) = parsed.next() else {
                            panic!("Missing argument 1");
                        };
                        let Some(RawToken::Token(reg2)) = parsed.next() else {
                            panic!("Missing argument 2");
                        };
                        let dst = decode_register(&reg1);
                        let src = decode_register(&reg2) << 2;
                        res.push(0x20 + dst + src);
                    }
                    "ld" => {
                        let Some(RawToken::Token(reg1)) = parsed.next() else {
                            panic!("Missing argument 1");
                        };
                        let dst = decode_register(&reg1);
                        res.push(0x60 + dst)
                    }
                    "test" => {
                        let Some(RawToken::Token(reg1)) = parsed.next() else {
                            panic!("Missing argument 1");
                        };
                        let dst = decode_register(&reg1);
                        res.push(0x70 + dst)
                    }
                    "jz" => {
                        let Some(RawToken::Token(arg1)) = parsed.next() else {
                            panic!("Missing argument 1");
                        };
                        let entry = labels.entry(arg1).or_default();
                        res.push(0x51);
                        entry.places.push(res.len() as u8);
                        res.push(0);
                    }
                    "int" => {
                        let Some(RawToken::Token(arg1)) = parsed.next() else {
                            panic!("Missing argument 1");
                        };
                        let v = parse_argument(&arg1);
                        if v == 3 {
                            res.push(0xCC);
                        } else {
                            res.push(0xCD);
                            res.push(v);
                        }
                    }
                    "inc" => {
                        let Some(RawToken::Token(arg1)) = parsed.next() else {
                            panic!("Missing argument 1");
                        };
                        let dst = decode_register(&arg1);
                        res.push(0x40 + dst);
                    }
                    "jmp" => {
                        let Some(RawToken::Token(arg1)) = parsed.next() else {
                            panic!("Missing argument 1");
                        };
                        res.push(0x52);
                        let entry = labels.entry(arg1).or_default();
                        entry.places.push(res.len() as u8);
                        res.push(0);
                    }
                    "hlt" => {
                        res.push(0xF4);
                    }
                    "ldi" => {
                        let Some(RawToken::Token(arg1)) = parsed.next() else {
                            panic!("Missing argument 1");
                        };
                        res.push(0x10);
                        res.push(parse_argument(&arg1));
                    }
                    "db" => {
                        for db in parsed.by_ref() {
                            match db {
                                RawToken::CharToken(t) => res.push(*t.as_bytes().first().unwrap()),
                                RawToken::StringToken(t) => {
                                    for b in t.as_bytes() {
                                        res.push(*b);
                                    }
                                }
                                RawToken::Token(t) => {
                                    let c = parse_argument(&t);
                                    res.push(c);
                                }
                                _ => break,
                            }
                        }
                    }
                    _ => panic!("Unknown token {:?}", token),
                }
            }
            _ => panic!("Invalid token: {:?}", next),
        }
    }

    for (label, position) in labels.drain() {
        if let Some(target) = position.target {
            for place in position.places {
                res[place as usize] = target;
            }
        } else {
            panic!("Label {label} not found");
        }
    }

    res
}

#[derive(Debug, PartialEq, Eq)]
enum ParserState {
    None,
    ReadToken,
    ReadString,
    ReadByte,
    Comment,
}

#[derive(Debug, PartialEq, Eq)]
enum RawToken {
    Token(String),
    StringToken(String),
    CharToken(String),
    EndLine,
}

fn tokenize<'a>(data: impl Iterator<Item = &'a u8>) -> Vec<RawToken> {
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
    use crate::compiler::{RawToken, tokenize};

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
