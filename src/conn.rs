use crate::protocol;
use crate::stats::Stats;
use crate::store::Store;
use bytes::Bytes;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

// Guardrails based on size and value
const MAX_LINE_BYTES: usize = 2 * 1024 * 1024; // 2MB
const MAX_VALUE_BYTES: usize = 1 * 1024 * 1024; // 1MB

pub async fn handle_connection(
    stream: TcpStream,
    store: Store,
    stats: Stats,
) -> anyhow::Result<()> {
    let (read_half, mut write_half) = stream.into_split();
    let mut reader = BufReader::new(read_half);

    let mut line = String::new();
    loop {
        line.clear();
        let n = reader.read_line(&mut line).await?;
        if n == 0 {
            return Ok(());
        }

        if line.len() > MAX_LINE_BYTES {
            write_half
                .write_all(protocol::err("line too long").as_bytes())
                .await?;
            continue;
        }

        let input = line.trim();
        if input.is_empty() {
            continue;
        }

        stats.inc_commands();

        let (resp, should_close) = handle_line(input, &store, &stats);
        write_half.write_all(resp.as_bytes()).await?;

        if should_close {
            return Ok(());
        }
    }
}

fn handle_line(line: &str, store: &Store, stats: &Stats) -> (String, bool) {
    let mut parts = line.split_whitespace();
    let cmd = match parts.next() {
        Some(c) => c.to_uppercase(),
        None => return (protocol::err("empty command"), false),
    };

    match cmd.as_str() {
        "PING" => (protocol::pong(), false),

        "HELP" => {
            let msg = concat!(
                "+COMMANDS:\n",
                "PING\n",
                "GET <key>\n",
                "SET <key> <value...> [EX <seconds>]\n",
                "DEL <key>\n",
                "TTL <key>\n",
                "EXPIRE <key> <seconds>\n",
                "STATS\n",
                "HELP\n",
                "QUIT\n"
            );
            (msg.to_string(), false)
        }

        "QUIT" => ("+BYE\n".to_string(), true),

        "STATS" => (stats.render(), false),

        "GET" => {
            stats.inc_gets();

            let key = match parts.next() {
                Some(k) => k,
                None => return (protocol::err("GET requires key"), false),
            };

            let resp = match store.get(key) {
                None => {
                    stats.inc_misses();
                    protocol::nil()
                }
                Some(v) => {
                    stats.inc_hits();
                    protocol::bulk_string(&v)
                }
            };
            (resp, false)
        }

        "SET" => {
            stats.inc_sets();

            let mut it = line.split_whitespace();
            it.next();

            let key = match it.next() {
                Some(k) => k.to_string(),
                None => return (protocol::err("SET requires key"), false),
            };

            let needle = format!("SET {}", key);
            let rest = match line.find(&needle) {
                Some(pos) => line[pos + needle.len()..].trim(),
                None => return (protocol::err("SET parse error"), false),
            };

            if rest.is_empty() {
                return (protocol::err("SET requires value"), false);
            }

            let tokens: Vec<&str> = rest.split_whitespace().collect();
            let (value_str, ex) =
                if tokens.len() >= 3 && tokens[tokens.len() - 2].eq_ignore_ascii_case("EX") {
                    let secs = match tokens[tokens.len() - 1].parse::<u64>() {
                        Ok(s) => s,
                        Err(_) => return (protocol::err("EX requires integer seconds"), false),
                    };
                    let value_tokens = &tokens[..tokens.len() - 2];
                    (value_tokens.join(" "), Some(secs))
                } else {
                    (rest.to_string(), None)
                };

            if value_str.as_bytes().len() > MAX_VALUE_BYTES {
                return (protocol::err("value too large"), false);
            }

            store.set(key, Bytes::from(value_str), ex);
            (protocol::ok(), false)
        }

        "DEL" => {
            stats.inc_dels();

            let key = match parts.next() {
                Some(k) => k,
                None => return (protocol::err("DEL requires key"), false),
            };
            (protocol::integer(if store.del(key) { 1 } else { 0 }), false)
        }

        "TTL" => {
            stats.inc_ttl();

            let key = match parts.next() {
                Some(k) => k,
                None => return (protocol::err("TTL requires key"), false),
            };
            (protocol::integer(store.ttl(key)), false)
        }

        "EXPIRE" => {
            stats.inc_expire();

            let key = match parts.next() {
                Some(k) => k,
                None => return (protocol::err("EXPIRE requires key"), false),
            };
            let secs = match parts.next() {
                Some(s) => match s.parse::<u64>() {
                    Ok(x) => x,
                    Err(_) => return (protocol::err("EXPIRE requires integer seconds"), false),
                },
                None => return (protocol::err("EXPIRE requires seconds"), false),
            };

            (
                protocol::integer(if store.expire(key, secs) { 1 } else { 0 }),
                false,
            )
        }

        _ => (protocol::err("unknown command"), false),
    }
}
