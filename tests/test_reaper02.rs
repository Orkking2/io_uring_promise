use std::{io, thread};

use io_uring::opcode::Nop;
use io_uring_promise::{cqreaper::CQReaper, rsqueue::RSQueue, PromiseIoUring};

#[test]
fn main() -> io::Result<()> {
    let mut ring = PromiseIoUring::new(16)?;

    let (s, sq, cq) = ring.split();

    thread::scope(|scope| {
        let reaper = CQReaper::new(scope, cq, None);

        let waker = reaper.get_waker();

        let mut rsq = RSQueue::new(sq, waker.clone());

        let entries = std::iter::from_fn(|| Some(Nop::new().build())).take(10).collect::<Box<_>>();

        let promises = unsafe { rsq.push_multiple(entries).expect("SQ full") };

        // Let kernel know we have added submissions to the queue.
        s.submit()?;

        println!("Polling promises");

        // Polling promises wakes reaper.
        while promises.iter().any(|p| !p.poll()) {}

        println!("All promises returned!");

        Ok(())
    })
}