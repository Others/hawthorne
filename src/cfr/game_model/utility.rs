use crate::cfr::game_model::{PlayerNumber, Probability};
use atomic_float::AtomicF64;
use tinyvec::{tiny_vec, TinyVec};

pub type Utility = f64;
pub type AtomicUtility = AtomicF64;

// FIXME: Should be a trait implemented for arrays or Vec
#[derive(Debug, Clone)]
pub struct UtilityForAllPlayers {
    pub(crate) util: TinyVec<[Utility; 4]>,
}

impl UtilityForAllPlayers {
    pub fn get(&self, player_number: PlayerNumber) -> Utility {
        *self.util.get(player_number).unwrap_or(&0.0)
    }

    pub fn accumulate(&mut self, other: &UtilityForAllPlayers, discount: Probability) {
        while self.util.len() < other.util.len() {
            self.util.push(0.0);
        }

        for (i, u) in other.util.iter().enumerate() {
            self.util[i] += u * discount;
        }
    }
}

impl Default for UtilityForAllPlayers {
    fn default() -> Self {
        Self { util: tiny_vec![] }
    }
}
