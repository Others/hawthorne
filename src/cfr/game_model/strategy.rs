use crate::cfr::game_model::{Moves, Probability, VisibleInfo};
use std::borrow::Cow;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct StrategyForVisibleInfoView<'a, INFO: VisibleInfo> {
    move_probabilities: Cow<'a, HashMap<INFO::Move, Probability>>,
}

impl<'a, INFO: VisibleInfo> StrategyForVisibleInfoView<'a, INFO> {
    pub fn get_probability(&self, m: &INFO::Move) -> Probability {
        *self
            .move_probabilities
            .get(m)
            .expect("Strategies must be complete")
    }
}

#[derive(Debug, Clone)]
pub struct Strategy<INFO: VisibleInfo> {
    pub(crate) probability: HashMap<INFO, HashMap<INFO::Move, Probability>>,
}

impl<INFO: VisibleInfo> Strategy<INFO> {
    pub fn get_probability_mut(&mut self, info: &INFO) -> &mut HashMap<INFO::Move, Probability> {
        self.probability
            .get_mut(info)
            .expect("CFR should generate a complete strategy!")
    }

    pub fn get_probabilities(&self, info: &INFO) -> StrategyForVisibleInfoView<INFO> {
        let move_probabilities = self
            .probability
            .get(info)
            .expect("Strategies must be complete");

        StrategyForVisibleInfoView {
            move_probabilities: Cow::Borrowed(move_probabilities),
        }
    }

    pub fn get_move_probability(&self, info: &INFO, m: &INFO::Move) -> Probability {
        self.get_probabilities(info).get_probability(m)
    }

    pub fn pick_move(&self, info: &INFO) -> Option<INFO::Move> {
        let move_probabilities = self.get_probabilities(info);

        let mut moves = match info.moves() {
            Moves::PossibleMoves(v) => v,
            Moves::Terminal { .. } => return None,
        };

        fastrand::shuffle(&mut moves);

        let mark = fastrand::f64();
        let mut cumulative = 0.0;

        for m in moves {
            let prob = *move_probabilities
                .move_probabilities
                .get(&m)
                .expect("Should have a probability for every move!");
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
            probability: HashMap::default(),
        }
    }
}
