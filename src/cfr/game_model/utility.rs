use crate::cfr::game_model::{PlayerNumber, Probability};
use atomic_float::AtomicF64;

pub type Utility = f64;
pub type AtomicUtility = AtomicF64;

// FIXME: Should be a trait implemented for arrays or Vec

const MAX_PLAYERS: usize = 4;

#[derive(Debug, Clone, Copy)]
pub struct UtilityForAllPlayers {
    pub(crate) util: [Utility; MAX_PLAYERS],
}

impl UtilityForAllPlayers {
    pub fn get(&self, player_number: PlayerNumber) -> Utility {
        *self.util.get(player_number).unwrap_or(&0.0)
    }

    pub fn accumulate(&mut self, other: &UtilityForAllPlayers, discount: Probability) {
        // while self.util.len() < other.util.len() {
        //     self.util.push(0.0);
        // }

        for (i, u) in other.util.iter().enumerate() {
            self.util[i] += u * discount;
        }
    }

    pub fn reduce(&mut self, discount: Probability) {
        for u in &mut self.util {
            *u *= discount;
        }
    }

    pub const fn const_default() -> UtilityForAllPlayers {
        Self {
            util: [0.0; MAX_PLAYERS],
        }
    }
}

impl Default for UtilityForAllPlayers {
    fn default() -> Self {
        Self {
            util: Default::default(),
        }
    }
}
