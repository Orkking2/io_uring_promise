use std::fmt::Display;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PromiseStatus {
    Scheduled,
    Completed,
    None
}

impl Display for PromiseStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PromiseStatus::Scheduled => write!(f, "promise scheduled"),
            PromiseStatus::Completed => write!(f, "promise completed"),
            PromiseStatus::None => write!(f, "promise not registered"),
        }
    }
}