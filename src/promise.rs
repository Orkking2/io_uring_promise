use std::fmt::{Debug, Display};

use crate::{CQEM, PIoUring, SQEM, pstatus::PromiseStatus, registry::RegRef};

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
    pub fn poll(&self) -> PromiseStatus {
        unsafe {
            self.trigger_reap();
        }

        self.reg_ref.borrow().get_status(&self.uuid)
    }

    #[inline]
    pub fn try_wait(self) -> Result<C, PromiseError<S, C>> {
        unsafe {
            self.trigger_reap();
        }

        { self.reg_ref.borrow_mut().remove(&self.uuid) }.map_err(|err| PromiseError::new(err, self))
    }
}

impl<S: SQEM, C: CQEM> Debug for Promise<S, C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Promise").field("uuid", &self.uuid).finish()
    }
}

#[derive(Debug)]
pub struct PromiseError<S: SQEM, C: CQEM> {
    pub status: PromiseStatus,
    pub promise: Promise<S, C>,
}

impl<S: SQEM, C: CQEM> PromiseError<S, C> {
    #[inline]
    pub fn new(status: PromiseStatus, promise: Promise<S, C>) -> Self {
        Self { status, promise }
    }

    #[inline]
    pub fn get_promise(self) -> Promise<S, C> {
        self.promise
    }
}

impl<S: SQEM, C: CQEM> Into<Promise<S, C>> for PromiseError<S, C> {
    fn into(self) -> Promise<S, C> {
        self.get_promise()
    }
}

impl<S: SQEM, C: CQEM> Display for PromiseError<S, C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "promise has status {} instead of `Complete`",
            &self.status
        )
    }
}
