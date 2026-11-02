//! 极简 JSON 读取器（只用于读取本地配置，避免引入 serde 依赖）。

use std::collections::HashMap;

#[derive(Clone, Debug)]
pub enum Value {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
}

pub type Obj = HashMap<String, Value>;

impl Value {
    pub fn as_num(&self) -> Option<f64> {
        match self {
            Value::Num(n) => Some(*n),
            Value::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
            Value::Str(s) => s.parse().ok(),
            _ => None,
        }
    }
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            Value::Num(n) => Some(*n != 0.0),
            Value::Str(s) => Some(s == "true"),
            _ => None,
        }
    }
}

pub fn get_num(map: &Obj, k: &str) -> Option<f64> {
    map.get(k).and_then(|v| v.as_num())
}

pub fn get_bool(map: &Obj, k: &str) -> Option<bool> {
    map.get(k).and_then(|v| v.as_bool())
}

struct P<'a> {
    b: &'a [u8],
    i: usize,
}

impl<'a> P<'a> {
    fn ws(&mut self) {
        while self.i < self.b.len() && (self.b[self.i] as char).is_whitespace() {
            self.i += 1;
        }
    }
    fn peek(&mut self) -> Option<u8> {
        self.ws();
        self.b.get(self.i).copied()
    }
    fn eat(&mut self, c: u8) -> bool {
        if self.peek() == Some(c) {
            self.i += 1;
            true
        } else {
            false
        }
    }
    fn string(&mut self) -> Option<String> {
        if !self.eat(b'"') {
            return None;
        }
        let mut out = String::new();
        while let Some(c) = self.b.get(self.i).copied() {
            self.i += 1;
            match c {
                b'"' => return Some(out),
                b'\\' => {
                    let e = *self.b.get(self.i)?;
                    self.i += 1;
                    match e {
                        b'n' => out.push('\n'),
                        b't' => out.push('\t'),
                        b'r' => out.push('\r'),
                        b'u' => {
                            let hex = std::str::from_utf8(self.b.get(self.i..self.i + 4)?).ok()?;
                            self.i += 4;
                            if let Ok(cp) = u32::from_str_radix(hex, 16) {
                                if let Some(ch) = char::from_u32(cp) {
                                    out.push(ch);
                                }
                            }
                        }
                        other => out.push(other as char),
                    }
                }
                _ => out.push(c as char),
            }
        }
        None
    }
    fn value(&mut self) -> Option<Value> {
        match self.peek()? {
            b'"' => self.string().map(Value::Str),
            b'{' => {
                self.i += 1;
                let mut map = Obj::new();
                loop {
                    if self.eat(b'}') {
                        break;
                    }
                    let k = self.string()?;
                    if !self.eat(b':') {
                        return None;
                    }
                    let v = self.value()?;
                    map.insert(k, v);
                    if !self.eat(b',') && self.peek() != Some(b'}') {
                        return None;
                    }
                }
                Some(Value::Str(format!("__obj{}", map.len())))
            }
            b't' => {
                self.i += 4;
                Some(Value::Bool(true))
            }
            b'f' => {
                self.i += 5;
                Some(Value::Bool(false))
            }
            b'n' => {
                self.i += 4;
                Some(Value::Null)
            }
            _ => {
                let start = self.i;
                while let Some(c) = self.b.get(self.i) {
                    let ch = *c as char;
                    if ch.is_ascii_digit() || "+-.eE".contains(ch) {
                        self.i += 1;
                    } else {
                        break;
                    }
                }
                std::str::from_utf8(&self.b[start..self.i])
                    .ok()?
                    .parse::<f64>()
                    .ok()
                    .map(Value::Num)
            }
        }
    }
}

/// 解析顶层对象；嵌套对象会被跳过（配置文件是扁平结构）。
pub fn parse(text: &str) -> Option<Obj> {
    let mut p = P {
        b: text.as_bytes(),
        i: 0,
    };
    if !p.eat(b'{') {
        return None;
    }
    let mut map = Obj::new();
    loop {
        if p.eat(b'}') {
            break;
        }
        let k = p.string()?;
        if !p.eat(b':') {
            return None;
        }
        let v = p.value()?;
        map.insert(k, v);
        if !p.eat(b',') && p.peek() != Some(b'}') {
            return None;
        }
    }
    Some(map)
}
