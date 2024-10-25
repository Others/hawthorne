mod utility;

pub use utility::*;

use atomic_float::AtomicF64;
use std::fmt::Debug;
use std::hash::Hash;

// FIXME: Probably should be a newtype
pub type PlayerNumber = usize;

pub type Probability = f64;
pub type AtomicProbability = AtomicF64;

pub trait OracleGamestate<INFO: VisibleInfo>: Clone + Debug + Sized + Send + Sync {
    fn info_for_turn_player(&self) -> INFO;

    fn turn(&self) -> PlayerNumber;

    fn advance(&self, m: &INFO::Move) -> Self;
}

pub trait VisibleInfo: Hash + Eq + Debug + Sized + Send + Sync {
    type Move: Hash + Eq + Clone + Copy + Debug + Send + Sync;

    type Gamestate: OracleGamestate<Self>;

    fn max_players(&self) -> PlayerNumber;

    fn turn(&self) -> PlayerNumber;

    fn run_for_moves(&self, f: impl FnMut(Self::Move)) -> Option<UtilityForAllPlayers>;

    fn get_all_possible_gamestates(
        &self,
    ) -> impl Iterator<Item = (Self::Gamestate, Probability)> + Send;

    fn gamestate_sampler(&self) -> impl GamestateSampler<Info = Self>;
}

pub trait GamestateSampler: Send {
    type Info: VisibleInfo;

    fn sample(&mut self) -> (<Self::Info as VisibleInfo>::Gamestate, Probability);
}

#[derive(Debug)]
pub struct RandomGamestateIterator<SAMPLER: GamestateSampler> {
    cumulative_probability: Probability,
    stopping_probability: Probability,

    cumulative_n: u32,
    stopping_n: u32,

    sampler: SAMPLER,
}

impl<SAMPLER: GamestateSampler> RandomGamestateIterator<SAMPLER> {
    pub fn new(sampler: SAMPLER, stopping_probability: Probability, stopping_n: u32) -> Self {
        Self {
            cumulative_probability: 0.0,
            stopping_probability,
            cumulative_n: 0,
            stopping_n,
            sampler,
        }
    }
}

impl<SAMPLER: GamestateSampler> Iterator for RandomGamestateIterator<SAMPLER> {
    type Item = (<SAMPLER::Info as VisibleInfo>::Gamestate, Probability);

    fn next(&mut self) -> Option<Self::Item> {
        // println!("{:?}", self.cumulative_probability);
        if self.cumulative_probability > self.stopping_probability {
            return None;
        }

        if self.cumulative_n > self.stopping_n {
            return None;
        }

        let (g, p) = self.sampler.sample();
        self.cumulative_probability += p;
        self.cumulative_n += 1;

        Some((g, p))
    }
}
