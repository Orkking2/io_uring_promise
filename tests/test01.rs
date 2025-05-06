use io_uring::opcode;
use io_uring_promise::PromiseIoUring;
use std::io;

#[test]
fn main() -> io::Result<()> {
    let mut ring = PromiseIoUring::new(32)?;

    let f = || Some(opcode::Nop::new().build());

    let reads = std::iter::from_fn(f).take(10).collect::<Box<_>>();

    // Note that the developer needs to ensure
    // that the entry pushed into submission queue is valid (e.g. fd, buffer).
    let promises = unsafe {
        ring.submission()
            .push_multiple(reads)
            .expect("submission queue is full")
    };

    assert_eq!(ring.submission().len(), 10);

    promises.iter().for_each(|p| assert!(!p.poll()));

    // Tell the kernel to run the operations in the SQ and move them to the CQ
    ring.submit_and_wait(10)?;

    assert_eq!(ring.completion().len(), 10);

    // Move all CQEs to the promise registry
    ring.completion().reap();

    assert!(ring.completion().is_empty());

    promises.into_iter().for_each(|p| assert!(p.poll()));

    Ok(())
}
