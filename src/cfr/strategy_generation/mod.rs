#![allow(dead_code)]

mod accumulate_regret;
mod counterfactual_workspace;
mod regret;
mod strategy;
mod update_strategy;
mod workspace_data;

use crate::cfr::game_model::VisibleInfo;
use crate::cfr::strategy_generation::accumulate_regret::add_to_regret;
use crate::cfr::strategy_generation::strategy::Strategy;
use crate::cfr::strategy_generation::update_strategy::update_strategy_from_regret;
use crate::cfr::strategy_generation::workspace_data::StrategyGenerationProgress;

pub fn generate_strategy_2<INFO: VisibleInfo>(
    starting_info: INFO,
    iterations: u32,
) -> Strategy<INFO> {
    // let progress_data = CfrProgressData::default();
    // let workspace = CounterfactualWorkspace::default();

    let data = StrategyGenerationProgress::new(iterations as usize);

    for i in 1..=iterations {
        eprintln!("Starting Iteration {}/{}", i, iterations);

        add_to_regret(&starting_info, &data, i);
        eprintln!("Switching to strategy update {}/{}", i, iterations);

        update_strategy_from_regret(&data);

        eprintln!("Ending Iteration {}/{}", i, iterations);
    }

    Strategy {
        infosets: data.into_infoset_data(),
    }
}
//
// #[derive(Debug)]
// pub(crate) struct CfrProgressData<INFO: VisibleInfo> {
//     // This is data we keep forever and permanently track
//     pub(crate) current_strategy: Strategy<INFO>,
//     pub(crate) cumulative_regret: Regret<INFO>,
//     pub(crate) witnessed: DashSet<INFO, BuildHasherDefault<FxHasher>>,
// }
//
// impl<INFO: VisibleInfo> Default for CfrProgressData<INFO> {
//     fn default() -> Self {
//         Self {
//             current_strategy: Strategy::default(),
//             cumulative_regret: Regret::default(),
//             witnessed: DashSet::default(),
//         }
//     }
// }
