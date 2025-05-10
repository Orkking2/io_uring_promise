use crate::{CQEM, registry::PRegRef};

pub struct Promise<C: CQEM> {
    registry: PRegRef<C>,
    uuid: u64,
}

impl<C: CQEM> Promise<C> {
    #[inline]
    pub fn new(uuid: u64, registry: PRegRef<C>) -> Self {
        Self { uuid, registry }
    }

    #[inline]
    pub fn poll(&self) -> bool {
        self.registry.contains_key(&self.uuid)
    }

    #[inline]
    pub fn try_wait(self) -> Result<C, Self> {
        match self.registry.remove(&self.uuid) {
            Some(entry) => Ok(entry),
            None => Err(self),
        }
    }
}

unsafe impl<C: CQEM> Send for Promise<C> {}
unsafe impl<C: CQEM> Sync for Promise<C> {}
