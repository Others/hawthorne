use std::hash::{BuildHasherDefault};
use std::sync::Arc;
use dashmap::DashMap;
use rustc_hash::{FxHashMap, FxHasher};
use crate::cfr::game_model::{AtomicProbability, AtomicUtility, UtilityForAllPlayers, VisibleInfo};

pub(crate) struct DataForKnownInfosets<INFO: VisibleInfo> {
    infoset_data: DashMap<INFO, Arc<DataForInfoSet<INFO>>, BuildHasherDefault<FxHasher>>,
}

pub(crate) struct DataForInfoSet<INFO: VisibleInfo> {
    cumulative_counterfactual_value: AtomicUtility,
    terminal_utility: UtilityForAllPlayers,
    move_data: FxHashMap<INFO::Move, DataForMove<INFO>>
}

pub(crate) struct DataForMove<INFO: VisibleInfo> {
    cumulative_move_regret: AtomicUtility,
    move_selection_probability: AtomicProbability
}
