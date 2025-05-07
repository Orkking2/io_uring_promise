use io_uring::opcode;
use io_uring_promise::{PromiseIoUring, cqreaper::CQReaper};
use std::{io, thread};

#[test]
fn main() -> io::Result<()> {
    println!("Creating ring");
    let mut ring = PromiseIoUring::new(32)?;

    let f = || Some(opcode::Nop::new().build());

    let entries = std::iter::from_fn(f).take(10).collect::<Box<_>>();

    let (s, mut sq, cq) = ring.split();

    println!("Pushing promises");
    let promises = unsafe { sq.push_multiple(entries).expect("submission queue is full") };

    assert_eq!(sq.len(), 10);

    promises.iter().for_each(|p| assert!(!p.poll()));

    thread::scope(move |scope| -> io::Result<()> {
        // reaper must live within the scope of a scoped thread to ensure
        // that the reaper thread (which holds a `CompletionQueue`) does not outlive the `CompletionQueue`'s borrow
        println!("Creating reaper");
        let reaper = CQReaper::new(scope, cq, None);
        
        println!("Waking reaper");
        reaper.wake();
        
        println!("Submit and wait...");
        s.submit_and_wait(10)?;
        
        // When the reaper is dropped, it will automatically reap one last time.
        println!("Dropping reaper");

        Ok(())
    })?;

    promises.into_iter().for_each(|p| assert!(p.poll()));

    Ok(())
}
