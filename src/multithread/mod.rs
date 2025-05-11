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
        fn handle_reap<S: SQEM, C: CQEM>(ring: &mut IoUring<S, C>, reg_ref: &RegRef<C>) {
            reg_ref
                .write()
                .expect(WERR)
                .batch_complete(ring.completion());
        }

        fn handle_entry<S: SQEM, C: CQEM>(ring: &mut IoUring<S, C>, reg_ref: &RegRef<C>, user_data: u64, entry: S) {
            loop {
                if unsafe { ring.submission().push(entry.clone()) }.is_ok() {
                    ring.submit().unwrap();
                    reg_ref.write().expect(WERR).schedule(user_data);
                    break;
                } else {
                    // Maybe the CQ is full because the SQ is full
                    handle_reap(ring, reg_ref);
                }
            }
        }

        // And then in your closure:
        move || {
            for signal in receiver.into_iter() {
                match signal {
                    Signal::Entry(user_data, entry) => {
                        handle_entry(&mut ring, &reg_ref, user_data, entry);
                    }
                    Signal::Reap => {
                        handle_reap(&mut ring, &reg_ref);
                    }
                }
            }
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

        self.send(Signal::Entry(promise.get_uuid(), entry));

        promise
    }

    #[inline]
    pub unsafe fn batch_submit<T, I>(&self, entries: T) -> Box<[Promise<S, C>]>
    where
        I: ExactSizeIterator<Item = S>,
        T: IntoIterator<IntoIter = I>,
    {
        entries
            .into_iter()
            .map(|entry| unsafe { self.submit(entry) })
            .collect::<Box<_>>()
    }
}
