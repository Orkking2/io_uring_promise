use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
};

use crate::{CQEM, cqreaper::RTWaker, promise::Promise};

pub struct RPromise<C: CQEM> {
    waker: Arc<RTWaker>,
    inner: Promise<C>,
}

impl<C: CQEM> RPromise<C> {
    #[inline]
    pub fn new(inner: Promise<C>, waker: Arc<RTWaker>) -> Self {
        Self { inner, waker }
    }

    #[inline]
    pub fn wake(&self) {
        self.waker.wake();
    }
}

impl<C: CQEM> Deref for RPromise<C> {
    type Target = Promise<C>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.wake();

        &self.inner
    }
}

impl<C: CQEM> DerefMut for RPromise<C> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.wake();

        &mut self.inner
    }
}

impl<C: CQEM> Into<Promise<C>> for RPromise<C> {
    fn into(self) -> Promise<C> {
        self.inner
    }
}
