use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone, Default)]
pub struct Stats {
    inner: Arc<Inner>,
}

#[derive(Default)]
struct Inner {
    connections: AtomicU64,
    commands: AtomicU64,
    gets: AtomicU64,
    hits: AtomicU64,
    misses: AtomicU64,
    sets: AtomicU64,
    dels: AtomicU64,
    ttl: AtomicU64,
    expire: AtomicU64,
    expired_removed: AtomicU64,
}

impl Stats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn inc_connections(&self) {
        self.inner.connections.fetch_add(1, Ordering::Relaxed);
    }
    pub fn inc_commands(&self) {
        self.inner.commands.fetch_add(1, Ordering::Relaxed);
    }
    pub fn inc_gets(&self) {
        self.inner.gets.fetch_add(1, Ordering::Relaxed);
    }
    pub fn inc_hits(&self) {
        self.inner.hits.fetch_add(1, Ordering::Relaxed);
    }
    pub fn inc_misses(&self) {
        self.inner.misses.fetch_add(1, Ordering::Relaxed);
    }
    pub fn inc_sets(&self) {
        self.inner.sets.fetch_add(1, Ordering::Relaxed);
    }
    pub fn inc_dels(&self) {
        self.inner.dels.fetch_add(1, Ordering::Relaxed);
    }
    pub fn inc_ttl(&self) {
        self.inner.ttl.fetch_add(1, Ordering::Relaxed);
    }
    pub fn inc_expire(&self) {
        self.inner.expire.fetch_add(1, Ordering::Relaxed);
    }
    pub fn add_expired_removed(&self, n: u64) {
        self.inner.expired_removed.fetch_add(n, Ordering::Relaxed);
    }

    pub fn render(&self) -> String {
        let o = Ordering::Relaxed;
        format!(
            "+STATS\nconnections={}\ncommands={}\ngets={}\nhits={}\nmisses={}\nsets={}\ndels={}\nttl={}\nexpire={}\nexpired_removed={}\n",
            self.inner.connections.load(o),
            self.inner.commands.load(o),
            self.inner.gets.load(o),
            self.inner.hits.load(o),
            self.inner.misses.load(o),
            self.inner.sets.load(o),
            self.inner.dels.load(o),
            self.inner.ttl.load(o),
            self.inner.expire.load(o),
            self.inner.expired_removed.load(o),
        )
    }
}
