use std::sync::{Arc, RwLock};

use crate::{CQEM, registry::PromiseRegistry};

pub type RegRef<C> = Arc<RwLock<PromiseRegistry<C>>>;

pub fn new_reg_ref<C: CQEM>() -> RegRef<C> {
    Arc::new(RwLock::new(PromiseRegistry::new()))
}
