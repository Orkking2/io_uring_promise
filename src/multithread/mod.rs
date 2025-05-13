use std::{
    sync::{
        RwLockWriteGuard,
        mpsc::{Receiver, Sender, channel},
    },
    thread,
};

use io_uring::IoUring;
use promise::Promise;
use registry::{RegRef, new_reg_ref};
use signal::Signal;

use crate::{CQE, CQEM, SQE, SQEM, registry::PromiseRegistry};

pub mod promise;
pub mod registry;
pub mod signal;

pub(crate) static WERR: &'static str = "Error obtaining write lock";
pub(crate) static RERR: &'static str = "Error obtaining read lock";

#[derive(Clone)]
pub struct PIoUring<S: SQEM = SQE, C: CQEM = CQE> {
    sender: Sender<Signal<S>>,
    reg_ref: RegRef<C>,
}

impl<S: SQEM + 'static, C: CQEM + 'static> PIoUring<S, C> {
    #[inline]
    pub fn new(ring: IoUring<S, C>) -> Self {
        let (sender, receiver) = channel();
        let reg_ref = new_reg_ref();

        // This thread naturally joins when the last sender is dropped.
        thread::spawn(Self::thread_fn_generator(ring, receiver, reg_ref.clone()));

        Self { sender, reg_ref }
    }

    #[inline]
    fn thread_fn_generator(
        mut ring: IoUring<S, C>,
        receiver: Receiver<Signal<S>>,
        reg_ref: RegRef<C>,
    ) -> impl FnOnce() -> () {
        move || {
            let reap = |ring: &mut IoUring<S, C>, reg_ref: &RegRef<C>| {
                reg_ref
                    .write()
                    .expect(WERR)
                    .batch_complete(ring.completion());
            };

            // Blocks when there are no `Signal`s to consume.
            for signal in receiver {
                match signal {
                    Signal::Entry(entry) => {
                        // Loops until submission of entry is successful.
                        loop {
                            // Fails if the SQ is full, possible if we are handed a ring with a full SQ or
                            // we have been pushing SQEs and not reaping their CQEs.
                            if unsafe { ring.submission().push(entry.clone()) }.is_ok() {
                                // Inform the kernel of our new submission.
                                ring.submit().unwrap();
                                // Schedule the promise.
                                reg_ref.write().expect(WERR).schedule(entry.get_user_data());
                                break;
                            } else {
                                // The SQ could be full because the CQ is full.
                                reap(&mut ring, &reg_ref);
                                // CQ is now empty, so we should wake the kernel.
                                ring.submit().unwrap();
                            }
                        }
                    }
                    Signal::Reap => {
                        reap(&mut ring, &reg_ref);
                    }
                }
            }
            // Thread joins when receiver produces a `None`, which happens when the last sender (PIoUring) gets dropped.
            // This means we don't actually have to keep track of this thread at all, it will take care of itself.
        }
    }

    #[inline]
    fn reg_write_lock(&self) -> RwLockWriteGuard<'_, PromiseRegistry<C>> {
        self.reg_ref.write().expect(WERR)
    }

    #[inline]
    fn new_promise(&self, entry: S) -> (Promise<S, C>, S) {
        let user_data = self.reg_write_lock().next_uuid();

        let entry = entry.set_user_data(user_data);
        let promise = Promise::new(user_data, self.reg_ref.clone(), self.clone());

        (promise, entry)
    }

    #[inline]
    pub fn send(&self, signal: Signal<S>) {
        self.sender.send(signal).unwrap();
    }

    #[inline]
    pub fn reap(&self) {
        self.send(Signal::Reap);
    }

    #[inline]
    pub unsafe fn submit(&self, entry: S) -> Promise<S, C> {
        let (promise, entry) = self.new_promise(entry);

        self.send(Signal::Entry(entry));

        promise
    }

    #[inline]
    pub unsafe fn batch_submit<I>(&self, entries: I) -> Box<[Promise<S, C>]>
    where
        I: IntoIterator<Item = S>,
    {
        entries
            .into_iter()
            .map(|entry| unsafe { self.submit(entry) })
            .collect::<Box<_>>()
    }
}
