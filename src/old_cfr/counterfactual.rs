use crate::cfr::game_model::{Moves, OracleGamestate, Probability, Strategy, Utility, VisibleInfo};
use rustc_hash::FxHashMap;
use std::borrow::Cow;
use std::cell::Cell;

pub(crate) struct GamestateChain<INFO: VisibleInfo> {
    chain: Vec<ChainItem<INFO>>,
}

#[derive(Clone, Debug)]
pub(crate) struct CounterfactualValue {
    move_probability: Cell<Probability>,
    after_move_utility: Cell<Utility>,
    updated_epoch: Cell<i32>,
}

impl CounterfactualValue {
    pub(crate) fn move_probability(&self) -> Probability {
        self.move_probability.get()
    }

    pub(crate) fn after_move_utility(&self) -> Utility {
        self.after_move_utility.get()
    }

    pub(crate) fn accumulate(
        &self,
        epoch: i32,
        move_probability: Probability,
        additional_utility: Utility,
    ) {
        self.move_probability.set(move_probability);
        self.after_move_utility
            .set(self.after_move_utility() + additional_utility);
        self.updated_epoch.set(epoch);
    }
}

impl Default for CounterfactualValue {
    fn default() -> Self {
        Self {
            move_probability: Cell::new(0.0),
            after_move_utility: Cell::new(0.0),
            updated_epoch: Cell::new(-1),
        }
    }
}

#[derive(Clone, Debug)]
struct ChainItem<INFO: VisibleInfo> {
    gamestate_before_move: INFO::Gamestate,
    move_selected: INFO::Move,
    move_selection_probability: Probability,
    gamestate_after_move: INFO::Gamestate,
}

pub(crate) fn populate_counterfactual_values<INFO: VisibleInfo>(
    iteration: i32,
    starting_info: &INFO,
    starting_moves: &[INFO::Move],
    current_strategy: &Strategy<INFO>,
    counterfactual_values: &FxHashMap<(Cow<INFO>, Cow<INFO::Move>), CounterfactualValue>,
    event_stack: &mut Vec<GamestateChain<INFO>>,
) {
    for (gamestate, state_probability) in starting_info.get_all_possible_gamestates() {
        let strategy_for_visible_info = current_strategy.get_probabilities(starting_info);

        // Push all the starting moves on the stack
        for starting_move in starting_moves {
            let move_probability = strategy_for_visible_info.get_probability(starting_move);

            event_stack.push(GamestateChain {
                chain: vec![ChainItem {
                    gamestate_before_move: gamestate.clone(),
                    move_selected: starting_move.clone(),
                    move_selection_probability: move_probability,
                    gamestate_after_move: gamestate.advance(starting_move),
                }],
            });
        }

        // FIXME: This code can let you explore the same node multiple times for the same ground state
        // That means expected utility can go above `1` which is a little broken
        while let Some(current_chain) = event_stack.pop() {
            let final_chainlink = current_chain.chain.last().unwrap();

            let current_gamestate = &final_chainlink.gamestate_after_move;
            let current_info = final_chainlink.gamestate_after_move.info_for_turn_player();

            let moves = current_info.moves();
            match moves {
                Moves::PossibleMoves(moves) => {
                    let strategy = current_strategy.get_probabilities(&current_info);
                    for m in moves {
                        let mut new_chain = Vec::with_capacity(current_chain.chain.len() + 1);
                        new_chain.extend_from_slice(&current_chain.chain);

                        let move_prob = strategy.get_probability(&m);

                        new_chain.push(ChainItem {
                            gamestate_before_move: current_gamestate.clone(),
                            gamestate_after_move: current_gamestate.advance(&m),
                            move_selected: m,
                            move_selection_probability: move_prob,
                        });

                        event_stack.push(GamestateChain { chain: new_chain })
                    }
                }
                Moves::Terminal { utility } => {
                    for (_, infoset_of_interest) in current_chain.chain.iter().enumerate() {
                        let mut reached_terminal_from_infoset = state_probability;

                        // FIXME: Which of these is right?
                        // Thought: Should this equal
                        // for a in idx..current_chain.chain.len() {
                        //     reached_terminal_from_infoset *= current_chain.chain[a].move_selection_probability;
                        // }
                        for a in 0..current_chain.chain.len() {
                            reached_terminal_from_infoset *=
                                current_chain.chain[a].move_selection_probability;
                        }

                        let utility_of_terminal =
                            utility.get(infoset_of_interest.gamestate_before_move.turn());

                        let counterfactual_value_for_turn_player =
                            reached_terminal_from_infoset * utility_of_terminal;

                        let key = (
                            Cow::Owned(
                                infoset_of_interest
                                    .gamestate_before_move
                                    .info_for_turn_player(),
                            ),
                            Cow::Borrowed(&infoset_of_interest.move_selected),
                        );

                        let counterfactual_value = counterfactual_values.get(&key).unwrap();

                        counterfactual_value.accumulate(
                            iteration,
                            infoset_of_interest.move_selection_probability,
                            counterfactual_value_for_turn_player,
                        );
                    }
                }
            }
        }
    }
}
