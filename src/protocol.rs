pub fn ok() -> String {
    "+OK\n".to_string()
}

pub fn pong() -> String {
    "+PONG\n".to_string()
}

pub fn err(msg: &str) -> String {
    format!("-ERR {}\n", msg)
}

pub fn integer(n: i64) -> String {
    format!(":{}\n", n)
}

pub fn bulk_string(value: &[u8]) -> String {
    format!("${}\n{}\n", value.len(), String::from_utf8_lossy(value))
}

pub fn nil() -> String {
    "$-1\n".to_string()
}
