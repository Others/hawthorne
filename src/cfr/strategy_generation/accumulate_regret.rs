use crate::cfr::game_model::{AtomicProbability, OracleGamestate, Probability, RandomGamestateIterator, Utility, UtilityForAllPlayers, VisibleInfo};
use crate::cfr::strategy_generation::counterfactual_workspace::{
    BufferedValue, BufferedValueTimestamp, CounterfactualWorkspace, ThreadLocalWorkspace,
};
use crate::cfr::strategy_generation::strategy::StrategyForInfoView;
use crate::cfr::strategy_generation::CfrProgressData;
use rayon::iter::{ParallelBridge, ParallelIterator};
use std::sync::atomic::Ordering;

pub(crate) fn add_to_regret<INFO: VisibleInfo>(
    starting_info: &INFO,
    progress_data: &CfrProgressData<INFO>,
    workspace: &CounterfactualWorkspace<INFO>,
    iteration: u32,
) {
    let gamestates = RandomGamestateIterator::new(starting_info.gamestate_sampler(), 0.1, 1);
    // let par_gamestates = ParallelBridge::par_bridge(gamestates.enumerate());

    gamestates.enumerate().for_each(|(i, (starting_gamestate, gamestate_probability))| {
        let timestamp = (iteration, i);

        let mut thread_local_workspace = workspace.workspace.get_or_default().borrow_mut();
        let buffer: &mut ThreadLocalWorkspace<INFO> = &mut *thread_local_workspace;
        buffer.clear();

        buffer.push_stack(starting_gamestate, starting_info.clone());

        let mut n: u32 = 0;
        while let Some((current_gamestate, current_info)) = buffer.pop_stack() {
            n += 1;
            if n.count_ones() == 1 {
                println!("Iteration 2^{}", n.ilog2());
            }

            // If we already got this value ready, we can skip it
            if buffer.buffered_value_ready(&current_info, timestamp) {
                continue;
            }

            let strategy_for_current_info = progress_data
                .current_strategy
                .get_move_probabilities(&current_info);

            // If the gamestate is terminal, we need to mark it as terminal
            if let Some(immediate_utility) = strategy_for_current_info.immediate_utility() {
                mark_terminal(
                    progress_data,
                    buffer,
                    timestamp,
                    current_info,
                    immediate_utility,
                );
                // Nothing needs pushed here, since there are no child gamestates
                continue;
            }

            // Save a spot for this item, in case the moves bellow need done first
            buffer.push_stack(current_gamestate.clone(), current_info.clone());

            let mut strategy_util = UtilityForAllPlayers::default();
            let mut complete = true;
            debug_assert!(strategy_for_current_info.move_count() > 0);
            for e in strategy_for_current_info.iter() {
                update_strategy_utility_for_move(
                    progress_data,
                    buffer,
                    &current_info,
                    &current_gamestate,
                    timestamp,
                    e,
                    &mut strategy_util,
                    &mut complete,
                );
            }

            if complete {
                // Retrieve from being pushed earlier in the function
                let (_, current_info) = buffer.pop_stack().unwrap();
                accumulate_regret_with_complete_children(
                    iteration,
                    progress_data,
                    buffer,
                    timestamp,
                    gamestate_probability,
                    &current_gamestate,
                    current_info,
                    strategy_util,
                    strategy_for_current_info,
                );
            }
        }
    });
}

fn mark_terminal<INFO: VisibleInfo>(
    progress_data: &CfrProgressData<INFO>,
    buffer: &mut ThreadLocalWorkspace<INFO>,
    timestamp: BufferedValueTimestamp,
    info: INFO,
    utility_for_all_players: &UtilityForAllPlayers,
) {
    progress_data.cumulative_regret.denote_terminal(&info);

    buffer.insert_buffered_value(
        info,
        BufferedValue {
            utility: utility_for_all_players.clone(),
            update_timestamp: timestamp,
        },
    );
}

fn update_strategy_utility_for_move<INFO: VisibleInfo>(
    progress_data: &CfrProgressData<INFO>,
    buffer: &mut ThreadLocalWorkspace<INFO>,
    info_before_move: &INFO,
    gamestate_before_move: &INFO::Gamestate,
    timestamp: BufferedValueTimestamp,
    (m, prob): (&INFO::Move, &AtomicProbability),
    strategy_util: &mut UtilityForAllPlayers,
    complete: &mut bool,
) {
    let move_probability = prob.load(Ordering::Relaxed);
    let state_after_move = gamestate_before_move.advance(m);
    let info_after_move = state_after_move.info_for_turn_player();

    let utility_after_move = buffer.get_buffered_value(&info_after_move, timestamp);

    match utility_after_move {
        Some(util) => {
            strategy_util.accumulate(&util.utility, move_probability);
        }
        None => {
            let skip = should_skip_due_to_mccfr(
                progress_data,
                info_before_move,
                m
            );

            if skip {
                buffer.insert_buffered_value(
                    info_after_move,
                    BufferedValue {
                        utility: UtilityForAllPlayers::default(),
                        update_timestamp: timestamp,
                    },
                );
            } else {
                *complete = false;
                buffer.push_stack(state_after_move, info_after_move);
            }
        }
    }
}

fn should_skip_due_to_mccfr<INFO: VisibleInfo>(
    progress_data: &CfrProgressData<INFO>,
    info_before_move: &INFO,
    m: &INFO::Move
) -> bool {
    let epsilon = 0.05;
    let beta = (1.0 as Utility).powi(1);
    let gamma = 1000.0;

    // Technically in MCCFR you're supposed to use cumulative counterfactual for this
    // But since we're in CFR+ we don't really have that and this is approximately the same thing
    let move_regret = progress_data.cumulative_regret.get_regret_for_all_moves_or_default(info_before_move);
    let this_move_regret = move_regret.get_move_regret(m);
    let total_regret = move_regret.get_total_regret();

    // We want to release this lock ASAP!
    drop(move_regret);

    let p_numerator = beta + gamma * this_move_regret;
    let p_denominator = beta + total_regret;
    let p = (p_numerator / p_denominator).max(epsilon);

    fastrand::f64() > p
}

fn accumulate_regret_with_complete_children<INFO: VisibleInfo>(
    iter: u32,

    progress_data: &CfrProgressData<INFO>,
    buffer: &mut ThreadLocalWorkspace<INFO>,

    timestamp: BufferedValueTimestamp,
    starting_gamestate_probability: Probability,

    gamestate_before_move: &INFO::Gamestate,
    info_before_move: INFO,

    strategy_util: UtilityForAllPlayers,
    strategy_for_gamestate_before_move: StrategyForInfoView<INFO>,
) {
    progress_data.witnessed.insert(info_before_move.clone());

    for (m, _) in strategy_for_gamestate_before_move.iter() {
        let state_after_move = gamestate_before_move.advance(m);
        let info_after_move = state_after_move.info_for_turn_player();

        let utility_after_move = buffer.get_buffered_value(&info_after_move, timestamp);

        progress_data.cumulative_regret.accumulate_positive_regret(
            &info_before_move,
            m,
            &strategy_util,
            &utility_after_move.unwrap().utility,
            starting_gamestate_probability * iter as Probability,
        );
    }

    buffer.insert_buffered_value(
        info_before_move,
        BufferedValue {
            utility: strategy_util,
            update_timestamp: timestamp,
        },
    );
}
