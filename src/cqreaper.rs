use std::{
    sync::{
        Arc, Condvar, Mutex, MutexGuard,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, Scope, ScopedJoinHandle},
    time::{Duration, Instant, SystemTime},
};

use io_uring::CompletionQueue;

use crate::{CQEM, pcqueue::PCompletionQueue};

pub struct CQReaper<'s, 'a, C: CQEM> {
    thread: Option<ScopedJoinHandle<'s, PCompletionQueue<'a, C>>>,
    waker: Arc<RTWaker>,
    kill: Arc<AtomicBool>,
}

impl<'s, 'a, C: CQEM> CQReaper<'s, 'a, C> {
    pub fn new(
        scope: &'s Scope<'s, 'a>,
        cq: PCompletionQueue<'a, C>,
        timeout: Option<Duration>, // defaults to 1s
    ) -> Self {
        let waker = Arc::new(RTWaker::new(false));
        let kill = Arc::new(AtomicBool::new(false));

        let timeout = timeout.unwrap_or(Duration::from_secs(1));

        Self {
            thread: Self::thread_generator(scope, cq, waker.clone(), kill.clone(), timeout),
            kill,
            waker,
        }
    }

    #[inline]
    pub fn get_waker(&self) -> Arc<RTWaker> {
        self.waker.clone()
    }

    #[inline]
    pub fn wake(&self) {
        self.waker.wake();
    }

    #[inline]
    pub fn kill(&self) {
        self.kill.store(true, Ordering::Release);
        self.wake();
    }

    fn thread_generator(
        scope: &'s Scope<'s, 'a>,
        cq: PCompletionQueue<'a, C>,
        wake: Arc<RTWaker>,
        kill: Arc<AtomicBool>,
        timeout: Duration,
    ) -> Option<ScopedJoinHandle<'s, PCompletionQueue<'a, C>>> {
        Some(scope.spawn(Self::thread_fn_generator(cq, wake, kill, timeout)))
    }

    fn thread_fn_generator(
        mut cq: PCompletionQueue<'a, C>,
        waker: Arc<RTWaker>,
        kill: Arc<AtomicBool>,
        timeout: Duration,
    ) -> impl FnOnce() -> PCompletionQueue<'a, C> {
        move || {
            'exit: loop {
                if kill.load(Ordering::Acquire) {
                    // When this thread is killed it will ensure that the current queue is emptied.
                    cq.reap();

                    break 'exit cq;
                } else {
                    let mut epoch = Instant::now();

                    while epoch.elapsed() < timeout {
                        if cq.reap() != 0 {
                            epoch = Instant::now();
                        }
                    }

                    waker.wait();
                }
            }
        }
    }
}

impl<'s, 'a, C: CQEM> Drop for CQReaper<'s, 'a, C> {
    fn drop(&mut self) {
        // Safety: The scoped thread does not need to be joined here
        // because it is guaranteed to get joined by the time `'scope` expires.
        self.kill();
    }
}

impl<'s, 'a, C: CQEM> Into<PCompletionQueue<'a, C>> for CQReaper<'s, 'a, C> {
    fn into(mut self) -> PCompletionQueue<'a, C> {
        self.kill();
        self.thread
            .take()
            // Safety: This is the only way to make `self.thread` == `None`
            // and it consumes `self` so there is no chance for it to be called
            // when `self.thread` is `None`.
            .unwrap()
            .join()
            .expect("Failed to join reaper thread")
    }
}

impl<'s, 'a, C: CQEM> Into<CompletionQueue<'a, C>> for CQReaper<'s, 'a, C> {
    fn into(self) -> CompletionQueue<'a, C> {
        <Self as Into<PCompletionQueue<'a, C>>>::into(self).into()
    }
}

pub struct RTWaker {
    wakebool: Mutex<bool>,
    wakecv: Condvar,
}

impl RTWaker {
    pub fn new(init: bool) -> Self {
        Self {
            wakebool: Mutex::new(init),
            wakecv: Condvar::new(),
        }
    }

    fn get_lock(&self) -> MutexGuard<'_, bool> {
        self.wakebool.lock().unwrap()
    }

    pub fn wait(&self) {
        let mut guard = self.get_lock();
        while !*guard {
            guard = self.wakecv.wait(guard).unwrap();
        }
        *guard = false;
    }

    pub fn wake(&self) {
        *self.get_lock() = true;
        self.wakecv.notify_all();
    }
}
