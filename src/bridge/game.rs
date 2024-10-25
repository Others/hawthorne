use crate::bridge::card::{Card, ALL_CARDS};
use crate::bridge::contract::Contract;
use crate::bridge::hand::Hand;
use crate::bridge::seat::Seat;
use crate::cfr::game_model::{
    GamestateSampler, OracleGamestate, PlayerNumber, Probability, Utility,
    UtilityForAllPlayers, VisibleInfo,
};
use std::hash::Hash;
use tinyvec::ArrayVec;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct BridgeGame {
    contract: Contract,

    declarer_tricks: u8,
    defender_tricks: u8,
    played_cards: usize,

    turn_player: Seat,
    cards_played: ArrayVec<[Card; 52]>,
    hand_of_cards_played: Hand,

    before_hand: Hand,
    declarer_hand: Hand,
    after_hand: Hand,
    dummy_hand: Hand,
}

impl BridgeGame {
    fn declarer_info(&self) -> DeclarerInfoSet {
        DeclarerInfoSet {
            universal_information: self.generate_universal_information(),
            declarer_hand: self.declarer_hand,
        }
    }

    fn dummy_info(&self) -> DummyInfoSet {
        DummyInfoSet {
            universal_information: self.generate_universal_information(),
            declarer_hand: self.declarer_hand,
        }
    }

    fn before_info(&self) -> BeforeDeclarerInfoSet {
        BeforeDeclarerInfoSet {
            universal_information: self.generate_universal_information(),
            before_hand: self.before_hand,
        }
    }

    fn after_info(&self) -> AfterDeclarerInfoSet {
        AfterDeclarerInfoSet {
            universal_information: self.generate_universal_information(),
            after_hand: self.after_hand,
        }
    }

    fn lead(&self) -> Option<(Seat, Card)> {
        let cards_played = &self.cards_played;
        let len = cards_played.len();

        if len % 4 == 0 {
            return None;
        }

        let mut n = 4 - len % 4;
        let mut p = self.turn_player;
        while n > 0 {
            p = p.prev();
            n -= 1;
        }

        Some((p, cards_played[len / 4]))
    }

    fn generate_universal_information(&self) -> UniversalInformation {
        UniversalInformation {
            contract: self.contract,
            declarer_tricks: self.declarer_tricks,
            defender_tricks: self.defender_tricks,
            cards_played: self.hand_of_cards_played,
            dummy_hand: self.dummy_hand,
            lead: self.lead(),
        }
    }

    fn turn_player_hand_mut(&mut self) -> &mut Hand {
        match self.turn_player {
            Seat::Declarer => &mut self.declarer_hand,
            Seat::AfterDeclarer => &mut self.after_hand,
            Seat::Dummy => &mut self.dummy_hand,
            Seat::BeforeDeclarer => &mut self.before_hand,
        }
    }

    fn handle_end_of_trick(&mut self) {
        let leader = self.turn_player.next();
        let lead_card = self.cards_played[self.cards_played.len() - 4];

        let mut winner = leader;
        let mut winner_card = lead_card;

        let mut current_player = leader;
        for i in (1..=3).rev() {
            current_player = current_player.prev();

            let card = self.cards_played[self.cards_played.len() - i];
            if self.contract.card_defeats_card(winner_card, card) {
                winner = current_player;
                winner_card = card;
            }
        }

        self.turn_player = winner;
        match self.turn_player {
            Seat::Declarer => {
                self.declarer_tricks += 1;
            }
            Seat::AfterDeclarer => {
                self.defender_tricks += 1;
            }
            Seat::Dummy => {
                self.declarer_tricks += 1;
            }
            Seat::BeforeDeclarer => {
                self.defender_tricks += 1;
            }
        }
    }

    fn moves(&self) -> Vec<Card> {
        let mut res = Vec::new();
        self.info_for_turn_player().run_for_moves(|m| {
            res.push(m);
        });

        res
    }
}

impl OracleGamestate<BridgeInfoSet> for BridgeGame {
    fn info_for_turn_player(&self) -> BridgeInfoSet {
        match self.turn_player {
            Seat::Declarer => BridgeInfoSet::Declarer(self.declarer_info()),
            Seat::AfterDeclarer => BridgeInfoSet::AfterDeclarer(self.after_info()),
            Seat::Dummy => BridgeInfoSet::Dummy(self.dummy_info()),
            Seat::BeforeDeclarer => BridgeInfoSet::BeforeDeclarer(self.before_info()),
        }
    }

    fn turn(&self) -> PlayerNumber {
        self.info_for_turn_player().turn()
    }

    fn advance(&self, m: &Card) -> Self {
        let mut new = self.clone();

        new.played_cards += 1;

        *new.turn_player_hand_mut() -= *m;
        new.cards_played.push(*m);
        new.hand_of_cards_played += *m;

        if new.cards_played.len() % 4 == 0 {
            new.handle_end_of_trick()
        } else {
            new.turn_player = self.turn_player.next();
        }

        new
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct UniversalInformation {
    contract: Contract,

    declarer_tricks: u8,
    defender_tricks: u8,

    // FIXME: Also include any witnessed voids
    // FIXME: Also include the cards played in this hand perhaps?
    lead: Option<(Seat, Card)>,

    dummy_hand: Hand,

    cards_played: Hand,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct DeclarerInfoSet {
    universal_information: UniversalInformation,
    declarer_hand: Hand,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct DummyInfoSet {
    universal_information: UniversalInformation,
    declarer_hand: Hand,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct BeforeDeclarerInfoSet {
    universal_information: UniversalInformation,
    before_hand: Hand,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct AfterDeclarerInfoSet {
    universal_information: UniversalInformation,
    after_hand: Hand,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum BridgeInfoSet {
    Declarer(DeclarerInfoSet),
    AfterDeclarer(AfterDeclarerInfoSet),
    Dummy(DummyInfoSet),
    BeforeDeclarer(BeforeDeclarerInfoSet),
}

impl BridgeInfoSet {
    pub fn seat(&self) -> Seat {
        match self {
            BridgeInfoSet::Declarer(_) => Seat::Declarer,
            BridgeInfoSet::AfterDeclarer(_) => Seat::AfterDeclarer,
            BridgeInfoSet::Dummy(_) => Seat::Dummy,
            BridgeInfoSet::BeforeDeclarer(_) => Seat::BeforeDeclarer,
        }
    }

    pub fn moves(&self) -> Vec<Card> {
        let mut res = Vec::new();
        self.run_for_moves(|m| {
            res.push(m);
        });

        res
    }

    fn turn_player_hand(&self) -> &Hand {
        match self {
            BridgeInfoSet::Declarer(i) => &i.declarer_hand,
            BridgeInfoSet::AfterDeclarer(i) => &i.after_hand,
            BridgeInfoSet::Dummy(i) => &i.universal_information.dummy_hand,
            BridgeInfoSet::BeforeDeclarer(i) => &i.before_hand,
        }
    }

    // fn other_visible_hand(&self) -> &Hand {
    //     match self {
    //         BridgeInfoSet::Declarer(i) => &i.universal_information.dummy_hand,
    //         BridgeInfoSet::AfterDeclarer(i) => &i.universal_information.dummy_hand,
    //         BridgeInfoSet::Dummy(i) => &i.declarer_hand,
    //         BridgeInfoSet::BeforeDeclarer(i) => &i.universal_information.dummy_hand,
    //     }
    // }

    fn universal_information(&self) -> &UniversalInformation {
        match self {
            BridgeInfoSet::Declarer(i) => &i.universal_information,
            BridgeInfoSet::AfterDeclarer(i) => &i.universal_information,
            BridgeInfoSet::Dummy(i) => &i.universal_information,
            BridgeInfoSet::BeforeDeclarer(i) => &i.universal_information,
        }
    }

    // fn materialize_gamestate(&self, other_a: Hand, other_b: Hand) -> BridgeGame {
    //     assert_eq!(self.universal_information().cards_played.len() % 4, 0);
    //
    //     let universal_info = self.universal_information();
    //     let dummy = universal_info.dummy_hand;
    //
    //     let (before, declarer, after, turn) = match self {
    //         BridgeInfoSet::Declarer(i) => (other_a, i.declarer_hand, other_b, Seat::Declarer),
    //         BridgeInfoSet::AfterDeclarer(i) => {
    //             (other_a, other_b, i.after_hand, Seat::AfterDeclarer)
    //         }
    //         BridgeInfoSet::Dummy(i) => (other_a, i.declarer_hand, other_b, Seat::Dummy),
    //         BridgeInfoSet::BeforeDeclarer(i) => {
    //             (i.before_hand, other_a, other_b, Seat::BeforeDeclarer)
    //         }
    //     };
    //
    //     BridgeGame {
    //         before_hand: before,
    //         declarer_hand: declarer,
    //         after_hand: after,
    //         dummy_hand: dummy,
    //         hand_of_cards_played: universal_info.cards_played,
    //         cards_played: ArrayVec::from(universal_info.cards_played.cards()),
    //         contract: universal_info.contract,
    //         declarer_tricks: universal_info.declarer_tricks,
    //         defender_tricks: universal_info.defender_tricks,
    //         played_cards: self.universal_information().cards_played.len() as usize,
    //         turn_player: turn,
    //     }
    // }
}

impl VisibleInfo for BridgeInfoSet {
    type Move = Card;
    type Gamestate = BridgeGame;

    fn max_players(&self) -> PlayerNumber {
        4
    }

    fn turn(&self) -> PlayerNumber {
        match self {
            BridgeInfoSet::Declarer(_) => 0,
            BridgeInfoSet::AfterDeclarer(_) => 1,
            BridgeInfoSet::Dummy(_) => 2,
            BridgeInfoSet::BeforeDeclarer(_) => 3,
        }
    }

    fn run_for_moves(&self, mut f: impl FnMut(Self::Move)) -> Option<UtilityForAllPlayers> {
        let universal_info = self.universal_information();
        let hand = self.turn_player_hand();

        // If our hand is empty, we need to return a terminal
        if hand.is_empty() {
            let tricks_for_declarer = universal_info.declarer_tricks;
            let value = universal_info
                .contract
                .declarer_points(tricks_for_declarer as i32) as Utility;

            return Some(UtilityForAllPlayers {
                util: [value, -value, value, -value],
            });
        }

        // If there is no lead card, we need to allow all moves
        let lead_suit = match self.universal_information().lead {
            Some((_, lead_card)) => lead_card.suit(),
            None => {
                hand.run_for_cards(f);
                return None;
            }
        };

        let mut found_matching = false;
        hand.run_for_cards(|card| {
            if card.suit() == lead_suit {
                found_matching = true;
                f(card)
            }
        });

        if !found_matching {
            hand.run_for_cards(f);
        }

        None
    }

    fn get_all_possible_gamestates(&self) -> impl Iterator<Item = (Self::Gamestate, Probability)> {
        // FIXME: Obviously dumb
        vec![].into_iter()
        //
        // assert_eq!(self.universal_information().cards_played.len() % 4, 0);
        //
        // let my_hand = *self.turn_player_hand();
        // let other_hand = *self.other_visible_hand();
        // assert_eq!(my_hand.len(), other_hand.len());
        //
        // let missing_cards = (*FULL_HAND) - my_hand - other_hand;
        //
        // let mut res = Vec::new();
        //
        // // for subset in missing_cards
        // //     .cards()
        // //     .iter()
        // //     .permutations(my_hand.len() as usize)
        // //     .take(10_000)
        // // {
        // //     let hand_a = Hand::new(&subset);
        // //     let hand_b = missing_cards - hand_a;
        // //
        // //     res.push((self.materialize_gamestate(hand_a, hand_b), 1.0));
        // // }
        //
        // let n = res.len() as Probability;
        // for (_, prob) in &mut res {
        //     *prob = 1.0 / n;
        // }
        //
        // res.into_iter()
    }

    fn gamestate_sampler(&self) -> impl GamestateSampler<Info = Self> {
        BridgeGamestateSampler::new(self)
    }
}

pub struct BridgeGamestateSampler {
    universal_information: UniversalInformation,

    turn_player: Seat,

    dummy_hand: Hand,
    dummy_missing: u32,

    before_declarer_hand: Hand,
    before_declarer_missing: u32,

    declarer_hand: Hand,
    declarer_missing: u32,

    after_declarer_hand: Hand,
    after_declarer_missing: u32,

    missing_cards: ArrayVec<[Card; 52]>,
    combos: u32,
}

impl BridgeGamestateSampler {
    fn new(info: &BridgeInfoSet) -> BridgeGamestateSampler {
        let universal_information = info.universal_information().clone();
        let turn_player = info.seat();

        assert_eq!(universal_information.cards_played.len() % 4, 0);
        let dummy_hand = universal_information.dummy_hand;
        let dummy_len = dummy_hand.len();
        let dummy_hand_missing = 0;

        let (before_declarer_hand, before_declarer_missing) = match info {
            BridgeInfoSet::BeforeDeclarer(i) => (i.before_hand, 0),
            _ => (Hand::default(), dummy_len),
        };

        let (after_declarer_hand, after_declarer_missing) = match info {
            BridgeInfoSet::AfterDeclarer(i) => (i.after_hand, 0),
            _ => (Hand::default(), dummy_len),
        };

        let (declarer_hand, declarer_hand_missing) = match info {
            BridgeInfoSet::Dummy(i) => (i.declarer_hand, 0),
            BridgeInfoSet::Declarer(i) => (i.declarer_hand, 0),
            _ => (Hand::default(), dummy_len),
        };

        let missing_cards = Hand::new(ALL_CARDS.as_slice())
            - dummy_hand
            - before_declarer_hand
            - declarer_hand
            - after_declarer_hand
            - universal_information.cards_played;

        let combos = Self::count_combinations_of_picks(
            missing_cards.len(),
            dummy_hand_missing,
            after_declarer_missing,
            declarer_hand_missing,
        );

        Self {
            universal_information,
            turn_player,
            dummy_hand,
            dummy_missing: dummy_hand_missing,
            before_declarer_hand,
            before_declarer_missing,
            declarer_hand,
            declarer_missing: declarer_hand_missing,
            after_declarer_hand,
            after_declarer_missing,
            missing_cards: missing_cards.cards(),
            combos,
        }
    }

    fn count_combinations_of_picks(n_to_pick: u32, a: u32, b: u32, c: u32) -> u32 {
        let mut acc = 1;

        acc *= Self::count_combinations(n_to_pick, a);
        acc *= Self::count_combinations(n_to_pick - a, b);
        acc *= Self::count_combinations(n_to_pick - a - b, c);

        acc
    }

    // Cribbed from: https://stackoverflow.com/questions/65561566
    fn count_combinations(n: u32, r: u32) -> u32 {
        if r > n {
            0
        } else {
            (1..=r.min(n - r)).fold(1, |acc, val| acc * (n - val + 1) / val)
        }
    }

    fn fill_hand(cards: &mut ArrayVec<[Card; 52]>, hand: &mut Hand, missing_n: u32) {
        let mut togo = missing_n;
        while togo > 0 {
            let i = fastrand::usize(..cards.len());
            let card = cards.swap_remove(i);
            *hand += card;
            togo -= 1;
        }
    }
}

impl GamestateSampler for BridgeGamestateSampler {
    type Info = BridgeInfoSet;

    fn sample(&mut self) -> (<Self::Info as VisibleInfo>::Gamestate, Probability) {
        let prob = 1.0 / (self.combos as Probability);
        let mut cards_to_pick = self.missing_cards;

        let mut dummy_hand = self.dummy_hand;
        Self::fill_hand(&mut cards_to_pick, &mut dummy_hand, self.dummy_missing);

        let mut before_declarer_hand = self.before_declarer_hand;
        Self::fill_hand(
            &mut cards_to_pick,
            &mut before_declarer_hand,
            self.before_declarer_missing,
        );

        let mut declarer_hand = self.declarer_hand;
        Self::fill_hand(
            &mut cards_to_pick,
            &mut declarer_hand,
            self.declarer_missing,
        );

        let mut after_declarer_hand = self.after_declarer_hand;
        // TODO: Can we just += here?
        Self::fill_hand(
            &mut cards_to_pick,
            &mut after_declarer_hand,
            self.after_declarer_missing,
        );

        let gamestate = BridgeGame {
            contract: self.universal_information.contract,
            declarer_tricks: self.universal_information.declarer_tricks,
            defender_tricks: self.universal_information.defender_tricks,
            played_cards: self.universal_information.cards_played.len() as usize,
            turn_player: self.turn_player,
            cards_played: self.universal_information.cards_played.cards(),
            hand_of_cards_played: self.universal_information.cards_played,
            before_hand: before_declarer_hand,
            declarer_hand,
            after_hand: after_declarer_hand,
            dummy_hand,
        };

        (gamestate, prob)
    }
}

#[cfg(test)]
mod test {
    use crate::bridge::card::{Card, Rank, Suit};
    use crate::bridge::contract::{Contract, Doubling};
    use crate::bridge::game::{BridgeInfoSet, DeclarerInfoSet, UniversalInformation};
    use crate::bridge::hand::Hand;
    use crate::cfr::game_model::{GamestateSampler, OracleGamestate, VisibleInfo};
    use crate::cfr::strategy_generation::generate_strategy_2;

    #[test]
    fn bridge_master_one() {
        let infoset = BridgeInfoSet::Declarer(DeclarerInfoSet {
            universal_information: UniversalInformation {
                contract: Contract {
                    trump: Some(Suit::Spades),
                    n: 6,
                    doubling: Doubling::None,
                    defender_vulnerable: false,
                    declarer_vulnerable: false,
                },
                defender_tricks: 0,
                declarer_tricks: 6,
                cards_played: Hand::new(&[
                    Card::new(Suit::Hearts, Rank::King),
                    Card::new(Suit::Hearts, Rank::Three),
                    Card::new(Suit::Hearts, Rank::Six),
                    Card::new(Suit::Hearts, Rank::Ace),

                    Card::new(Suit::Diamonds, Rank::King),
                    Card::new(Suit::Diamonds, Rank::Six),
                    Card::new(Suit::Diamonds, Rank::Seven),
                    Card::new(Suit::Diamonds, Rank::Eight),

                    Card::new(Suit::Diamonds, Rank::Ace),
                    Card::new(Suit::Diamonds, Rank::Five),
                    Card::new(Suit::Diamonds, Rank::Jack),
                    Card::new(Suit::Diamonds, Rank::Ten),

                    Card::new(Suit::Spades, Rank::King),
                    Card::new(Suit::Spades, Rank::Eight),
                    Card::new(Suit::Spades, Rank::Two),
                    Card::new(Suit::Spades, Rank::Three),

                    Card::new(Suit::Spades, Rank::Seven),
                    Card::new(Suit::Spades, Rank::Ace),
                    Card::new(Suit::Spades, Rank::Four),
                    Card::new(Suit::Spades, Rank::Five),

                ]),
                lead: None,
                dummy_hand: Hand::new(&[
                    Card::new(Suit::Spades, Rank::Nine),
                    Card::new(Suit::Hearts, Rank::Five),
                    Card::new(Suit::Hearts, Rank::Four),
                    Card::new(Suit::Clubs, Rank::King),
                    Card::new(Suit::Clubs, Rank::Five),
                    Card::new(Suit::Clubs, Rank::Four),
                    Card::new(Suit::Clubs, Rank::Three),
                    Card::new(Suit::Clubs, Rank::Two),
                ]),
            },
            declarer_hand: Hand::new(&[
                Card::new(Suit::Spades, Rank::Queen),
                Card::new(Suit::Spades, Rank::Jack),
                Card::new(Suit::Spades, Rank::Ten),
                Card::new(Suit::Hearts, Rank::Two),
                Card::new(Suit::Clubs, Rank::Ace),
                Card::new(Suit::Diamonds, Rank::Four),
                Card::new(Suit::Diamonds, Rank::Three),
                Card::new(Suit::Diamonds, Rank::Two),
            ]),
        });

        let strategy = generate_strategy_2(infoset.clone(), 1000);
        // let strategy = Strategy::default();

        let mut board = infoset.gamestate_sampler().sample().0;
        println!("{:?}", strategy.get_move_probabilities(infoset.clone()));
        println!("{:?}", board);
        println!(
            "{:?}",
            strategy.get_move_probabilities(board.info_for_turn_player().clone())
        );
        println!();

        while let Some(m) = strategy.pick_move(board.info_for_turn_player().clone()) {
            board = board.advance(&m);
            println!("board {:?}", board);
            println!("lead {:?}", board.lead());
            println!("turn player hand {}", board.turn_player_hand_mut());
            println!("moves {:?}", board.info_for_turn_player().moves());
            println!();

            println!(
                "{:?}",
                strategy.get_move_probabilities(board.info_for_turn_player().clone())
            );
        }
    }
}
