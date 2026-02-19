mod cleanup;
mod conn;
mod protocol;
mod server;
mod stats;
mod store;

use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long, default_value = "127.0.0.1")]
    host: String,
    #[arg(long, default_value_t = 6379)]
    port: u16,
    #[arg(long, default_value_t = 500)]
    cleanup_interval_ms: u64,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let addr = format!("{}:{}", args.host, args.port);

    let store = store::Store::new();
    let stats = stats::Stats::new();

    cleanup::spawn_janitor(store.clone(), stats.clone(), args.cleanup_interval_ms);
    server::run(&addr, store, stats).await
}
