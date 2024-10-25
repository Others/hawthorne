use crate::cfr::game_model::UtilityForAllPlayers;
use crate::cfr::strategy_generation::workspace_data::timestamp::{Timestamp, MAX_BATCH_SIZE};
use parking_lot::{Mutex, MutexGuard};
use std::sync::atomic::{AtomicU32, Ordering};

#[derive(Debug)]
pub(crate) struct DataPerBatchItem<T> {
    updated_iteration_per_work_item: [AtomicU32; MAX_BATCH_SIZE],
    // Since batches are in practice each handled by a different thread, this mutex is uncontended
    // and should be nearly free
    data: [Mutex<T>; MAX_BATCH_SIZE],
}

impl<T> DataPerBatchItem<T> {
    pub fn new(f: impl Fn() -> T) -> Self {
        Self {
            updated_iteration_per_work_item: [const { AtomicU32::new(0) }; MAX_BATCH_SIZE],
            data: [(); MAX_BATCH_SIZE].map(|_| Mutex::new(f())),
        }
    }

    pub fn set(&self, value: T, timestamp: Timestamp) {
        self.updated_iteration_per_work_item[timestamp.item_within_iteration]
            .store(timestamp.cfr_iteration, Ordering::Relaxed);
        *self.data[timestamp.item_within_iteration].lock() = value
    }

    pub fn get(&self, timestamp: Timestamp) -> Option<MutexGuard<T>> {
        if !self.ready(timestamp) {
            None
        } else {
            Some(self.data[timestamp.item_within_iteration].lock())
        }
    }

    pub fn ready(&self, timestamp: Timestamp) -> bool {
        self.updated_iteration_per_work_item[timestamp.item_within_iteration]
            .load(Ordering::Relaxed)
            >= timestamp.cfr_iteration
    }
}

impl DataPerBatchItem<UtilityForAllPlayers> {
    pub const fn const_default_utility() -> Self {
        Self {
            updated_iteration_per_work_item: [const { AtomicU32::new(0) }; MAX_BATCH_SIZE],
            data: [const { Mutex::new(UtilityForAllPlayers::const_default()) }; MAX_BATCH_SIZE],
        }
    }
}

impl<T> DataPerBatchItem<Option<T>> {
    pub const fn const_default_option() -> Self {
        Self {
            updated_iteration_per_work_item: [const { AtomicU32::new(0) }; MAX_BATCH_SIZE],
            data: [const { Mutex::new(None) }; MAX_BATCH_SIZE],
        }
    }
}

impl<T> Default for DataPerBatchItem<T>
where
    T: Default,
    T: Clone,
{
    fn default() -> Self {
        Self::new(Default::default)
    }
}
