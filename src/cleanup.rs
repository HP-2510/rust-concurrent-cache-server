use crate::stats::Stats;
use crate::store::Store;
use std::time::Duration;

pub fn spawn_janitor(store: Store, stats: Stats, interval_ms: u64) {
    tokio::spawn(async move {
        let interval = Duration::from_millis(interval_ms.max(50));
        loop {
            tokio::time::sleep(interval).await;
            let removed = store.remove_expired_batch();
            if removed > 0 {
                stats.add_expired_removed(removed as u64);
            }
        }
    });
}
