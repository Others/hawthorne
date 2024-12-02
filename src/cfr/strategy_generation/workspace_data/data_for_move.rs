use crate::cfr::game_model::{
    AtomicProbability, AtomicUtility, OracleGamestate, Probability, Utility, UtilityForAllPlayers,
    VisibleInfo,
};
use crate::cfr::strategy_generation::workspace_data::batch_item_data::DataPerBatchItem;
use crate::cfr::strategy_generation::workspace_data::data_for_infoset::DataForInfoSet;
use crate::cfr::strategy_generation::workspace_data::timestamp::Timestamp;
use crate::cfr::strategy_generation::workspace_data::StrategyGenerationProgress;
use bumpalo_herd::Member;
use std::sync::atomic::Ordering;

#[derive(Debug)]
pub(crate) struct DataForMove {
    move_selection_probability: AtomicProbability,
    // Since multiple work threads can exist at a time, we need dedicated storage per batch item
    cumulative_move_regret: AtomicUtility,
    utility_after_move: DataPerBatchItem<UtilityForAllPlayers>,
    // cached_post_move_infoset:
    //     DataPerBatchItem<Option<(Arc<DataForInfoSet<INFO>>, Arc<INFO::Gamestate>)>>,
    // TODO: Can we consoldiate the previous two?
}

impl DataForMove {
    pub const fn new() -> Self {
        Self {
            cumulative_move_regret: AtomicProbability::new(0.0),
            // Zero on first iteration. NaN if the probability is actually zero
            move_selection_probability: AtomicProbability::new(0.0),
            utility_after_move: const { DataPerBatchItem::const_default_utility() },
            // cached_post_move_infoset: const { DataPerBatchItem::const_default_option() },
        }
    }

    pub fn ready(&self, timestamp: Timestamp) -> bool {
        self.utility_after_move.ready(timestamp)
    }

    pub fn load_move_probability(&self, n_moves: usize) -> Probability {
        let res = self.move_selection_probability.load(Ordering::Relaxed);

        if res == 0.0 {
            1.0 / n_moves as Probability
        } else if res.is_nan() {
            0.0
        } else {
            res
        }
    }

    pub fn load_move_probability_unchecked(&self) -> Probability {
        self.move_selection_probability.load(Ordering::Relaxed)
    }

    pub fn write_move_probability(&self, p: Probability) {
        debug_assert!(!p.is_nan());

        if p == 0.0 {
            self.move_selection_probability
                .store(Probability::NAN, Ordering::Relaxed);
        } else {
            self.move_selection_probability.store(p, Ordering::Relaxed);
        }
    }

    pub fn utility_after_move(&self) -> &DataPerBatchItem<UtilityForAllPlayers> {
        &self.utility_after_move
    }

    pub fn get_post_move_infoset<'h, INFO: VisibleInfo>(
        &self,
        strategy_generation_progress: &StrategyGenerationProgress<'h, INFO>,
        member: &Member<'h>,
        gamestate_before_move: &INFO::Gamestate,
        m: &INFO::Move,
    ) -> (&'h DataForInfoSet<INFO>, INFO::Gamestate) {
        // if let Some(d) = self.cached_post_move_infoset.get(timestamp) {
        //     return d.as_ref().unwrap().clone();
        // }

        let state_after_move = gamestate_before_move.advance(m);
        let info_after_move = state_after_move.info_for_turn_player();
        let data_for_info_after_move =
            strategy_generation_progress.get_data_for_infoset(info_after_move, member);

        // self.cached_post_move_infoset.set(
        //     Some((data_for_info_after_move.clone(), state_after_move.clone())),
        //     timestamp,
        // );

        (data_for_info_after_move, state_after_move)
    }

    pub fn accumulate_regret<'h, INFO: VisibleInfo>(
        &self,
        strategy_generation_progress: &StrategyGenerationProgress<'h, INFO>,
        member: &Member<'h>,
        timestamp: Timestamp,
        pre_move_info: &DataForInfoSet<INFO>,
        pre_move_gamestate: &INFO::Gamestate,
        m: &INFO::Move,
    ) {
        let turn = pre_move_info.turn();
        let counterfactual_before = pre_move_info
            .get_iteration_utility_if_ready(timestamp)
            .unwrap();

        let info_after = pre_move_gamestate.advance(m).info_for_turn_player();
        let counterfactual_after = strategy_generation_progress
            .get_data_for_infoset(info_after, member)
            .get_iteration_utility_if_ready(timestamp)
            .unwrap();

        // let counterfactual_after = self
        //     .cached_post_move_infoset
        //     .get(timestamp)
        //     .and_then(|a| {
        //         a.as_ref()
        //             .map(|(data, _)| data.get_cumulative_counterfactual().clone())
        //     })
        //     .expect("You can only accumulate regret when children are ready");

        let new_positive_regret =
            (counterfactual_after.get(turn) - counterfactual_before.get(turn)).max(0.0);

        // FIXME: Make this configurable
        let weighted_regret = new_positive_regret * timestamp.cfr_iteration as Utility;

        self.cumulative_move_regret
            .fetch_add(weighted_regret, Ordering::Relaxed);
    }

    pub fn regret(&self) -> Utility {
        self.cumulative_move_regret.load(Ordering::Relaxed)
    }
}

#[cfg(test)]
mod test {
    use crate::cfr::strategy_generation::workspace_data::data_for_move::DataForMove;
    use std::mem;

    #[test]
    fn empty_is_zeroed() {
        let empty = DataForMove::new();
        let struct_size = size_of::<DataForMove>();

        let struct_pointer = (&empty) as *const DataForMove as *const u8;
        mem::forget(empty);

        let bytes = unsafe { std::slice::from_raw_parts(struct_pointer, struct_size) };

        for (i, b) in bytes.iter().enumerate() {
            assert_eq!(*b, 0, "Testing byte {}", i);
        }
    }
}
