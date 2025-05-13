use std::fmt::Debug;

use crate::{CQEM, Result, SQEM, error::Error, pstatus::PromiseStatus};

use super::{PIoUring, RERR, WERR, registry::RegRef};

pub struct Promise<S: SQEM + 'static, C: CQEM + 'static> {
    sender: PIoUring<S, C>,
    reg_ref: RegRef<C>,
    uuid: u64,
}

impl<S: SQEM, C: CQEM> Promise<S, C> {
    #[inline]
    pub fn new(uuid: u64, reg_ref: RegRef<C>, sender: PIoUring<S, C>) -> Self {
        Self {
            sender,
            reg_ref,
            uuid,
        }
    }

    #[inline]
    pub fn get_uuid(&self) -> u64 {
        self.uuid
    }

    #[inline]
    pub fn trigger_reap(&self) {
        self.sender.reap();
    }

    #[inline]
    pub fn status(&self) -> PromiseStatus {
        self.trigger_reap();

        self.reg_ref.read().expect(RERR).get_status(&self.uuid)
    }

    #[inline]
    pub fn try_wait(&mut self) -> Result<C> {
        self.trigger_reap();

        { self.reg_ref.write().expect(WERR).remove(&self.uuid) }.map_err(Error::from)
    }
}

impl<S: SQEM, C: CQEM> Debug for Promise<S, C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Promise").field("uuid", &self.uuid).finish()
    }
}
