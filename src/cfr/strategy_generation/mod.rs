mod cfr_algorithm_impl;
mod strategy;
pub mod strategy_generator;
mod update_strategy;
mod workspace_data;

// FIXME: Next steps
//     1. Abstract out to a struct so the strategy can be refined as the game advances
//     2. Abstract out configuration decisions out to a type parameter
//     3. Fix bridge infoset abstraction
//     4. Cleanup code
