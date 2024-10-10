
pub mod strategy;

use std::hash::BuildHasherDefault;
use dashmap::DashSet;
use rustc_hash::FxHasher;
use strategy::Strategy;
mod accumulate_regret;
mod counterfactual_workspace;
mod regret;
mod update_strategy;
mod workspace_data;

use crate::cfr::game_model::VisibleInfo;
use crate::cfr::strategy_generation::accumulate_regret::add_to_regret;
use crate::cfr::strategy_generation::counterfactual_workspace::CounterfactualWorkspace;
use crate::cfr::strategy_generation::regret::Regret;
use crate::cfr::strategy_generation::update_strategy::update_strategy_from_regret;

pub fn generate_strategy_2<INFO: VisibleInfo>(
    starting_info: INFO,
    iterations: u32,
) -> Strategy<INFO> {
    let progress_data = CfrProgressData::default();
    let workspace = CounterfactualWorkspace::default();

    for i in 1..=iterations {
        eprintln!("Starting Iteration {}/{}", i, iterations);

        add_to_regret(&starting_info, &progress_data, &workspace, i);
        eprintln!("Switching to strategy update {}/{}", i, iterations);
        update_strategy_from_regret(&progress_data);

        eprintln!("Ending Iteration {}/{}", i, iterations);
    }

    progress_data.current_strategy
}

#[derive(Debug)]
pub(crate) struct CfrProgressData<INFO: VisibleInfo> {
    // This is data we keep forever and permanently track
    pub(crate) current_strategy: Strategy<INFO>,
    pub(crate) cumulative_regret: Regret<INFO>,
    pub(crate) witnessed: DashSet<INFO, BuildHasherDefault<FxHasher>>
}

impl<INFO: VisibleInfo> Default for CfrProgressData<INFO> {
    fn default() -> Self {
        Self {
            current_strategy: Strategy::default(),
            cumulative_regret: Regret::default(),
            witnessed: DashSet::default()
        }
    }
}
