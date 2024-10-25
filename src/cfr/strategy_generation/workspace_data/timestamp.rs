use static_assertions::const_assert;

pub(crate) const MAX_BATCH_SIZE: usize = 1;
// If you want to update this assert, you need to also change the size of the bitset in the `DataForInfoset` struct
const_assert!(MAX_BATCH_SIZE > 0 && MAX_BATCH_SIZE <= 64);

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
