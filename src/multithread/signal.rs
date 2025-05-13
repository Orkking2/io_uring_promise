use crate::SQEM;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Signal<S: SQEM> {
    Entry(S),
    Reap,
}