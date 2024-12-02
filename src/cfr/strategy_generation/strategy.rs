use crate::cfr::game_model::{Probability, VisibleInfo};
use crate::cfr::strategy_generation::workspace_data::data_for_infoset::DataForInfoSet;
use crate::cfr::strategy_generation::workspace_data::data_for_known_infosets::DataForKnownInfosets;
use bumpalo_herd::Member;
use rustc_hash::FxHashMap;
use std::collections::hash_map::Iter;
use std::fmt::{Debug, Formatter};

pub struct Strategy<'h, INFO: VisibleInfo> {
    pub(crate) infosets: DataForKnownInfosets<'h, INFO>,
}

impl<'h, INFO: VisibleInfo> Strategy<'h, INFO> {
    pub fn get_move_probabilities(
        &self,
        info: INFO,
        member: &Member<'h>,
    ) -> StrategyForInfoView<INFO> {
        StrategyForInfoView::new(self.infosets.data_for_infoset(info, member))
    }

    pub fn pick_move(&self, info: INFO, member: &Member<'h>) -> Option<INFO::Move> {
        let move_probabilities = self.get_move_probabilities(info, member);
        move_probabilities.pick_move()
    }
}

impl<'h, INFO: VisibleInfo> Default for Strategy<'h, INFO> {
    fn default() -> Self {
        Self {
            infosets: Default::default(),
        }
    }
}

pub struct StrategyForInfoView<'h, INFO: VisibleInfo> {
    data_for_info_set: &'h DataForInfoSet<INFO>,
    moves: FxHashMap<INFO::Move, Probability>,
}

impl<'h, INFO: VisibleInfo> StrategyForInfoView<'h, INFO> {
    pub(crate) fn new(data_for_info_set: &'h DataForInfoSet<INFO>) -> Self {
        Self {
            moves: data_for_info_set
                .moves()
                .iter()
                .map(|move_with_data| {
                    (
                        move_with_data.m,
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

    pub fn move_probabilities(&self) -> &FxHashMap<INFO::Move, Probability> {
        &self.moves
    }

    pub fn pick_move(&self) -> Option<INFO::Move> {
        for move_with_data in self.data_for_info_set.moves() {
            eprintln!(
                "\tMove {:?} has regret {:?}, probability {:?}",
                move_with_data.m,
                move_with_data.d.regret(),
                move_with_data.d.load_move_probability_unchecked()
            );
        }

        if self.data_for_info_set.is_terminal() {
            return None;
        }

        let mut moves: Vec<INFO::Move> = self.moves.keys().copied().collect();

        fastrand::shuffle(&mut moves);

        let mark = fastrand::f64();
        let mut cumulative = 0.0;

        for m in moves {
            let prob = self.move_probability(&m);
            cumulative += prob;
            eprintln!(
                "- Move {:?} has probability {:?} cumm {:?}",
                m, prob, cumulative
            );

            if mark < cumulative {
                return Some(m);
            }
        }

        eprintln!("Error with move probabilities {:?}", self.moves);
        panic!("Move probabilities did not sum to 1.0!")
    }
}

impl<'h, INFO: VisibleInfo> Debug for StrategyForInfoView<'h, INFO> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("StrategyForInfo[")?;
        self.data_for_info_set.fmt(f)?;
        f.write_str("]")?;

        Ok(())
    }
}
