use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

use crate::{CQEM, pstatus::PromiseStatus};

pub type RegRef<C> = Rc<RefCell<PromiseRegistry<C>>>;

pub fn new_reg_ref<C: CQEM>() -> RegRef<C> {
    Rc::new(RefCell::new(PromiseRegistry::new()))
}

pub struct PromiseRegistry<C: CQEM> {
    completed: HashMap<u64, C>,
    scheduled: HashSet<u64>,
    curr_uuid: u64,
}

impl<C: CQEM> PromiseRegistry<C> {
    #[inline]
    pub fn new() -> Self {
        Self {
            completed: HashMap::new(),
            scheduled: HashSet::new(),
            curr_uuid: 0,
        }
    }

    #[inline]
    pub fn curr_uuid(&self) -> u64 {
        self.curr_uuid
    }

    #[inline]
    fn incr_uuid(&mut self) -> u64 {
        let out = self.curr_uuid;
        self.curr_uuid = self.curr_uuid.wrapping_add(1);
        out
    }

    #[inline]
    pub fn next_uuid(&mut self) -> u64 {
        loop {
            let id = self.incr_uuid();
            if self.get_status(&id) == PromiseStatus::None {
                break id;
            }
        }
    }

    #[inline]
    pub fn get_status(&self, k: &u64) -> PromiseStatus {
        if self.completed.contains_key(k) {
            PromiseStatus::Completed
        } else if self.scheduled.contains(k) {
            PromiseStatus::Scheduled
        } else {
            PromiseStatus::None
        }
    }

    /// PromiseStatus tells you in what state the promise is, either it is scheduled or the key does not exist.
    #[inline]
    pub fn remove(&mut self, k: &u64) -> Result<C, PromiseStatus> {
        if let Some(entry) = self.completed.remove(k) {
            Ok(entry)
        } else {
            Err(self.get_status(k))
        }
    }

    /// Returns `false` if a promise with this user_data was already scheduled.
    #[inline]
    pub fn schedule(&mut self, user_data: u64) -> bool {
        self.scheduled.insert(user_data)
    }

    /// Returns `true` if a promise was successfully unscheduled.
    #[inline]
    pub fn unschedule(&mut self, user_data: &u64) -> bool {
        self.scheduled.remove(user_data)
    }

    #[inline]
    fn extract_user_data(entry: C) -> (u64, C) {
        let user_data = entry.user_data();

        (user_data, entry)
    }

    /// Returns the completed promise that this call overwrites, if it exists.
    #[inline]
    pub fn complete(&mut self, entry: C) -> Option<C> {
        let (user_data, entry) = Self::extract_user_data(entry);

        self.unschedule(&user_data);

        self.completed.insert(user_data, entry)
    }

    /// Complete a batch of entries.
    #[inline]
    pub fn batch_complete<I>(&mut self, entries: I)
    where
        I: IntoIterator<Item = C>,
    {
        self.completed
            .extend(
                entries
                    .into_iter()
                    .map(Self::extract_user_data)
                    .map(|(user_data, entry)| {
                        self.scheduled.remove(&user_data);

                        (user_data, entry)
                    }),
            )
    }
}
