use crate::cfr::game_model::{Probability, VisibleInfo};
use crate::cfr::strategy_generation::workspace_data::data_for_infoset::DataForInfoSet;
use crate::cfr::strategy_generation::workspace_data::data_for_known_infosets::DataForKnownInfosets;
use rustc_hash::FxHashMap;
use std::collections::hash_map::Iter;
use std::fmt::{Debug, Formatter};
use std::sync::Arc;

pub struct Strategy<INFO: VisibleInfo> {
    pub(crate) infosets: DataForKnownInfosets<INFO>,
}

impl<INFO: VisibleInfo> Strategy<INFO> {
    pub fn get_move_probabilities(&self, info: INFO) -> StrategyForInfoView<INFO> {
        StrategyForInfoView::new(self.infosets.data_for_infoset(info))
    }

    pub fn pick_move(&self, info: INFO) -> Option<INFO::Move> {
        let move_probabilities = self.get_move_probabilities(info);
        eprintln!(
            "The current infoset has an expected value of {:?}",
            move_probabilities
                .data_for_info_set
                .get_cumulative_counterfactual()
        );
        for move_with_data in move_probabilities.data_for_info_set.moves() {
            eprintln!(
                "\tMove {:?} has regret {:?}, probability {:?}",
                move_with_data.m,
                move_with_data.d.regret(),
                move_with_data.d.load_move_probability_unchecked()
            );
        }

        if move_probabilities.data_for_info_set.is_terminal() {
            return None;
        }

        let mut moves: Vec<INFO::Move> =
            move_probabilities.iter().map(|(m, _)| m.clone()).collect();

        fastrand::shuffle(&mut moves);

        let mark = fastrand::f64();
        let mut cumulative = 0.0;

        for m in moves {
            let prob = move_probabilities.move_probability(&m);
            cumulative += prob;
            eprintln!(
                "- Move {:?} has probability {:?} cumm {:?}",
                m, prob, cumulative
            );

            if mark < cumulative {
                return Some(m);
            }
        }

        eprintln!(
            "Error with move probabilities {:?}",
            move_probabilities.moves
        );
        panic!("Move probabilities did not sum to 1.0!")
    }
}

impl<INFO: VisibleInfo> Default for Strategy<INFO> {
    fn default() -> Self {
        Self {
            infosets: Default::default(),
        }
    }
}

pub struct StrategyForInfoView<INFO: VisibleInfo> {
    data_for_info_set: Arc<DataForInfoSet<INFO>>,
    moves: FxHashMap<INFO::Move, Probability>,
}

impl<INFO: VisibleInfo> StrategyForInfoView<INFO> {
    pub(crate) fn new(data_for_info_set: Arc<DataForInfoSet<INFO>>) -> Self {
        Self {
            moves: data_for_info_set
                .moves()
                .iter()
                .map(|move_with_data| {
                    (
                        move_with_data.m.clone(),
                        move_with_data
                            .d
                            .load_move_probability(data_for_info_set.moves().len()),
                    )
                })
                .collect(),
            data_for_info_set,
        }
    }

    pub fn iter(&self) -> Iter<'_, INFO::Move, Probability> {
        self.moves.iter()
    }

    pub fn move_count(&self) -> usize {
        self.moves.len()
    }

    pub fn move_probability(&self, m: &INFO::Move) -> Probability {
        *self
            .moves
            .get(m)
            .expect("All moves for infoset must have probability")
    }
}

impl<INFO: VisibleInfo> Debug for StrategyForInfoView<INFO> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("StrategyForInfo[")?;
        self.data_for_info_set.fmt(f)?;
        f.write_str("]")?;

        Ok(())
    }
}
