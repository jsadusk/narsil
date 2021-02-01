#[macro_use]
use lazy_static;
use std::sync::atomic::{AtomicU64, Ordering};

lazy_static! {
    static ref NEXT_REGION_ID: AtomicU64 = AtomicU64::new(0);
}

pub fn get_next_region_id() -> u64 {
    let id = NEXT_REGION_ID.load(Ordering::Relaxed);
    NEXT_REGION_ID.store(id + 1, Ordering::Relaxed);
    id
}
