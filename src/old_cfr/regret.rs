use crate::cfr::game_model::{Moves, Strategy, Utility, VisibleInfo};
use crate::old_cfr::counterfactual::CounterfactualValue;
use rustc_hash::FxHashMap;
use std::borrow::Cow;
use std::cell::Cell;

#[derive(Clone, Debug)]
pub(crate) struct RegretStorage {
    regret: Cell<Utility>,
}

impl RegretStorage {
    pub(crate) fn regret(&self) -> Utility {
        self.regret.get()
    }
}

impl Default for RegretStorage {
    fn default() -> Self {
        Self {
            regret: Cell::new(0.0),
        }
    }
}

#[inline(never)]
pub(crate) fn add_to_cumulative_regret<INFO: VisibleInfo>(
    current_strategy: &Strategy<INFO>,
    counterfactual_values: &FxHashMap<(Cow<INFO>, Cow<INFO::Move>), CounterfactualValue>,
    cumulative_regret: &FxHashMap<(Cow<INFO>, Cow<INFO::Move>), RegretStorage>,
) {
    for info in current_strategy.probability.keys() {
        let moves = match info.moves() {
            Moves::PossibleMoves(moves) => moves,
            Moves::Terminal { .. } => continue,
        };

        let mut strategy_utility = 0.0;
        for m in &moves {
            let counterfactual_value = counterfactual_values
                .get(&(Cow::Borrowed(info), Cow::Borrowed(m)))
                .unwrap();

            strategy_utility +=
                counterfactual_value.move_probability() * counterfactual_value.after_move_utility();
        }

        for m in &moves {
            let counterfactual_value = counterfactual_values
                .get(&(Cow::Borrowed(info), Cow::Borrowed(m)))
                .unwrap();

            let regret = (counterfactual_value.after_move_utility() - strategy_utility).max(0.0);

            let key = (Cow::Borrowed(info), Cow::Borrowed(m));

            let regret_storage = cumulative_regret.get(&key).unwrap();

            let current_regret = regret_storage.regret.get();
            regret_storage.regret.set(current_regret + regret);
        }
    }
}
