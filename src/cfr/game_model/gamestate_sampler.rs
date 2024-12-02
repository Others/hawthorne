use crate::cfr::game_model::{Probability, VisibleInfo};

pub trait GamestateSampler: Clone + Send {
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
