use std::{
    collections::BTreeMap,
    sync::{Arc, RwLock},
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
    inner: RwLock<BTreeMap<u64, C>>,
}

impl<C: CQEM> PromiseRegistry<C> {
    #[inline]
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(BTreeMap::new()),
        }
    }

    #[inline]
    pub fn contains_key(&self, uuid: &u64) -> bool {
        self.inner.read().expect(RERR).contains_key(uuid)
    }

    #[inline]
    pub fn remove(&self, uuid: &u64) -> Option<C> {
        self.inner.write().expect(WERR).remove(uuid)
    }

    #[inline]
    pub fn insert(&self, uuid: u64, cqe: C) {
        self.inner.write().expect(WERR).insert(uuid, cqe);
    }

    #[inline]
    pub fn reap(&self, cq: &mut CompletionQueue<'_, C>) -> usize {
        cq.sync();

        // Greedily aggregate (user_data, entry) pairs to minimize write lock lifetime.
        let v = cq.into_iter()
            .map(|e| (e.user_data(), e))
            .collect::<Vec<_>>();

        let reaped = v.len();

        self.inner.write().expect(WERR).extend(v);

        reaped
    }
}
