use crate::{CQEM, registry::PRegRef};

// 
pub struct Promise<C: CQEM> {
    pub(crate) uuid: u64,
    pub(crate) registry: PRegRef<C>,
}

impl<C: CQEM> Promise<C> {
    pub fn new(uuid: u64, registry: PRegRef<C>) -> Self {
        Self { uuid, registry }
    }

    pub fn poll(&self) -> bool {
        self.registry.contains_key(&self.uuid)
    }

    pub fn try_wait(self) -> Result<C, Self> {
        match self.registry.remove(&self.uuid) {
            Some(entry) => Ok(entry),
            None => Err(self),
        }
    }
}