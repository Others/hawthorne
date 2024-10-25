use crate::cfr::game_model::{PlayerNumber, Probability, UtilityForAllPlayers, VisibleInfo};
use crate::cfr::strategy_generation::workspace_data::batch_item_data::DataPerBatchItem;
use crate::cfr::strategy_generation::workspace_data::move_data::{
    MoveWithData, MoveWithDataAllocation,
};
use crate::cfr::strategy_generation::workspace_data::timestamp::Timestamp;
use atomic_float::AtomicF64;
use parking_lot::Mutex;
use std::sync::atomic::{AtomicU32, Ordering};

#[derive(Debug)]
pub(crate) struct DataForInfoSet<INFO: VisibleInfo> {
    turn_player: PlayerNumber,
    terminal_utility: Option<UtilityForAllPlayers>,

    counterfactual_n: AtomicF64,
    cumulative_counterfactual: Mutex<UtilityForAllPlayers>,
    counterfactual_for_current_iteration: DataPerBatchItem<UtilityForAllPlayers>,

    // Use this to figure out if we need to write into the `updated_infosets` SegQueue
    global_updated_iteration: AtomicU32,

    move_data: Vec<MoveWithData<INFO>>,
}

impl<INFO: VisibleInfo> DataForInfoSet<INFO> {
    #[inline]
    pub(crate) fn new(info: &INFO) -> Self {
        let mut move_data = MoveWithDataAllocation::new();

        let terminal_utility = info.run_for_moves(|m| {
            move_data.push(m);
        });

        Self {
            turn_player: info.turn(),
            terminal_utility,
            move_data: move_data.into_vec(),

            counterfactual_n: AtomicF64::new(0.0),
            cumulative_counterfactual: const { Mutex::new(UtilityForAllPlayers::const_default()) },
            counterfactual_for_current_iteration: const { DataPerBatchItem::const_default_utility() },

            global_updated_iteration: AtomicU32::new(0),
        }
    }

    pub(crate) fn turn(&self) -> PlayerNumber {
        self.turn_player
    }

    pub(crate) fn is_ready(&self, timestamp: Timestamp) -> bool {
        self.counterfactual_for_current_iteration.ready(timestamp)
    }

    pub(crate) fn ready_with_counterfactual(
        &self,
        util: UtilityForAllPlayers,
        gamestate_probablity: Probability,
        timestamp: Timestamp,
    ) {
        let prev_n = self.counterfactual_n.fetch_add(1.0, Ordering::Relaxed);
        let mut cumulative_counterfactual = self.cumulative_counterfactual.lock();
        cumulative_counterfactual.reduce(prev_n);
        // TODO: Does this really make sense in games where different gamestates have different probabilities?
        cumulative_counterfactual.accumulate(&util, gamestate_probablity);
        cumulative_counterfactual.reduce(1.0 / (prev_n + 1.0));

        let mut util = util;
        util.reduce(gamestate_probablity);
        self.counterfactual_for_current_iteration
            .set(util, timestamp);
    }

    pub(crate) fn move_count(&self) -> usize {
        self.move_data.len()
    }

    pub(crate) fn moves(&self) -> &[MoveWithData<INFO>] {
        &self.move_data
    }

    pub(crate) fn increment_terminal_counterfactual(
        &self,
        gamestate_probablity: Probability,
        timestamp: Timestamp,
    ) -> bool {
        match &self.terminal_utility {
            Some(x) => {
                self.ready_with_counterfactual(*x, gamestate_probablity, timestamp);
                true
            }
            None => false,
        }
    }

    pub(crate) fn check_for_writeback(&self, timestamp: Timestamp) -> bool {
        let current_iteration = timestamp.cfr_iteration;

        let old_iter = self
            .global_updated_iteration
            .swap(current_iteration, Ordering::Relaxed);

        // Return that writeback is needed iff the iteration was updated
        old_iter != current_iteration
    }

    pub(crate) fn get_iteration_utility_if_ready(
        &self,
        timestamp: Timestamp,
    ) -> Option<UtilityForAllPlayers> {
        self.counterfactual_for_current_iteration
            .get(timestamp)
            .map(|x| *x)
    }

    pub(crate) fn get_cumulative_counterfactual(&self) -> UtilityForAllPlayers {
        *self.cumulative_counterfactual.lock()
    }

    pub(crate) fn is_terminal(&self) -> bool {
        self.terminal_utility.is_some()
    }
}
