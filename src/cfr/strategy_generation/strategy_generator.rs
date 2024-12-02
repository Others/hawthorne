use crate::cfr::game_model::{GamestateSampler, VisibleInfo};
use crate::cfr::strategy_generation::cfr_algorithm_impl::accumulate_regret::add_to_regret;
use crate::cfr::strategy_generation::strategy::{Strategy, StrategyForInfoView};
use crate::cfr::strategy_generation::update_strategy::update_strategy_from_regret;
use crate::cfr::strategy_generation::workspace_data::StrategyGenerationProgress;
use bumpalo_herd::{Herd, Member};
use std::sync::atomic::{AtomicU32, Ordering};

pub struct StrategyGenerator<'h, INFO: VisibleInfo> {
    herd: &'h Herd,
    herd_member: Member<'h>,

    iterations: AtomicU32,
    strategy_generation_progress: StrategyGenerationProgress<'h, INFO>,
}

impl<'h, INFO: VisibleInfo> StrategyGenerator<'h, INFO> {
    pub fn new(herd: &'h Herd) -> Self {
        Self {
            herd,
            herd_member: herd.get(),
            iterations: AtomicU32::new(1),
            strategy_generation_progress: StrategyGenerationProgress::new(),
        }
    }

    fn advance_strategy_once<GENERATOR: GamestateSampler<Info = INFO>>(
        &self,
        starting_gamestate_sampler: GENERATOR,
    ) {
        let iteration = self.iterations.fetch_add(1, Ordering::Relaxed);
        eprintln!("Starting Iteration {}", iteration);

        add_to_regret(
            starting_gamestate_sampler,
            &self.strategy_generation_progress,
            self.herd,
            iteration,
        );
        eprintln!("Switching to strategy update {}", iteration);

        update_strategy_from_regret(&self.strategy_generation_progress);
        eprintln!("Ending Iteration {}", iteration);
    }

    pub fn refine_strategy<GENERATOR: GamestateSampler<Info = INFO>>(
        &self,
        starting_gamestate_sampler: GENERATOR,
        n: u32,
    ) {
        for _ in 0..n {
            self.advance_strategy_once(starting_gamestate_sampler.clone());
        }
    }

    pub fn strategy_for_info(&self, state: INFO) -> StrategyForInfoView<'h, INFO> {
        let data_for_info = self
            .strategy_generation_progress
            .get_data_for_infoset(state, &self.herd_member);

        StrategyForInfoView::new(data_for_info)
    }

    pub fn into_strategy(self) -> Strategy<'h, INFO> {
        Strategy {
            infosets: self.strategy_generation_progress.into_infoset_data(),
        }
    }
}
