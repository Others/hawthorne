// use crate::cfr::game_model::{
//     AtomicUtility, Moves, Probability, Utility, UtilityForAllPlayers, VisibleInfo,
// };
// use dashmap::mapref::one::Ref;
// use dashmap::DashMap;
// use rustc_hash::{FxHashMap, FxHasher};
// use std::hash::BuildHasherDefault;
// use std::sync::atomic::Ordering;
//
// #[derive(Debug)]
// pub struct Regret<INFO: VisibleInfo> {
//     regret_of_move_for_turn_player:
//         DashMap<INFO, FxHashMap<INFO::Move, AtomicUtility>, BuildHasherDefault<FxHasher>>,
// }
//
// impl<INFO: VisibleInfo> Regret<INFO> {
//     pub fn get_regret_for_all_moves(&self, info: &INFO) -> RegretForAllMoves<INFO> {
//         let regret_for_moves = self
//             .regret_of_move_for_turn_player
//             .get(info)
//             .expect("Regret should be present for every infoset of interest");
//
//         RegretForAllMoves { regret_for_moves }
//     }
//
//     pub fn get_regret_for_all_moves_or_default(&self, info: &INFO) -> RegretForAllMoves<INFO> {
//         let regret_for_moves = match self.regret_of_move_for_turn_player.get(info) {
//             Some(v) => v,
//             None => self.get_utility_ref_locking(info),
//         };
//
//         RegretForAllMoves { regret_for_moves }
//     }
//
//     pub fn denote_terminal(&self, info: &INFO) {
//         if !self.regret_of_move_for_turn_player.contains_key(info) {
//             self.regret_of_move_for_turn_player
//                 .insert(info.clone(), FxHashMap::default());
//         }
//     }
//
//     pub fn accumulate_positive_regret(
//         &self,
//         info: &INFO,
//         m: &INFO::Move,
//         strategic_utility: &UtilityForAllPlayers,
//         utility_after_move: &UtilityForAllPlayers,
//         discount: Probability,
//     ) {
//         let new_positive_regret =
//             (utility_after_move.get(info.turn()) - strategic_utility.get(info.turn())).max(0.0);
//
//         let r = match self.regret_of_move_for_turn_player.get(info) {
//             Some(v) => v,
//             None => self.get_utility_ref_locking(info),
//         };
//
//         r.get(m)
//             .expect("All moves should have regret for infoset of interest")
//             .fetch_add(new_positive_regret * discount, Ordering::Relaxed);
//     }
//
//     fn get_utility_ref_locking(
//         &self,
//         info: &INFO,
//     ) -> Ref<INFO, FxHashMap<INFO::Move, AtomicUtility>> {
//         let v = self
//             .regret_of_move_for_turn_player
//             .entry(info.clone())
//             .or_insert_with(|| match info.moves() {
//                 Moves::PossibleMoves(moves) => moves
//                     .into_iter()
//                     .map(|m| (m, AtomicUtility::from(0.0)))
//                     .collect(),
//                 Moves::Terminal { .. } => FxHashMap::default(),
//             });
//
//         v.downgrade()
//     }
// }
//
// impl<INFO: VisibleInfo> Default for Regret<INFO> {
//     fn default() -> Self {
//         Self {
//             regret_of_move_for_turn_player: DashMap::default(),
//         }
//     }
// }
//
// pub struct RegretForAllMoves<'a, INFO: VisibleInfo> {
//     regret_for_moves: Ref<'a, INFO, FxHashMap<INFO::Move, AtomicUtility>>,
// }
//
// impl<'a, INFO: VisibleInfo> RegretForAllMoves<'a, INFO> {
//     pub fn get_move_regret(&self, m: &INFO::Move) -> Utility {
//         self.regret_for_moves
//             .get(m)
//             .expect("All moves for info should have utility")
//             .load(Ordering::Relaxed)
//     }
//
//     pub fn get_total_regret(&self) -> Utility {
//         self.regret_for_moves
//             .values()
//             .map(|it| it.load(Ordering::Relaxed))
//             .sum()
//     }
// }
