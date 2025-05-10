use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
};

use crate::{CQEM, SQEM, cqreaper::RTWaker, psqueue::PSubmissionQueue, rpromise::RPromise};

// Move pushing operation to Reaper thread since we have a waker for that here anyway.
// Give the RSQueue

pub struct RSQueue<'a, S: SQEM, C: CQEM> {
    sq: PSubmissionQueue<'a, S, C>,
    waker: Arc<RTWaker>,
}

impl<'a, S: SQEM, C: CQEM> RSQueue<'a, S, C> {
    #[inline]
    pub fn new(sq: PSubmissionQueue<'a, S, C>, waker: Arc<RTWaker>) -> Self {
        Self { sq, waker }
    }

    pub fn get_waker(&self) -> Arc<RTWaker> {
        self.waker.clone()
    }

    pub unsafe fn push_unchecked(&mut self, entry: S) -> RPromise<C> {
        RPromise::new(
            unsafe { self.deref_mut().push_unchecked(entry) },
            self.get_waker(),
        )
    }

    #[inline]
    pub unsafe fn push_nosync(&mut self, entry: S) -> Result<RPromise<C>, S> {
        if !self.is_full() {
            Ok(unsafe { self.push_unchecked(entry) })
        } else {
            Err(entry)
        }
    }

    #[inline]
    pub unsafe fn push(&mut self, entry: S) -> Result<RPromise<C>, S> {
        self.sync();
        let res = unsafe { self.push_nosync(entry) };
        self.sync();

        res
    }

    #[inline]
    pub unsafe fn push_multiple_nosync<I, T>(
        &mut self,
        entries: T,
    ) -> Result<Box<[RPromise<C>]>, Box<[S]>>
    where
        I: ExactSizeIterator<Item = S>,
        T: IntoIterator<Item = S, IntoIter = I>,
    {
        let iter = entries.into_iter();

        if self.capacity() - self.len() < iter.len() {
            return Err(iter.collect::<Box<_>>());
        }

        Ok(iter
            .map(|entry| unsafe { self.push_unchecked(entry) })
            .collect::<Box<_>>())
    }

    #[inline]
    pub unsafe fn push_multiple<I, T>(&mut self, entries: T) -> Result<Box<[RPromise<C>]>, Box<[S]>>
    where
        I: ExactSizeIterator<Item = S>,
        T: IntoIterator<Item = S, IntoIter = I>,
    {
        self.sync();
        let res = unsafe { self.push_multiple_nosync(entries) };
        self.sync();

        res
    }
}

impl<'a, S: SQEM, C: CQEM> Deref for RSQueue<'a, S, C> {
    type Target = PSubmissionQueue<'a, S, C>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.sq
    }
}

impl<'a, S: SQEM, C: CQEM> DerefMut for RSQueue<'a, S, C> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.sq
    }
}
