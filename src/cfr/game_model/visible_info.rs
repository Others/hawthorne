use crate::cfr::game_model::{OracleGamestate, PlayerNumber, UtilityForAllPlayers};
use std::fmt::Debug;
use std::hash::Hash;

pub trait VisibleInfo: Hash + Eq + Debug + Sized + Send + Sync {
    type Move: Hash + Eq + Clone + Copy + Debug + Send + Sync;

    type Gamestate: OracleGamestate<Self>;

    fn players_playing(&self) -> PlayerNumber;

    fn turn(&self) -> PlayerNumber;

    fn run_for_moves(&self, f: impl FnMut(Self::Move)) -> Option<UtilityForAllPlayers>;
}
