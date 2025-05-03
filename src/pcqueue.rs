use std::ops::{Deref, DerefMut};

use io_uring::CompletionQueue;

use crate::{CQEM, registry::PRegRef};

pub struct PCompletionQueue<'a, C: CQEM> {
    cq: CompletionQueue<'a, C>,
    registry: PRegRef<C>,
}

impl<'a, C: CQEM> PCompletionQueue<'a, C> {
    pub fn new(cq: CompletionQueue<'a, C>, registry: PRegRef<C>) -> Self {
        Self { cq, registry }
    }

    pub fn reap(&mut self) -> usize {
        self.registry.reap(&mut self.cq)
    }
}

impl<'a, C: CQEM> Deref for PCompletionQueue<'a, C> {
    type Target = CompletionQueue<'a, C>;

    fn deref(&self) -> &Self::Target {
        &self.cq
    }
}

impl<'a, C: CQEM> DerefMut for PCompletionQueue<'a, C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.cq
    }
}