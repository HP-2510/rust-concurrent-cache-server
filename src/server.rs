use crate::conn::handle_connection;
use crate::stats::Stats;
use crate::store::Store;
use tokio::net::TcpListener;

pub async fn run(addr: &str, store: Store, stats: Stats) -> anyhow::Result<()> {
    let listener = TcpListener::bind(addr).await?;
    println!("Cache server listening on {}", addr);

    loop {
        let (stream, peer) = listener.accept().await?;
        let store_clone = store.clone();
        let stats_clone = stats.clone();

        stats_clone.inc_connections();

        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream, store_clone, stats_clone).await {
                eprintln!("Connection error from {}: {:?}", peer, e);
            }
        });
    }
}
