use crate::cfr::game_model::{Utility, VisibleInfo};
use crate::cfr::strategy_generation::CfrProgressData;
use std::sync::atomic::Ordering;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

pub(crate) fn update_strategy_from_regret<INFO: VisibleInfo>(
    progress_data: &CfrProgressData<INFO>,
) {
    progress_data.witnessed.par_iter().for_each(|entry| {
        let info = entry.key();

        let strategy =  progress_data.current_strategy.strategy.get_mut(
            info
        ).unwrap();

        let mut total_regret = 0.0;
        let number_of_moves = strategy.number_of_moves() as Utility;

        // First we get the regret of every move, and write it in as a fake probability
        // This is a hack to avoid having to store any intermediate values
        let regret_for_moves = progress_data
            .cumulative_regret
            .get_regret_for_all_moves(info);
        for (m, probability) in strategy.iter() {
            let move_regret = regret_for_moves.get_move_regret(m);
            total_regret += move_regret;
            probability.store(move_regret, Ordering::Relaxed);
        }

        // Next we grab those stored regret values and divide them by the total regret
        for (_, probability) in strategy.iter() {
            let stored_regret_value = probability.load(Ordering::Relaxed);

            let new_probability = if total_regret > 0.0 {
                stored_regret_value / total_regret
            } else {
                1.0 / number_of_moves
            };

            probability.store(new_probability, Ordering::Relaxed);
        }
    });
}
