use io_uring::cqueue::EntryMarker;
use io_uring::{IoUring, opcode, types};
use io_uring_promise::PromiseIoUring;
use std::os::unix::io::AsRawFd;
use std::{fs, io};

#[test]
fn main() -> io::Result<()> {
    let mut ring = PromiseIoUring::new(32)?;

    let f = |_| opcode::Nop::new().build();

    let reads = (0..10).into_iter().map(f).collect::<Box<[_]>>();

    let promises;
    
    // Note that the developer needs to ensure
    // that the entry pushed into submission queue is valid (e.g. fd, buffer).
    unsafe {
        promises = ring
            .submission()
            .push_multiple(reads)
            .expect("submission queue is full");
    }

    ring.submit_and_wait(10)?;

    ring.completion().reap();

    promises.into_iter().for_each(|p| assert!(p.poll()));

    Ok(())
}
