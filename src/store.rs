use bytes::Bytes;
use dashmap::DashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct Store {
    inner: Arc<DashMap<String, Entry>>,
}

#[derive(Clone)]
struct Entry {
    value: Bytes,
    expires_at: Option<Instant>,
}

impl Store {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(DashMap::new()),
        }
    }

    pub fn set(&self, key: String, value: Bytes, ex_seconds: Option<u64>) {
        let expires_at = ex_seconds.map(|s| Instant::now() + Duration::from_secs(s));
        self.inner.insert(key, Entry { value, expires_at });
    }

    pub fn get(&self, key: &str) -> Option<Bytes> {
        if let Some(entry) = self.inner.get(key) {
            if is_expired(entry.expires_at) {
                drop(entry);
                self.inner.remove(key);
                return None;
            }
            return Some(entry.value.clone());
        }
        None
    }

    pub fn del(&self, key: &str) -> bool {
        self.inner.remove(key).is_some()
    }

    pub fn expire(&self, key: &str, ex_seconds: u64) -> bool {
        if let Some(mut entry) = self.inner.get_mut(key) {
            entry.expires_at = Some(Instant::now() + Duration::from_secs(ex_seconds));
            return true;
        }
        false
    }

    /// Semantics are as such:
    /// -2 key doesn't exist
    /// -1 exists but no expiry
    /// >=0 seconds remaining otherwise
    pub fn ttl(&self, key: &str) -> i64 {
        match self.inner.get(key) {
            None => -2,
            Some(entry) => {
                if is_expired(entry.expires_at) {
                    drop(entry);
                    self.inner.remove(key);
                    return -2;
                }
                match entry.expires_at {
                    None => -1,
                    Some(t) => {
                        let now = Instant::now();
                        if t <= now {
                            -2
                        } else {
                            t.duration_since(now).as_secs() as i64
                        }
                    }
                }
            }
        }
    }

    pub fn remove_expired_batch(&self) -> usize {
        let mut expired = Vec::new();
        for item in self.inner.iter() {
            if is_expired(item.expires_at) {
                expired.push(item.key().clone());
            }
        }
        let n = expired.len();
        for k in expired {
            self.inner.remove(&k);
        }
        n
    }
}

fn is_expired(expires_at: Option<Instant>) -> bool {
    matches!(expires_at, Some(t) if t <= Instant::now())
}
