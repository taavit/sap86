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

    for line in program.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let mut op = line.split_whitespace();
        match op.next() {
            Some("add") => {
                let Some(arg) = op.next() else {
                    panic!("Missing argument");
                };
                res.push(0x01);
                res.push(parse_argument(arg));
                continue;
            }
            Some("ldi") => {
                let Some(arg) = op.next() else {
                    panic!("Missing argument");
                };
                res.push(0x10);
                res.push(parse_argument(arg));
                continue;
            }
            Some("lea") => {
                res.push(0x11);
                let Some(dest) = op.next() else {
                    panic!("Missing argument");
                };
                let entry = labels.entry(dest.trim().to_string()).or_default();
                entry.places.push(res.len() as u8);
                res.push(0);
                continue;
            }
            Some("mov") => {
                let next = op.next();
                if let Some((reg1, reg2)) = next.and_then(|s| s.split_once(',')) {
                    let dst = decode_register(reg1);
                    let src = decode_register(reg2) << 2;
                    res.push(0x20 + dst + src);
                } else {
                    panic!("Invalid command");
                }
                continue;
            }
            Some("int") => {
                let v = op.next().map(parse_argument).unwrap();
                if v == 3 {
                    res.push(0xCC);
                } else {
                    res.push(0xCD);
                    res.push(v);
                }
                continue;
            }
            _ => {}
        }
        if let Some(dest) = line.strip_prefix("inc") {
            let dst = decode_register(dest);
            res.push(0x40 + dst);
        } else if let Some(dest) = line.strip_prefix("dec") {
            let dst = decode_register(dest);
            res.push(0x48 + dst);
        } else if line == "hlt" {
            res.push(0xF4);
        } else if let Some(dest) = line.strip_prefix("jnz") {
            res.push(0x50);
            let entry = labels.entry(dest.trim().to_string()).or_default();
            entry.places.push(res.len() as u8);
            res.push(0);
        } else if let Some(dest) = line.strip_prefix("jz") {
            res.push(0x51);
            let entry = labels.entry(dest.trim().to_string()).or_default();
            entry.places.push(res.len() as u8);
            res.push(0);
        } else if let Some(dest) = line.strip_prefix("jmp") {
            res.push(0x52);
            let entry = labels.entry(dest.trim().to_string()).or_default();
            entry.places.push(res.len() as u8);
            res.push(0);
        } else if let Some(l) = line.strip_suffix(':') {
            labels.entry(l.trim().to_string()).or_default().target = Some(res.len() as u8);
        } else if let Some(db) = line.strip_prefix("db") {
            if let Some(c) = db
                .trim()
                .strip_prefix("'")
                .and_then(|d| d.strip_suffix("'"))
            {
                res.push(c.as_bytes()[0]);
            }
            if let Ok(v) = db.trim().parse() {
                res.push(v);
            }
        } else if let Some(dest) = line.strip_prefix("ld") {
            let dst = decode_register(dest);
            res.push(0x60 + dst)
        } else if let Some(dest) = line.strip_prefix("test") {
            let dst = decode_register(dest);
            res.push(0x70 + dst)
        } else {
            panic!("Unknown instruction: {line}");
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
