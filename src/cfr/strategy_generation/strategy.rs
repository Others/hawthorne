use crate::cfr::game_model::{
    AtomicProbability, Moves, Probability, UtilityForAllPlayers, VisibleInfo,
};
use dashmap::mapref::one::Ref;
use dashmap::DashMap;
use rustc_hash::{FxHashMap, FxHasher};
use std::collections::hash_map::Iter;
use std::fmt::{Debug, Formatter};
use std::hash::BuildHasherDefault;
use std::sync::atomic::Ordering;

#[derive(Debug)]
pub struct Strategy<INFO: VisibleInfo> {
    pub(crate) strategy: DashMap<INFO, StrategyForInfo<INFO>, BuildHasherDefault<FxHasher>>,
}

impl<INFO: VisibleInfo> Strategy<INFO> {

    pub fn get_move_probabilities(&self, info: &INFO) -> StrategyForInfoView<INFO> {
        match self.strategy.get(info) {
            Some(v) => StrategyForInfoView { v },
            None => self.get_move_probabilities_locking(info),
        }
    }

    fn get_move_probabilities_locking(&self, info: &INFO) -> StrategyForInfoView<INFO> {
        let v = self
            .strategy
            .entry(info.clone())
            .or_insert_with(|| match info.moves() {
                Moves::PossibleMoves(moves) => {
                    let len = moves.len() as Probability;

                    let m = moves
                        .into_iter()
                        .map(|m| (m, AtomicProbability::from(1.0 / len)))
                        .collect();
                    StrategyForInfo {
                        strategy: m,
                        terminal_utility: None,
                    }
                }
                Moves::Terminal { utility } => StrategyForInfo {
                    strategy: FxHashMap::default(),
                    terminal_utility: Some(utility),
                },
            })
            .downgrade();

        StrategyForInfoView { v }
    }
    pub fn pick_move(&self, info: &INFO) -> Option<INFO::Move> {
        let move_probabilities = self.get_move_probabilities(info);

        if move_probabilities.v.terminal_utility.is_some() {
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

            if mark < cumulative {
                return Some(m);
            }
        }

        panic!("Move probabilities did not sum to 1.0!")
    }
}

impl<INFO: VisibleInfo> Default for Strategy<INFO> {
    fn default() -> Self {
        Self {
            strategy: DashMap::default(),
        }
    }
}

#[derive(Debug)]
pub struct StrategyForInfo<INFO: VisibleInfo> {
    strategy: FxHashMap<INFO::Move, AtomicProbability>,
    terminal_utility: Option<UtilityForAllPlayers>,
}

impl<INFO: VisibleInfo> StrategyForInfo<INFO> {
    pub fn iter(&self) -> Iter<INFO::Move, AtomicProbability> {
        self.strategy.iter()
    }

    pub fn immediate_utility(&self) -> Option<&UtilityForAllPlayers> {
        self.terminal_utility.as_ref()
    }

    pub fn number_of_moves(&self) -> usize {
        self.strategy.len()
    }
}

pub struct StrategyForInfoView<'a, INFO: VisibleInfo> {
    v: Ref<'a, INFO, StrategyForInfo<INFO>>,
}

impl<'a, INFO: VisibleInfo> StrategyForInfoView<'a, INFO> {
    pub fn iter(&self) -> Iter<INFO::Move, AtomicProbability> {
        self.v.iter()
    }

    pub fn immediate_utility(&self) -> Option<&UtilityForAllPlayers> {
        self.v.immediate_utility()
    }

    pub fn move_count(&self) -> usize {
        self.v.strategy.len()
    }

    pub fn info(&self) -> &INFO {
        self.v.key()
    }

    pub fn move_probability(&self, m: &INFO::Move) -> Probability {
        self.v
            .strategy
            .get(m)
            .expect("All moves for infoset must have probability")
            .load(Ordering::Relaxed)
    }
}

impl<'a, INFO: VisibleInfo> Debug for StrategyForInfoView<'a, INFO> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let strategy_for_info = &*self.v;
        strategy_for_info.fmt(f)
    }
}
