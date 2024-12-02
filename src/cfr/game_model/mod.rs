mod gamestate_sampler;
mod oracle_gamestate;
mod utility;
mod visible_info;

pub use gamestate_sampler::*;
pub use oracle_gamestate::*;
pub use utility::*;
pub use visible_info::*;

use atomic_float::AtomicF64;

// FIXME: Probably should be newtypes
pub type PlayerNumber = usize;
pub type Probability = f64;
pub type AtomicProbability = AtomicF64;
