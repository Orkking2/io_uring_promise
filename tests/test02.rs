use std::{io, os::fd::AsRawFd, thread};

use io_uring::{IoUring, opcode, types};
use io_uring_promise::{CQE, SQE, multithread::PIoUring, pstatus::PromiseStatus};
use tempfile::tempfile;

fn gen_thread_fn(ring: PIoUring<SQE, CQE>, id: usize, num_entries: usize) -> impl FnOnce() -> () {
    move || {
        let id = format!("{id:>2}");

        let fs = tempfile().unwrap();

        let buf = vec![0u8; 32];

        let promises = unsafe {
            ring.batch_submit(
                std::iter::repeat(
                    opcode::Write::new(
                        types::Fd(fs.as_raw_fd()),
                        buf.as_ptr() as _,
                        buf.len() as _,
                    )
                    .build(),
                )
                .take(num_entries),
            )
        };

        let mut len = promises.len();

        println!("Thread {id} submitted {len} promises");

        loop {
            let plen = promises
                .iter()
                .filter(|&p| p.status() != PromiseStatus::Completed)
                .collect::<Vec<_>>()
                .len();
            if plen < len {
                println!("Thread {id} has completed {}, has left {plen}", len - plen);
                len = plen;
            } else {
                break;
            }
        }

        println!("Thread {id} joining");
    }
}

#[test]
fn main() -> io::Result<()> {
    // The SQ needs only 1 slot since queueing can be done entirely within user-space using the Sender.
    let ring = PIoUring::new(IoUring::new(1)?);

    let threads = (0..32)
        .map(|id| thread::spawn(gen_thread_fn(ring.clone(), id, 2000)))
        .collect::<Vec<_>>();

    threads
        .into_iter()
        .for_each(|thread| thread.join().unwrap());

    Ok(())
}
