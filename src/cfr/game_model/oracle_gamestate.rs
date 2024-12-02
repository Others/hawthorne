use crate::cfr::game_model::{PlayerNumber, VisibleInfo};
use std::fmt::Debug;
use std::hash::Hash;

pub trait OracleGamestate<INFO: VisibleInfo>: Hash + Clone + Debug + Sized + Send + Sync {
    fn info_for_turn_player(&self) -> INFO;

    fn players_playing(&self) -> PlayerNumber;

    fn turn(&self) -> PlayerNumber;

    fn advance(&self, m: &INFO::Move) -> Self;
}
