use crate::cfr::game_model::{Probability, VisibleInfo};
use crate::cfr::strategy_generation::workspace_data::StrategyGenerationProgress;

pub(crate) fn update_strategy_from_regret<INFO: VisibleInfo>(
    strategy_generation_progress: &StrategyGenerationProgress<INFO>,
) {
    strategy_generation_progress.consume_updated_infosets(|i| {
        let mut total_regret = 0.0;
        let number_of_moves = i.moves().len();

        // First we get the regret of every move, and write it in as a fake probability
        // This is a hack to avoid having to store any intermediate values
        for move_with_data in i.moves() {
            total_regret += move_with_data.d.regret();
        }

        // Next we grab those stored regret values and divide them by the total regret
        let mut total_probability = 0.0;
        for move_with_data in i.moves() {
            let new_probability = if total_regret > 0.0 {
                move_with_data.d.regret() / total_regret
            } else {
                1.0 / number_of_moves as Probability
            };

            total_probability += new_probability;

            debug_assert!(new_probability >= 0.0);
            debug_assert!(new_probability <= 1.0);
            debug_assert!(total_probability <= 1.1);

            move_with_data.d.write_move_probability(new_probability);
        }
    });
}
