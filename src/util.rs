use std::sync::atomic::{AtomicU64, Ordering};

static CURR_UUID: AtomicU64 = AtomicU64::new(0);

pub(crate) fn get_uuid() -> u64 {
    CURR_UUID.fetch_add(1, Ordering::Relaxed)
}
