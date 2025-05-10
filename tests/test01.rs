use std::{io, thread, time::Duration};

use io_uring::{opcode, IoUring};
use io_uring_promise::PIoUring;

#[test]
fn main() -> io::Result<()> {
    let mut ring = PIoUring::new(IoUring::new(32)?);

    let entry = opcode::Nop::new().build();

    let mut promise = unsafe { ring.submit(entry) }.unwrap();

    // do other stuff
    thread::sleep(Duration::from_secs(1));

    promise.try_wait().unwrap();

    Ok(())
}