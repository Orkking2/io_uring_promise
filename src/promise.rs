use std::fmt::Debug;

use crate::{CQEM, PIoUring, Result, SQEM, error::Error, pstatus::PromiseStatus, registry::RegRef};

pub struct Promise<S: SQEM, C: CQEM> {
    ring_ref: PIoUring<S, C>,
    reg_ref: RegRef<C>,
    uuid: u64,
}

impl<S: SQEM, C: CQEM> Promise<S, C> {
    #[inline]
    pub fn new(user_data: u64, reg_ref: RegRef<C>, ring_ref: PIoUring<S, C>) -> Self {
        Self {
            uuid: user_data,
            ring_ref,
            reg_ref,
        }
    }

    #[inline]
    pub fn get_uuid(&self) -> u64 {
        self.uuid
    }

    /// Safety:
    ///
    /// Do not use in conjunction with any borrows of the `PIoUring` that created this.
    #[inline]
    pub unsafe fn trigger_reap(&self) {
        unsafe { self.ring_ref.reap_shared() };
    }

    #[inline]
    pub fn status(&self) -> PromiseStatus {
        unsafe {
            self.trigger_reap();
        }

        self.reg_ref.borrow().get_status(&self.uuid)
    }

    #[inline]
    pub fn try_wait(&mut self) -> Result<C> {
        unsafe {
            self.trigger_reap();
        }

        { self.reg_ref.borrow_mut().remove(&self.uuid) }.map_err(Error::from)
    }
}

impl<S: SQEM, C: CQEM> Debug for Promise<S, C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Promise").field("uuid", &self.uuid).finish()
    }
}
