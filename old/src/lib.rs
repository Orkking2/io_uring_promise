pub mod cqreaper;
pub mod pcqueue;
pub mod promise;
pub mod psqueue;
pub mod registry;
pub mod rpromise;
pub mod rsqueue;

#[rustfmt::skip]
use squeue::Entry as SQE;
use cqueue::Entry as CQE;
use cqueue::EntryMarker as CQEM;
use squeue::EntryMarker as SQEM;

use io_uring::Submitter;
use pcqueue::PCompletionQueue;
use psqueue::PSubmissionQueue;

use std::{
    io,
    ops::{Deref, DerefMut},
};

use io_uring::{IoUring, cqueue, squeue};

use crate::registry::{PRegRef, new_preg_ref};

pub struct PromiseIoUring<S: SQEM = SQE, C: CQEM = CQE> {
    ring: IoUring<S, C>,
    registry: PRegRef<C>,
}

impl PromiseIoUring<SQE, CQE> {
    #[inline]
    pub fn new(entries: u32) -> io::Result<Self> {
        IoUring::new(entries).map(<Self as From<IoUring<SQE, CQE>>>::from)
    }
}

impl<S: SQEM, C: CQEM> PromiseIoUring<S, C> {
    #[inline]
    pub fn get_reg(&self) -> PRegRef<C> {
        self.registry.clone()
    }

    #[inline]
    #[must_use]
    pub fn builder() -> PBuilder<S, C> {
        IoUring::builder().into()
    }

    #[inline]
    pub fn reap(&mut self) -> usize {
        self.completion().reap()
    }

    #[inline]
    pub fn split(
        &mut self,
    ) -> (
        Submitter<'_>,
        PSubmissionQueue<'_, S, C>,
        PCompletionQueue<'_, C>,
    ) {
        (
            self.submitter(),
            unsafe { self.submission_shared() },
            unsafe { self.completion_shared() },
        )
    }

    #[inline]
    pub fn submission(&mut self) -> PSubmissionQueue<'_, S, C> {
        unsafe { self.submission_shared() }
    }

    #[inline]
    pub unsafe fn submission_shared(&self) -> PSubmissionQueue<'_, S, C> {
        PSubmissionQueue::new_with_reg(unsafe { self.deref().submission_shared() }, self.get_reg())
    }

    #[inline]
    pub fn completion(&mut self) -> PCompletionQueue<'_, C> {
        unsafe { self.completion_shared() }
    }

    #[inline]
    pub unsafe fn completion_shared(&self) -> PCompletionQueue<'_, C> {
        PCompletionQueue::new(unsafe { self.deref().completion_shared() }, self.get_reg())
    }
}

impl<S: SQEM, C: CQEM> Deref for PromiseIoUring<S, C> {
    type Target = IoUring<S, C>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.ring
    }
}

impl<S: SQEM, C: CQEM> DerefMut for PromiseIoUring<S, C> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.ring
    }
}

impl<S: SQEM, C: CQEM> Into<IoUring<S, C>> for PromiseIoUring<S, C> {
    #[inline]
    fn into(self) -> IoUring<S, C> {
        self.ring
    }
}

impl<S: SQEM, C: CQEM> From<IoUring<S, C>> for PromiseIoUring<S, C> {
    #[inline]
    fn from(uring: IoUring<S, C>) -> Self {
        Self {
            ring: uring,
            registry: new_preg_ref(),
        }
    }
}

#[derive(Clone, Default)]
pub struct PBuilder<S: SQEM = SQE, C: CQEM = CQE> {
    inner: io_uring::Builder<S, C>,
}

impl<S: SQEM, C: CQEM> PBuilder<S, C> {
    #[inline]
    pub fn build(&self, entries: u32) -> io::Result<PromiseIoUring<S, C>> {
        self.inner
            .build(entries)
            .map(<PromiseIoUring<S, C> as From<IoUring<S, C>>>::from)
    }
}

impl<S: SQEM, C: CQEM> Deref for PBuilder<S, C> {
    type Target = io_uring::Builder<S, C>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<S: SQEM, C: CQEM> DerefMut for PBuilder<S, C> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<S: SQEM, C: CQEM> From<io_uring::Builder<S, C>> for PBuilder<S, C> {
    #[inline]
    fn from(inner: io_uring::Builder<S, C>) -> Self {
        Self { inner }
    }
}

impl<S: SQEM, C: CQEM> Into<io_uring::Builder<S, C>> for PBuilder<S, C> {
    #[inline]
    fn into(self) -> io_uring::Builder<S, C> {
        self.inner
    }
}
