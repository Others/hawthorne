use crate::bridge::card::{Card, Suit};

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct Contract {
    pub(crate) trump: Option<Suit>,
    pub(crate) n: i32,
    pub(crate) doubling: Doubling,
    pub(crate) declarer_vulnerable: bool,
    pub(crate) defender_vulnerable: bool,
}

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub enum Doubling {
    None,
    Doubled,
    Redoubled,
}

impl Contract {
    pub fn card_defeats_card(&self, current_winner: Card, new_card: Card) -> bool {
        let current_is_trump = Some(current_winner.suit()) == self.trump;
        let new_is_trump = Some(new_card.suit()) == self.trump;
        let outranked = new_card.rank().n() > current_winner.rank().n();

        if current_is_trump {
            return new_is_trump && outranked;
        }

        if new_is_trump {
            return true;
        }

        new_card.suit() == current_winner.suit() && outranked
    }

    // Note: ChatGPT wrote this whole thing, and I take no responsibility
    pub fn declarer_points(&self, tricks_made: i32) -> i32 {
        let contract_tricks = 6 + self.n;
        let delta_tricks = tricks_made - contract_tricks;

        if delta_tricks >= 0 {
            // Contract made
            let multiplier = self.calculate_multiplier();
            let base_trick_score = self.calculate_base_trick_score(multiplier);
            let overtrick_points = self.calculate_overtrick_points(delta_tricks);
            let insult_bonus = self.calculate_insult_bonus();
            let bonus = self.calculate_contract_bonus(base_trick_score);

            base_trick_score + overtrick_points + insult_bonus + bonus
        } else {
            // Contract down
            let undertricks = -delta_tricks;
            let penalty = self.calculate_undertrick_penalty(undertricks);

            -penalty
        }
    }

    /// Calculates the multiplier based on the doubling level.
    fn calculate_multiplier(&self) -> i32 {
        match self.doubling {
            Doubling::None => 1,
            Doubling::Doubled => 2,
            Doubling::Redoubled => 4,
        }
    }

    /// Calculates the base trick score for the bid tricks.
    fn calculate_base_trick_score(&self, multiplier: i32) -> i32 {
        match self.trump {
            None => {
                // Notrump
                if self.n >= 1 {
                    let first_trick_value = 40 * multiplier;
                    let subsequent_tricks_value = 30 * (self.n - 1) * multiplier;
                    first_trick_value + subsequent_tricks_value
                } else {
                    0
                }
            }
            Some(Suit::Clubs) | Some(Suit::Diamonds) => 20 * self.n * multiplier,
            Some(Suit::Hearts) | Some(Suit::Spades) => 30 * self.n * multiplier,
        }
    }

    /// Calculates the points earned from overtricks.
    fn calculate_overtrick_points(&self, delta_tricks: i32) -> i32 {
        if delta_tricks <= 0 {
            return 0;
        }

        match self.doubling {
            Doubling::None => {
                let overtrick_value = match self.trump {
                    None => 30,
                    Some(Suit::Clubs) | Some(Suit::Diamonds) => 20,
                    Some(Suit::Hearts) | Some(Suit::Spades) => 30,
                };
                overtrick_value * delta_tricks
            }
            Doubling::Doubled => {
                let value_per_trick = if self.declarer_vulnerable { 200 } else { 100 };
                value_per_trick * delta_tricks
            }
            Doubling::Redoubled => {
                let value_per_trick = if self.declarer_vulnerable { 400 } else { 200 };
                value_per_trick * delta_tricks
            }
        }
    }

    /// Calculates the insult bonus for doubled or redoubled contracts.
    fn calculate_insult_bonus(&self) -> i32 {
        match self.doubling {
            Doubling::None => 0,
            Doubling::Doubled => 50,
            Doubling::Redoubled => 100,
        }
    }

    /// Determines the appropriate bonus for part-score, game, or slam.
    fn calculate_contract_bonus(&self, base_trick_score: i32) -> i32 {
        match self.n {
            6 => {
                // Small slam
                if self.declarer_vulnerable {
                    1250
                } else {
                    800
                }
            }
            7 => {
                // Grand slam
                if self.declarer_vulnerable {
                    2000
                } else {
                    1300
                }
            }
            _ => {
                if base_trick_score >= 100 {
                    // Game bonus
                    if self.declarer_vulnerable {
                        500
                    } else {
                        300
                    }
                } else {
                    // Part-score bonus
                    50
                }
            }
        }
    }

    /// Calculates the penalty for undertricks when the contract is not made.
    fn calculate_undertrick_penalty(&self, undertricks: i32) -> i32 {
        match self.doubling {
            Doubling::None => {
                let undertrick_value = if self.declarer_vulnerable { 100 } else { 50 };
                undertrick_value * undertricks
            }
            Doubling::Doubled => {
                if self.declarer_vulnerable {
                    200 * undertricks
                } else {
                    match undertricks {
                        1 => 100,
                        2 => 300,
                        3 => 500,
                        _ => 500 + 300 * (undertricks - 3),
                    }
                }
            }
            Doubling::Redoubled => {
                if self.declarer_vulnerable {
                    400 * undertricks
                } else {
                    match undertricks {
                        1 => 200,
                        2 => 600,
                        3 => 1000,
                        _ => 1000 + 600 * (undertricks - 3),
                    }
                }
            }
        }
    }
}
