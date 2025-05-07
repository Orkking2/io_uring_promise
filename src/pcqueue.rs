use std::ops::{Deref, DerefMut};

use io_uring::CompletionQueue;

use crate::{CQEM, registry::PRegRef};

/// This can be created from a normal IoUring's SubmissionQueue!
/// Also comes from the PromiseIoUring's `submission`, `submission_shared`, or `split` functions.
pub struct PCompletionQueue<'a, C: CQEM> {
    cq: CompletionQueue<'a, C>,
    registry: PRegRef<C>,
}

impl<'a, C: CQEM> PCompletionQueue<'a, C> {
    #[inline]
    pub fn new(cq: CompletionQueue<'a, C>, registry: PRegRef<C>) -> Self {
        Self { cq, registry }
    }

    #[inline]
    pub fn reap(&mut self) -> usize {
        self.registry.reap(&mut self.cq)
    }
}

impl<'a, C: CQEM> Deref for PCompletionQueue<'a, C> {
    type Target = CompletionQueue<'a, C>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.cq
    }
}

impl<'a, C: CQEM> DerefMut for PCompletionQueue<'a, C> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.cq
    }
}

impl<'a, C: CQEM> Into<CompletionQueue<'a, C>> for PCompletionQueue<'a, C> {
    #[inline]
    fn into(self) -> CompletionQueue<'a, C> {
        self.cq
    }
}

unsafe impl<'a, C: CQEM> Send for PCompletionQueue<'a, C> {}
