use std::{
    collections::BTreeMap,
    sync::{
        Arc, RwLock, RwLockReadGuard, RwLockWriteGuard,
        atomic::{AtomicU64, Ordering},
    },
};

use io_uring::CompletionQueue;

use crate::CQEM;

static WERR: &'static str = "Failed to obtain write lock";
static RERR: &'static str = "Failed to obtain read lock";

pub type PRegRef<C> = Arc<PromiseRegistry<C>>;

pub fn new_preg_ref<C: CQEM>() -> PRegRef<C> {
    Arc::new(PromiseRegistry::new())
}

pub struct PromiseRegistry<C: CQEM> {
    completed: RwLock<BTreeMap<u64, C>>,
    curr_uuid: AtomicU64,
}

impl<C: CQEM> PromiseRegistry<C> {
    #[inline]
    pub fn new() -> Self {
        Self {
            completed: RwLock::new(BTreeMap::new()),
            curr_uuid: AtomicU64::new(0),
        }
    }

    #[inline]
    pub fn curr_uuid(&self) -> u64 {
        self.curr_uuid.load(Ordering::Relaxed)
    }

    #[inline]
    pub fn next_uuid(&self) -> u64 {
        self.curr_uuid.fetch_add(1, Ordering::Relaxed)
    }

    #[inline]
    pub fn get_uuid(&self) -> u64 {
        let mut uuid = self.next_uuid();

        // Ensure that this registry does not contain this id already
        while self.contains_key(&uuid) {
            uuid = self.next_uuid()
        }

        uuid
    }

    #[inline]
    fn write_lock(&self) -> RwLockWriteGuard<'_, BTreeMap<u64, C>> {
        self.completed.write().expect(WERR)
    }

    #[inline]
    fn read_lock(&self) -> RwLockReadGuard<'_, BTreeMap<u64, C>> {
        self.completed.read().expect(RERR)
    }

    #[inline]
    pub fn contains_key(&self, uuid: &u64) -> bool {
        self.read_lock().contains_key(uuid)
    }

    #[inline]
    pub fn remove(&self, uuid: &u64) -> Option<C> {
        self.write_lock().remove(uuid)
    }

    #[inline]
    pub fn insert(&self, uuid: u64, cqe: C) {
        self.write_lock().insert(uuid, cqe);
    }

    #[inline]
    pub fn extend<I: IntoIterator<Item = (u64, C)>>(&self, entries: I) {
        self.write_lock().extend(entries);
    }

    #[inline]
    pub fn reap(&self, cq: &mut CompletionQueue<'_, C>) -> usize {
        cq.sync();

        // Greedily aggregate (user_data, entry) pairs to minimize write lock lifetime.
        let v = cq.map(|e| (e.user_data(), e)).collect::<Vec<_>>();

        cq.sync();

        let reaped = v.len();

        self.extend(v);

        reaped
    }
}
