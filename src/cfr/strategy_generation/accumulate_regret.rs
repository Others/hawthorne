use crate::cfr::game_model::{
    Probability, RandomGamestateIterator, UtilityForAllPlayers, VisibleInfo,
};
use crate::cfr::strategy_generation::workspace_data::data_for_infoset::DataForInfoSet;
use crate::cfr::strategy_generation::workspace_data::data_for_move::DataForMove;
use crate::cfr::strategy_generation::workspace_data::timestamp::Timestamp;
use crate::cfr::strategy_generation::workspace_data::{
    StrategyGenerationProgress, ThreadLocalWorkStack,
};
use rayon::iter::IndexedParallelIterator;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::sync::Arc;

pub(crate) fn add_to_regret<INFO: VisibleInfo>(
    starting_info: &INFO,
    strategy_generation_progress: &StrategyGenerationProgress<INFO>,
    iteration: u32,
) {
    let mut gamestates = RandomGamestateIterator::new(starting_info.gamestate_sampler(), 0.1, 1);
    // let par_gamestates = ParallelBridge::par_bridge(gamestates.enumerate());

    let mut batch = Vec::new();
    while batch.len() < 1 {
        batch.push(gamestates.next().unwrap())
    }

    batch.into_par_iter().enumerate().for_each(
        |(i, (starting_gamestate, gamestate_probability))| {
            let timestamp = Timestamp::new(iteration, i);

            let mut workstack = strategy_generation_progress.thread_local_workstack();
            workstack.push_default(starting_gamestate);

            let mut n: u32 = 0;
            let mut already_ready: u32 = 1;
            let mut terminals: u32 = 1;
            while let Some((data_for_info, gamestate)) = workstack.pop() {
                // println!("Gamestate: {:?}", gamestate);

                n += 1;
                if n.count_ones() == 1 {
                    println!(
                        "Iteration 2^{} (already ready 2^{}, terminal 2^{})",
                        n.ilog2(),
                        already_ready.ilog2(),
                        terminals.ilog2()
                    );
                    // println!("Gamestate: {:?}", gamestate);
                    println!("Stack size: {:?}", workstack.len());

                    println!(
                        "Size DataForInfoset {:?}",
                        size_of::<DataForInfoSet<INFO>>()
                    );
                    println!("Size DataForMove {:?}", size_of::<DataForMove>());
                    println!(
                        "Size Info {:?}",
                        size_of::<INFO>()
                    );
                    println!(
                        "Size Gamestate {:?}",
                        size_of::<INFO::Gamestate>()
                    );
                    // workstack.print_debug();

                    if n.ilog2() == 23 {
                        panic!("CAPPED!")
                    }
                }

                // If we already got this value ready, we can skip it
                // FIXME: Is this pointless? Won't we only push if it's not ready?
                if data_for_info.is_ready(timestamp) {
                    already_ready += 1;
                    continue;
                }

                // If the gamestate is terminal, we need to mark it as terminal
                if data_for_info.increment_terminal_counterfactual(gamestate_probability, timestamp)
                {
                    terminals += 1;
                    strategy_generation_progress.mark_as_completed(data_for_info, timestamp);
                    continue;
                }

                // Save a spot for this item, in case the moves bellow need done first
                workstack.push(Arc::clone(&data_for_info), gamestate.clone());

                let moves = data_for_info.moves();
                let n_moves = moves.len();
                let mut strategy_util = UtilityForAllPlayers::default();
                let mut complete = true;
                debug_assert!(data_for_info.move_count() > 0);
                for move_with_data in moves {
                    update_strategy_utility_for_move(
                        strategy_generation_progress,
                        &mut *workstack,
                        timestamp,
                        &gamestate,
                        n_moves,
                        &move_with_data.m,
                        &move_with_data.d,
                        &mut strategy_util,
                        &mut complete,
                    );
                }

                if complete {
                    // Retrieve from being pushed earlier in the function
                    workstack.pop();
                    accumulate_regret_with_complete_children(
                        strategy_generation_progress,
                        timestamp,
                        gamestate_probability,
                        data_for_info,
                        gamestate,
                        strategy_util,
                    );
                }
            }
        },
    );
}

fn update_strategy_utility_for_move<INFO: VisibleInfo>(
    strategy_generation_progress: &StrategyGenerationProgress<INFO>,
    workstack: &mut ThreadLocalWorkStack<INFO>,

    timestamp: Timestamp,

    gamestate_before_move: &INFO::Gamestate,
    n_moves: usize,

    m: &INFO::Move,
    data_for_move: &DataForMove,
    strategy_util: &mut UtilityForAllPlayers,
    complete: &mut bool,
) {
    let move_probability = data_for_move.load_move_probability(n_moves);

    let cached_utility_after_move = data_for_move.utility_after_move();
    if let Some(util) = cached_utility_after_move.get(timestamp) {
        strategy_util.accumulate(&util, move_probability);
        return;
    }

    let (data_after_move, state_after_move) = data_for_move.get_post_move_infoset(
        strategy_generation_progress,
        timestamp,
        gamestate_before_move,
        m,
    );
    let utility_after_move = data_after_move.get_iteration_utility_if_ready(timestamp);
    match utility_after_move {
        Some(utility_after_move) => {
            strategy_util.accumulate(&utility_after_move, move_probability);
            cached_utility_after_move.set(utility_after_move, timestamp);
        }
        None => {
            *complete = false;
            workstack.push(data_after_move, state_after_move);
            // let skip = should_skip_due_to_mccfr(progress_data, info_before_move, m);
            //
            // if skip {
            //     buffer.insert_buffered_value(
            //         info_after_move,
            //         BufferedValue {
            //             utility: UtilityForAllPlayers::default(),
            //             update_timestamp: timestamp,
            //         },
            //     );
        }
    }
}

// fn should_skip_due_to_mccfr<INFO: VisibleInfo>(
//     progress_data: &CfrProgressData<INFO>,
//     info_before_move: &INFO,
//     m: &INFO::Move,
// ) -> bool {
//     let epsilon = 0.05;
//     let beta = (1.0 as Utility).powi(1);
//     let gamma = 1000.0;
//
//     // Technically in MCCFR you're supposed to use cumulative counterfactual for this
//     // But since we're in CFR+ we don't really have that and this is approximately the same thing
//     let move_regret = progress_data
//         .cumulative_regret
//         .get_regret_for_all_moves_or_default(info_before_move);
//     let this_move_regret = move_regret.get_move_regret(m);
//     let total_regret = move_regret.get_total_regret();
//
//     // We want to release this lock ASAP!
//     drop(move_regret);
//
//     let p_numerator = beta + gamma * this_move_regret;
//     let p_denominator = beta + total_regret;
//     let p = (p_numerator / p_denominator).max(epsilon);
//
//     fastrand::f64() > p
// }

fn accumulate_regret_with_complete_children<INFO: VisibleInfo>(
    strategy_generation_progress: &StrategyGenerationProgress<INFO>,
    timestamp: Timestamp,

    starting_gamestate_probability: Probability,

    info_before_move: Arc<DataForInfoSet<INFO>>,
    gamestate_before_move: INFO::Gamestate,
    strategy_util: UtilityForAllPlayers,
) {
    // We need to do three steps here:
    // - Update the counterfactual values for this specific item on the item itself
    // - Update the regret by accumulating the regret on the move using the counterfactual values
    // - Mark this item as complete

    info_before_move.ready_with_counterfactual(
        strategy_util,
        starting_gamestate_probability,
        timestamp,
    );

    for move_with_data in info_before_move.moves() {
        move_with_data.d.accumulate_regret(
            strategy_generation_progress,
            timestamp,
            &info_before_move,
            &gamestate_before_move,
            &move_with_data.m,
        )
    }

    strategy_generation_progress.mark_as_completed(info_before_move, timestamp);
}
