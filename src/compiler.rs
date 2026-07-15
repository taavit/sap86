use std::collections::HashMap;

#[derive(Debug, Default)]
struct LabelMatch {
    target: Option<u8>,
    places: Vec<u8>,
}

fn decode_register(r: &str) -> u8 {
    match r.trim() {
        "a" => 0,
        "b" => 1,
        "c" => 2,
        "d" => 3,
        _ => panic!("Invalid register"),
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
        if let Some(lda) = line.strip_prefix("ldi") {
            res.push(0x10);
            res.push(lda.trim().parse().unwrap())
        } else if let Some(dest) = line.strip_prefix("lea") {
            res.push(0x11);
            let entry = labels.entry(dest.trim().to_string()).or_default();
            entry.places.push(res.len() as u8);
            res.push(0);
        } else if let Some(mov) = line.strip_prefix("mov") {
            if let Some((reg1, reg2)) = mov.split_once(',') {
                let dst = decode_register(reg1);
                let src = decode_register(reg2) << 2;
                res.push(0x20 + dst + src);
            } else {
                panic!("Invalid command");
            }
        } else if let Some(int) = line.strip_prefix("int") {
            let v: u8 = int.trim().parse().unwrap();
            res.push(0x30 + (v & 0x0F));
        } else if let Some(dest) = line.strip_prefix("inc") {
            let dst = decode_register(dest);
            res.push(0x40 + dst);
        } else if let Some(dest) = line.strip_prefix("dec") {
            let dst = decode_register(dest);
            res.push(0x48 + dst);
        } else if line == "hlt" {
            res.push(0xFF);
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
