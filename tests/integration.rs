use cache_server::{cleanup, server, stats::Stats, store::Store};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

async fn start_server() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);

    let addr = format!("127.0.0.1:{}", port);

    let store = Store::new();
    let stats = Stats::new();
    cleanup::spawn_janitor(store.clone(), stats.clone(), 50);

    tokio::spawn(async move {
        let _ = server::run(&addr, store, stats).await;
    });

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    port
}

async fn send(port: u16, payload: &str) -> String {
    let mut s = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
    s.write_all(payload.as_bytes()).await.unwrap();

    tokio::time::sleep(std::time::Duration::from_millis(30)).await;

    let mut buf = vec![0u8; 16384];
    let n = s.read(&mut buf).await.unwrap();
    String::from_utf8_lossy(&buf[..n]).to_string()
}

#[tokio::test]
async fn set_get_roundtrip() {
    let port = start_server().await;
    let out = send(port, "SET a hello\nGET a\n").await;
    assert!(out.contains("+OK"));
    assert!(out.contains("$5"));
    assert!(out.contains("hello"));
}

#[tokio::test]
async fn del_removes() {
    let port = start_server().await;
    let out = send(port, "SET a hello\nDEL a\nGET a\n").await;
    assert!(out.contains("+OK"));
    assert!(out.contains(":1"));
    assert!(out.contains("$-1"));
}

#[tokio::test]
async fn expire_and_ttl() {
    let port = start_server().await;
    let out = send(port, "SET x value EX 1\nGET x\nTTL x\n").await;
    assert!(out.contains("+OK"));
    assert!(out.contains("value"));
    assert!(out.contains(":0") || out.contains(":1"));

    tokio::time::sleep(std::time::Duration::from_millis(1200)).await;

    let out2 = send(port, "GET x\nTTL x\n").await;
    assert!(out2.contains("$-1"));
    assert!(out2.contains(":-2"));
}

#[tokio::test]
async fn stats_command_works() {
    let port = start_server().await;
    let out = send(port, "PING\nGET missing\nSTATS\n").await;
    assert!(out.contains("+PONG"));
    assert!(out.contains("+STATS"));
    assert!(out.contains("commands="));
}

#[tokio::test]
async fn concurrent_clients_smoke() {
    let port = start_server().await;

    let mut tasks = Vec::new();
    for i in 0..20 {
        tasks.push(tokio::spawn(async move {
            let key = format!("k{}", i);
            let cmd = format!("SET {} v{}\nGET {}\n", key, i, key);
            let out = send(port, &cmd).await;
            assert!(out.contains("+OK"));
            assert!(out.contains("$2") || out.contains("$3"));
        }));
    }

    for t in tasks {
        t.await.unwrap();
    }
}
