pub(crate) const MAX_BATCH_SIZE: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Timestamp {
    pub(crate) cfr_iteration: u32,
    pub(crate) item_within_iteration: usize,
}

impl Timestamp {
    pub fn new(cfr_iteration: u32, item_within_batch: usize) -> Self {
        debug_assert!(item_within_batch < MAX_BATCH_SIZE);
        debug_assert!(cfr_iteration > 0);

        Self {
            cfr_iteration,
            item_within_iteration: item_within_batch,
        }
    }
}
