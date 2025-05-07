use std::ops::{Deref, DerefMut};

use io_uring::SubmissionQueue;

use crate::{
    CQEM, SQEM,
    promise::Promise,
    registry::{PRegRef, new_preg_ref},
};

pub struct PSubmissionQueue<'a, S: SQEM, C: CQEM> {
    sq: SubmissionQueue<'a, S>,
    registry: PRegRef<C>,
}

impl<'a, S: SQEM, C: CQEM> PSubmissionQueue<'a, S, C> {
    #[inline]
    pub fn new(sq: SubmissionQueue<'a, S>) -> Self {
        Self {
            sq,
            registry: new_preg_ref(),
        }
    }

    #[inline]
    pub fn new_with_reg(sq: SubmissionQueue<'a, S>, registry: PRegRef<C>) -> Self {
        Self { sq, registry }
    }

    #[inline]
    pub fn get_reg(&self) -> PRegRef<C> {
        self.registry.clone()
    }

    #[inline]
    pub unsafe fn push_unchecked(&mut self, entry: S) -> Promise<C> {
        let uuid = self.registry.get_uuid();

        // link SQE to CQE with uuid
        let entry = entry.set_user_data(uuid);

        unsafe { self.deref_mut().push_unchecked(entry) };

        Promise::new(uuid, self.get_reg())
    }

    #[inline]
    pub unsafe fn push_nosync(&mut self, entry: S) -> Result<Promise<C>, S> {
        if !self.is_full() {
            Ok(unsafe { self.push_unchecked(entry) })
        } else {
            Err(entry)
        }
    }

    #[inline]
    pub unsafe fn push(&mut self, entry: S) -> Result<Promise<C>, S> {
        self.sync();
        let res = unsafe { self.push_nosync(entry) };
        self.sync();

        res
    }

    #[inline]
    pub unsafe fn push_multiple_nosync<I, T>(
        &mut self,
        entries: T,
    ) -> Result<Box<[Promise<C>]>, Box<[S]>>
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
    pub unsafe fn push_multiple<I, T>(&mut self, entries: T) -> Result<Box<[Promise<C>]>, Box<[S]>>
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

impl<'a, S: SQEM, C: CQEM> Deref for PSubmissionQueue<'a, S, C> {
    type Target = SubmissionQueue<'a, S>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.sq
    }
}

impl<'a, S: SQEM, C: CQEM> DerefMut for PSubmissionQueue<'a, S, C> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.sq
    }
}
