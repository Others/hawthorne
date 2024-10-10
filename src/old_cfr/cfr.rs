use crate::cfr::game_model::{Moves, OracleGamestate, Probability, Strategy, VisibleInfo};
use crate::old_cfr::counterfactual::{
    populate_counterfactual_values, CounterfactualValue, GamestateChain,
};
use crate::old_cfr::regret::{add_to_cumulative_regret, RegretStorage};
use rustc_hash::FxHashMap;
use std::borrow::Cow;
use std::collections::HashMap;

// TODO: It makes things simpler if only the `starting_info` needs to share oracle values
pub fn generate_strategy<INFO: VisibleInfo>(
    starting_info: INFO,
    iterations: u32,
) -> Strategy<INFO> {
    // Since the `INFO` contains the turn player, we don't need a per-player strategy
    let mut current_strategy: Strategy<INFO> = Strategy::default();

    let starting_moves = match starting_info.moves() {
        Moves::PossibleMoves(moves) => moves,
        Moves::Terminal { .. } => return current_strategy,
    };

    let mut counterfactual_values: FxHashMap<(Cow<INFO>, Cow<INFO::Move>), CounterfactualValue> =
        FxHashMap::default();
    let mut cumulative_regret: FxHashMap<(Cow<INFO>, Cow<INFO::Move>), RegretStorage> =
        FxHashMap::default();
    let mut event_stack: Vec<GamestateChain<INFO>> = Vec::new();

    preinit_maps(
        &starting_info,
        &mut current_strategy,
        &mut counterfactual_values,
        &mut cumulative_regret,
    );

    println!("Preinit complete");

    for i in 0..(iterations as i32) {
        run_cfr_loop(
            i,
            &starting_info,
            &starting_moves,
            &mut current_strategy,
            &counterfactual_values,
            &cumulative_regret,
            &mut event_stack,
        )
    }

    current_strategy
}

#[inline(never)]
fn preinit_maps<INFO: VisibleInfo>(
    starting_info: &INFO,
    current_strategy: &mut Strategy<INFO>,
    counterfactual_values: &mut FxHashMap<(Cow<INFO>, Cow<INFO::Move>), CounterfactualValue>,
    cumulative_regret: &mut FxHashMap<(Cow<INFO>, Cow<INFO::Move>), RegretStorage>,
) {
    let mut infoset = Vec::from(&[starting_info.clone()]);
    while let Some(info) = infoset.pop() {
        if !current_strategy.probability.contains_key(&info) {
            let moves = match info.moves() {
                Moves::PossibleMoves(m) => m,
                Moves::Terminal { .. } => Vec::new(),
            };
            let move_len = moves.len() as Probability;

            current_strategy.probability.insert(
                info.clone(),
                moves.iter().map(|m| (m.clone(), 1.0 / move_len)).collect(),
            );

            for m in &moves {
                counterfactual_values.insert(
                    (Cow::Owned(info.clone()), Cow::Owned(m.clone())),
                    Default::default(),
                );

                cumulative_regret.insert(
                    (Cow::Owned(info.clone()), Cow::Owned(m.clone())),
                    Default::default(),
                );

                for state in info.get_all_possible_gamestates().iter().map(|(g, _)| g) {
                    let new = state.advance(m);
                    infoset.push(new.info_for_turn_player());
                }
            }
        }
    }
}

fn run_cfr_loop<INFO: VisibleInfo>(
    iteration: i32,
    starting_info: &INFO,
    starting_moves: &[INFO::Move],
    current_strategy: &mut Strategy<INFO>,
    counterfactual_values: &FxHashMap<(Cow<INFO>, Cow<INFO::Move>), CounterfactualValue>,
    cumulative_regret: &FxHashMap<(Cow<INFO>, Cow<INFO::Move>), RegretStorage>,
    event_stack: &mut Vec<GamestateChain<INFO>>,
) {
    // Populate the counterfactual values
    populate_counterfactual_values(
        iteration,
        starting_info,
        starting_moves,
        current_strategy,
        counterfactual_values,
        event_stack,
    );

    add_to_cumulative_regret(current_strategy, counterfactual_values, cumulative_regret);

    // Then update the strategy
    update_strategy(current_strategy, cumulative_regret);
}

#[inline(never)]
fn update_strategy<INFO: VisibleInfo>(
    current_strategy: &mut Strategy<INFO>,
    cumulative_regret: &FxHashMap<(Cow<INFO>, Cow<INFO::Move>), RegretStorage>,
) {
    for (info, strategy) in current_strategy.probability.iter_mut() {
        let moves = match info.moves() {
            Moves::PossibleMoves(m) => m,
            Moves::Terminal { .. } => continue,
        };
        let move_count = moves.len();

        if move_count == 1 {
            // *strategy = StrategyForVisibleInfoInternal::EvenSplit;
            continue;
        }

        let mut move_regret = HashMap::with_capacity(move_count);
        let mut total_regret = 0.0;
        let move_count = moves.len();
        for m in moves {
            let regret_for_move = cumulative_regret
                .get(&(Cow::Borrowed(info), Cow::Borrowed(&m)))
                .unwrap()
                .regret();

            total_regret += regret_for_move;
            move_regret.insert(m, regret_for_move);
        }

        if total_regret > 0.0 {
            move_regret.iter_mut().for_each(|(_, v)| {
                *v /= total_regret;
            });
        } else {
            move_regret.iter_mut().for_each(|(_, v)| {
                *v = 1.0 / move_count as Probability;
            });
        }

        *strategy = move_regret;
    }
}
