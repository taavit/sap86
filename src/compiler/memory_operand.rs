use crate::compiler::Operand;

#[derive(Debug, Clone, Copy)]
enum OperandParserState {
    None,
    Start,
    Reading,
    End,
}

pub fn parse_memory_operand(r: &str) -> Option<Operand> {
    let mut state = OperandParserState::None;
    let mut buf = String::new();
    let mut stream = Vec::new();
    for c in r.bytes() {
        match (c, state) {
            (b' '|b'\t', OperandParserState::None|OperandParserState::Start) => {},
            (b'[', OperandParserState::None) => {
                state = OperandParserState::Start;
            }
            (b' '|b'\t', OperandParserState::Reading) => {
                state = OperandParserState::Start;
                stream.push(buf.clone());
                buf.clear();
            }
            (b'+', OperandParserState::Reading) => {
                stream.push(buf.clone());
                buf.clear();
                stream.push("+".to_string());
                state = OperandParserState::Start;
            }
            (b'-', OperandParserState::Reading) => {
                stream.push(buf.clone());
                buf.clear();
                stream.push("-".to_string());
                state = OperandParserState::Start;
            }
            (b'+', OperandParserState::Start) => {
                stream.push("+".to_string());
                state = OperandParserState::Start;
            }
            (b'-', OperandParserState::Start) => {
                stream.push("-".to_string());
                state = OperandParserState::Start;
            }
            (c, OperandParserState::Start|OperandParserState::Reading) if c.is_ascii_alphanumeric() => {
                buf.push(c as char);
                state = OperandParserState::Reading;
            }
            (b']', OperandParserState::Reading) => {
                stream.push(buf.clone());
                buf.clear();
                state = OperandParserState::End;
            }
            (b']', OperandParserState::Start) => {
                state = OperandParserState::End;
            }
            (_, OperandParserState::End) => {
                panic!("End of stream")
            }
            (c, s) => {
                panic!("Unexpected combination: {} {:?}", c as char, s)
            }
        }
    }
    dbg!(stream);
    None
}

#[cfg(test)]
mod tests {
    use crate::compiler::memory_operand::parse_memory_operand;

    #[test]
    fn test_memory_reader() {
        parse_memory_operand("[bx+123]");
        parse_memory_operand("[123h]");
        parse_memory_operand("[bx+si+123h]");
        parse_memory_operand("[ bx + si + 1a3h]");
        parse_memory_operand("[ bx + si + 0x2a]");
    }
}
