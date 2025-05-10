use std::cell::RefCell;
use std::io;
use std::rc::Rc;

use error::Error;
use io_uring::{self, IoUring, cqueue, squeue};

use promise::Promise;
use registry::{RegRef, new_reg_ref};

#[rustfmt::skip]
use squeue::Entry as SQE;
use cqueue::Entry as CQE;
use cqueue::EntryMarker as CQEM;
use squeue::EntryMarker as SQEM;

pub mod multithread;

pub mod error;
pub mod promise;
pub mod pstatus;
pub mod registry;

pub type RingRef<S, C> = Rc<RefCell<IoUring<S, C>>>;
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone /* This type should be extremely cheap to clone. */)]
pub struct PIoUring<S: SQEM = SQE, C: CQEM = CQE> {
    ring: RingRef<S, C>,
    registry: RegRef<C>,
}

impl<S: SQEM, C: CQEM> PIoUring<S, C> {
    #[inline]
    pub fn new(ring: IoUring<S, C>) -> Self {
        Self {
            ring: Rc::new(RefCell::new(ring)),
            registry: new_reg_ref(),
        }
    }

    #[inline]
    fn new_promise(&mut self, entry: S) -> (Promise<S, C>, S) {
        let user_data = self.registry.borrow_mut().next_uuid();

        let entry = entry.set_user_data(user_data);
        let promise = Promise::new(user_data, self.registry.clone(), self.clone());

        (promise, entry)
    }

    #[inline]
    pub fn schedule_promise(&mut self, promise: Promise<S, C>) -> Promise<S, C> {
        self.registry.borrow_mut().schedule(promise.get_uuid());

        promise
    }

    #[inline]
    pub fn trigger_submitter(&self) -> io::Result<usize> {
        self.ring.borrow().submit()
    }

    #[inline]
    pub unsafe fn submit(&mut self, entry: S) -> Result<Promise<S, C>> {
        let (promise, entry) = self.new_promise(entry);

        unsafe { self.ring.borrow_mut().submission().push(entry) }
            .map_err(Error::from)
            .map(|()| self.schedule_promise(promise))
            .and_then(|promise| {
                self.trigger_submitter()
                    .map(|_| promise)
                    .map_err(Error::from)
            })
    }

    #[inline]
    pub unsafe fn batch_submit<T, I>(&mut self, entries: T) -> Result<Box<[Promise<S, C>]>>
    where
        I: ExactSizeIterator<Item = S>,
        T: IntoIterator<IntoIter = I>,
    {
        let (promises, entries): (Vec<_>, Vec<_>) = entries
            .into_iter()
            .map(|entry| self.new_promise(entry))
            .unzip();

        let (promises, entries) = (promises.into_boxed_slice(), entries.into_boxed_slice());

        let promises = {
            unsafe { self.ring.borrow_mut().submission().push_multiple(entries) }
                .map_err(Error::from)?;

            promises.into_iter().map(|promise| self.schedule_promise(promise)).collect::<Box<_>>()
        };

        self.trigger_submitter()?;

        Ok(promises)
    }

    #[inline]
    pub fn reap(&mut self) {
        unsafe { self.reap_shared() };
    }

    /// Safety:
    /// 
    /// Ensure that `self.registry` is not already mutably borrowed.
    #[inline]
    pub unsafe fn reap_shared(&self) {
        self.registry
            .borrow_mut()
            .batch_complete(self.ring.borrow_mut().completion());
    }
}
