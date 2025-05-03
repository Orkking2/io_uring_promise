pub mod pcqueue;
pub mod promise;
pub mod psqueue;
pub mod registry;
pub mod util;

#[rustfmt::skip]
use squeue::Entry as SQE;
use cqueue::Entry as CQE;
use cqueue::EntryMarker as CQEM;
use squeue::EntryMarker as SQEM;

use io_uring::Submitter;
use pcqueue::PCompletionQueue;
use psqueue::PSubmissionQueue;
use util::get_uuid;

use std::{
    io,
    ops::{Deref, DerefMut},
};

use io_uring::{IoUring, cqueue, squeue};

use crate::registry::{PRegRef, new_preg_ref};

pub struct PromiseIoUring<S: SQEM = SQE, C: CQEM = CQE> {
    uring: IoUring<S, C>,
    registry: PRegRef<C>,
}

impl PromiseIoUring<SQE, CQE> {
    pub fn new(entries: u32) -> io::Result<Self> {
        IoUring::new(entries).map(<Self as From<IoUring<SQE, CQE>>>::from)
    }
}

impl<S: SQEM, C: CQEM> PromiseIoUring<S, C> {
    pub fn get_reg(&self) -> PRegRef<C> {
        self.registry.clone()
    }

    #[must_use]
    pub fn builder() -> PBuilder<S, C> {
        IoUring::builder().into()
    }

    #[inline]
    pub fn split(
        &mut self,
    ) -> (
        Submitter<'_>,
        PSubmissionQueue<'_, S, C>,
        PCompletionQueue<'_, C>,
    ) {
        let sr = self.get_reg();
        let cr = self.get_reg();

        let (s, sq, cq) = self.deref_mut().split();

        (
            s,
            PSubmissionQueue::new(sq, sr),
            PCompletionQueue::new(cq, cr),
        )
    }

    #[inline]
    pub fn submission(&mut self) -> PSubmissionQueue<'_, S, C> {
        unsafe { self.submission_shared() }
    }

    #[inline]
    pub unsafe fn submission_shared(&self) -> PSubmissionQueue<'_, S, C> {
        PSubmissionQueue::new(unsafe { self.deref().submission_shared() }, self.get_reg())
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

    fn deref(&self) -> &Self::Target {
        &self.uring
    }
}

impl<S: SQEM, C: CQEM> DerefMut for PromiseIoUring<S, C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.uring
    }
}

impl<S: SQEM, C: CQEM> Into<IoUring<S, C>> for PromiseIoUring<S, C> {
    fn into(self) -> IoUring<S, C> {
        self.uring
    }
}

impl<S: SQEM, C: CQEM> From<IoUring<S, C>> for PromiseIoUring<S, C> {
    fn from(uring: IoUring<S, C>) -> Self {
        Self {
            uring,
            registry: new_preg_ref(),
        }
    }
}

#[derive(Clone, Default)]
pub struct PBuilder<S: SQEM = SQE, C: CQEM = CQE> {
    inner: io_uring::Builder<S, C>,
}

impl<S: SQEM, C: CQEM> PBuilder<S, C> {
    pub fn build(&self, entries: u32) -> io::Result<PromiseIoUring<S, C>> {
        self.inner
            .build(entries)
            .map(<PromiseIoUring<S, C> as From<IoUring<S, C>>>::from)
    }
}

impl<S: SQEM, C: CQEM> Deref for PBuilder<S, C> {
    type Target = io_uring::Builder<S, C>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<S: SQEM, C: CQEM> DerefMut for PBuilder<S, C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<S: SQEM, C: CQEM> From<io_uring::Builder<S, C>> for PBuilder<S, C> {
    fn from(inner: io_uring::Builder<S, C>) -> Self {
        Self { inner }
    }
}

impl<S: SQEM, C: CQEM> Into<io_uring::Builder<S, C>> for PBuilder<S, C> {
    fn into(self) -> io_uring::Builder<S, C> {
        self.inner
    }
}
