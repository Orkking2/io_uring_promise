use std::ops::{Deref, DerefMut};

use io_uring::SubmissionQueue;

use crate::{CQEM, SQEM, get_uuid, promise::Promise, registry::PRegRef};

pub struct PSubmissionQueue<'a, S: SQEM, C: CQEM> {
    sq: SubmissionQueue<'a, S>,
    registry: PRegRef<C>,
}

impl<'a, S: SQEM, C: CQEM> PSubmissionQueue<'a, S, C> {
    pub fn new(sq: SubmissionQueue<'a, S>, registry: PRegRef<C>) -> Self {
        Self { sq, registry }
    }

    pub fn get_reg(&self) -> PRegRef<C> {
        self.registry.clone()
    }

    pub unsafe fn push_unchecked(&mut self, entry: S) -> Promise<C> {
        let mut uuid = get_uuid();

        // Ensure no collisions -- will busy loop forever if registry is full
        while self.registry.contains_key(&uuid) {
            uuid = get_uuid();
        }

        // link SQE to CQE with uuid
        let entry = entry.set_user_data(uuid);

        unsafe { self.deref_mut().push_unchecked(entry) };

        Promise::new(uuid, self.get_reg())
    }

    #[inline]
    pub unsafe fn push(&mut self, entry: S) -> Result<Promise<C>, S> {
        if !self.is_full() {
            Ok(unsafe { self.push_unchecked(entry) })
        } else {
            Err(entry)
        }
    }

    #[inline]
    pub unsafe fn push_multiple(
        &mut self,
        entries: Box<[S]>,
    ) -> Result<Box<[Promise<C>]>, Box<[S]>> {
        if self.capacity() - self.len() < entries.len() {
            return Err(entries);
        }

        Ok(entries
            .into_iter()
            .map(|entry| unsafe { self.push_unchecked(entry) })
            .collect::<Box<_>>())
    }
}

impl<'a, S: SQEM, C: CQEM> Deref for PSubmissionQueue<'a, S, C> {
    type Target = SubmissionQueue<'a, S>;

    fn deref(&self) -> &Self::Target {
        &self.sq
    }
}

impl<'a, S: SQEM, C: CQEM> DerefMut for PSubmissionQueue<'a, S, C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.sq
    }
}
