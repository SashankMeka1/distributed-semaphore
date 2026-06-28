use bytes::Bytes;

#[derive(Debug, Clone, PartialEq)]
pub enum RespValue {
    SimpleString(String),
    Error(String),
    Integer(i64),
    BulkString(Option<String>), // None = null bulk string
    Array(Option<Vec<RespValue>>),
}

impl RespValue {
    pub fn encode(&self) -> Bytes {
        let s = match self {
            RespValue::SimpleString(s) => format!("+{}\r\n", s),
            RespValue::Error(e) => format!("-{}\r\n", e),
            RespValue::Integer(n) => format!(":{}\r\n", n),
            RespValue::BulkString(None) => "$-1\r\n".to_string(),
            RespValue::BulkString(Some(s)) => format!("${}\r\n{}\r\n", s.len(), s),
            RespValue::Array(None) => "*-1\r\n".to_string(),
            RespValue::Array(Some(items)) => {
                let mut out = format!("*{}\r\n", items.len());
                for item in items {
                    out.push_str(&String::from_utf8_lossy(&item.encode()));
                }
                out
            }
        };
        Bytes::from(s)
    }
}

pub struct RespParser {
    buf: Vec<u8>,
}

impl RespParser {
    pub fn new() -> Self {
        Self { buf: Vec::new() }
    }

    pub fn feed(&mut self, data: &[u8]) {
        self.buf.extend_from_slice(data);
    }

    // Returns the parsed value and how many bytes were consumed
    pub fn parse(&mut self) -> Option<RespValue> {
        let (value, consumed) = parse_value(&self.buf)?;
        self.buf.drain(..consumed);
        Some(value)
    }
}

fn parse_value(buf: &[u8]) -> Option<(RespValue, usize)> {
    if buf.is_empty() {
        return None;
    }
    match buf[0] {
        b'+' => parse_simple_string(buf),
        b'-' => parse_error(buf),
        b':' => parse_integer(buf),
        b'$' => parse_bulk_string(buf),
        b'*' => parse_array(buf),
        _ => None,
    }
}

fn read_line(buf: &[u8]) -> Option<(&[u8], usize)> {
    let pos = buf.windows(2).position(|w| w == b"\r\n")?;
    Some((&buf[1..pos], pos + 2))
}

fn parse_simple_string(buf: &[u8]) -> Option<(RespValue, usize)> {
    let (line, consumed) = read_line(buf)?;
    let s = std::str::from_utf8(line).ok()?.to_string();
    Some((RespValue::SimpleString(s), consumed))
}

fn parse_error(buf: &[u8]) -> Option<(RespValue, usize)> {
    let (line, consumed) = read_line(buf)?;
    let s = std::str::from_utf8(line).ok()?.to_string();
    Some((RespValue::Error(s), consumed))
}

fn parse_integer(buf: &[u8]) -> Option<(RespValue, usize)> {
    let (line, consumed) = read_line(buf)?;
    let n: i64 = std::str::from_utf8(line).ok()?.parse().ok()?;
    Some((RespValue::Integer(n), consumed))
}

fn parse_bulk_string(buf: &[u8]) -> Option<(RespValue, usize)> {
    let (line, header_len) = read_line(buf)?;
    let len: i64 = std::str::from_utf8(line).ok()?.parse().ok()?;
    if len == -1 {
        return Some((RespValue::BulkString(None), header_len));
    }
    let len = len as usize;
    let start = header_len;
    let end = start + len;
    if buf.len() < end + 2 {
        return None; // not enough data yet
    }
    let s = std::str::from_utf8(&buf[start..end]).ok()?.to_string();
    Some((RespValue::BulkString(Some(s)), end + 2))
}

fn parse_array(buf: &[u8]) -> Option<(RespValue, usize)> {
    let (line, header_len) = read_line(buf)?;
    let count: i64 = std::str::from_utf8(line).ok()?.parse().ok()?;
    if count == -1 {
        return Some((RespValue::Array(None), header_len));
    }
    let count = count as usize;
    let mut items = Vec::with_capacity(count);
    let mut offset = header_len;
    for _ in 0..count {
        let (val, consumed) = parse_value(&buf[offset..])?;
        items.push(val);
        offset += consumed;
    }
    Some((RespValue::Array(Some(items)), offset))
}
