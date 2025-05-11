use crate::SQEM;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Signal<S: SQEM> {
    Entry(u64, S),
    Reap,
}