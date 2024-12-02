use crate::bridge::card::{Card, ALL_CARDS};
use crate::bridge::contract::Contract;
use crate::bridge::gamestate::BridgeGamestate;
use crate::bridge::hand::{Hand, FULL_HAND};
use crate::bridge::seat::Seat;
use crate::cfr::game_model::{
    GamestateSampler, PlayerNumber, Probability, Utility, UtilityForAllPlayers, VisibleInfo,
};
use std::mem;
use tinyvec::ArrayVec;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum VisibleInfoForBridgePlayer {
    InPlay(InfoForTurnPlayer),
    Terminal {
        contract: Contract,
        declarer_tricks: u8,
    },
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct InfoForTurnPlayer {
    pub(crate) player: Seat,

    pub(crate) declarer_tricks: u8,

    pub(crate) my_hand: Hand,
    pub(crate) other_visible_hand: Hand,

    pub(crate) cards_in_other_hands: Hand,

    pub(crate) current_trick: ArrayVec<[Card; 4]>,
}

impl VisibleInfo for VisibleInfoForBridgePlayer {
    type Move = Card;
    type Gamestate = BridgeGamestate;

    fn players_playing(&self) -> PlayerNumber {
        4
    }

    fn turn(&self) -> PlayerNumber {
        match self {
            VisibleInfoForBridgePlayer::InPlay(s) => match s.player {
                Seat::Declarer => 0,
                Seat::AfterDeclarer => 1,
                Seat::Dummy => 2,
                Seat::BeforeDeclarer => 3,
            },
            VisibleInfoForBridgePlayer::Terminal { .. } => 0,
        }
    }

    fn run_for_moves(&self, mut f: impl FnMut(Self::Move)) -> Option<UtilityForAllPlayers> {
        match self {
            VisibleInfoForBridgePlayer::InPlay(s) => {
                let lead_suit = s.current_trick.first().map(|c| c.suit());

                let mut ran_card = false;
                s.my_hand.run_for_cards(|c| {
                    if Some(c.suit()) == lead_suit {
                        ran_card = true;
                        f(c);
                    }
                });

                // We have no on suit cards apparently
                if !ran_card {
                    s.my_hand.run_for_cards(f);
                }

                None
            }
            VisibleInfoForBridgePlayer::Terminal {
                contract,
                declarer_tricks,
            } => {
                let util = contract.declarer_points(*declarer_tricks as i32) as Utility;

                Some(UtilityForAllPlayers {
                    util: [util, -util, util, -util],
                })
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::bridge::card::{Card, Rank, Suit};
    use crate::bridge::contract::{Contract, Doubling};
    use crate::bridge::gamestate_sampler::GamestateSamplerForBridgePlayerInfo;
    use crate::bridge::hand::{Hand, FULL_HAND};
    use crate::bridge::player_info::InfoForTurnPlayer;
    use crate::bridge::seat::Seat;
    use crate::cfr::game_model::{GamestateSampler, OracleGamestate};
    use crate::cfr::strategy_generation::strategy_generator::StrategyGenerator;
    use bumpalo_herd::Herd;
    use tinyvec::array_vec;

    #[test]
    fn bridge_master_one_modified() {
        let herd = Herd::new();
        let strategy_generator = StrategyGenerator::new(&herd);

        let contract = Contract {
            trump: Some(Suit::Spades),
            n: 6,
            doubling: Doubling::None,
            defender_vulnerable: false,
            declarer_vulnerable: false,
        };
        let played_cards = array_vec!(
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
            Card::new(Suit::Spades, Rank::Five)
        );
        let my_hand_unreduced = Hand::new(&[
            Card::new(Suit::Spades, Rank::Queen),
            Card::new(Suit::Spades, Rank::Jack),
            Card::new(Suit::Spades, Rank::Ten),
            Card::new(Suit::Hearts, Rank::Two),
            Card::new(Suit::Clubs, Rank::Ace),
            Card::new(Suit::Diamonds, Rank::Four),
            Card::new(Suit::Diamonds, Rank::Three),
            Card::new(Suit::Diamonds, Rank::Two),
        ]);
        let other_visible_hand_unreduced = Hand::new(&[
            Card::new(Suit::Spades, Rank::Nine),
            Card::new(Suit::Hearts, Rank::Five),
            Card::new(Suit::Hearts, Rank::Four),
            Card::new(Suit::Clubs, Rank::King),
            Card::new(Suit::Clubs, Rank::Five),
            Card::new(Suit::Clubs, Rank::Four),
            Card::new(Suit::Clubs, Rank::Three),
            Card::new(Suit::Clubs, Rank::Two),
        ]);
        let played_cards_hand = Hand::new(&played_cards);
        let info_for_turn_player = InfoForTurnPlayer {
            player: Seat::Declarer,
            declarer_tricks: 5,
            my_hand:my_hand_unreduced.reduce(played_cards_hand),
            other_visible_hand: other_visible_hand_unreduced.reduce(played_cards_hand),
            cards_in_other_hands: (*FULL_HAND - played_cards_hand - my_hand_unreduced - other_visible_hand_unreduced).reduce(played_cards_hand),
            current_trick: Default::default(),
        };
        let mut sampler =
            GamestateSamplerForBridgePlayerInfo::new(info_for_turn_player, contract, played_cards);
        strategy_generator.refine_strategy(sampler.clone(), 128);

        let mut board = sampler.sample().0;
        println!("{:?}", board);
        println!(
            "{:?}",
            strategy_generator
                .strategy_for_info(board.info_for_turn_player().clone())
                .move_probabilities(),
        );
        println!();

        while let Some(m) = strategy_generator
            .strategy_for_info(board.info_for_turn_player().clone())
            .pick_move()
        {
            board = board.advance(&m);
            println!("board {:?}", board);
            // println!("lead {:?}", board.lead());
            // println!("turn player hand {}", board.turn_player_hand_mut());
            // println!("moves {:?}", board.info_for_turn_player().moves());
            println!();

            // println!("Refining real quick!");
            // strategy_generator.refine_strategy(BridgeGamestateSampler::new(&board.info_for_turn_player()), 128);

            println!(
                "{:?}",
                strategy_generator
                    .strategy_for_info(board.info_for_turn_player().clone())
                    .move_probabilities(),
            );
        }
    }

    #[test]
    fn bridge_master_expert_a1() {
        let herd = Herd::new();
        let strategy_generator = StrategyGenerator::new(&herd);

        let contract = Contract {
            trump: Some(Suit::Spades),
            n: 6,
            doubling: Doubling::None,
            defender_vulnerable: false,
            declarer_vulnerable: false,
        };
        let played_cards = array_vec!(
            [Card; 52] => Card::new(Suit::Clubs, Rank::King)
        );
        let my_hand_unreduced = Hand::new(&[
            Card::new(Suit::Spades, Rank::King),
            Card::new(Suit::Spades, Rank::Four),
            Card::new(Suit::Spades, Rank::Three),
            Card::new(Suit::Spades, Rank::Two),

            Card::new(Suit::Hearts, Rank::Ace),
            Card::new(Suit::Hearts, Rank::Four),
            Card::new(Suit::Hearts, Rank::Three),
            Card::new(Suit::Hearts, Rank::Two),

            Card::new(Suit::Clubs, Rank::Three),
            Card::new(Suit::Clubs, Rank::Two),

            Card::new(Suit::Diamonds, Rank::King),
            Card::new(Suit::Diamonds, Rank::Four),
            Card::new(Suit::Diamonds, Rank::Three),
        ]);
        let other_visible_hand_unreduced = Hand::new(&[
            Card::new(Suit::Spades, Rank::Ace),
            Card::new(Suit::Spades, Rank::Queen),
            Card::new(Suit::Spades, Rank::Eight),
            Card::new(Suit::Spades, Rank::Seven),
            Card::new(Suit::Spades, Rank::Six),
            Card::new(Suit::Spades, Rank::Five),

            Card::new(Suit::Hearts, Rank::King),
            Card::new(Suit::Hearts, Rank::Nine),
            Card::new(Suit::Hearts, Rank::Eight),
            Card::new(Suit::Hearts, Rank::Seven),

            Card::new(Suit::Clubs, Rank::Ace),

            Card::new(Suit::Diamonds, Rank::Ace),
            Card::new(Suit::Diamonds, Rank::Two),
        ]);
        let played_cards_hand = Hand::new(&played_cards);
        let info_for_turn_player = InfoForTurnPlayer {
            player: Seat::Dummy,
            declarer_tricks: 0,
            my_hand:my_hand_unreduced.reduce(played_cards_hand),
            other_visible_hand: other_visible_hand_unreduced.reduce(played_cards_hand),
            cards_in_other_hands: (*FULL_HAND - played_cards_hand - my_hand_unreduced - other_visible_hand_unreduced).reduce(played_cards_hand),
            current_trick: array_vec!([Card; 4] => Card::new(Suit::Clubs, Rank::King)),
        };
        let mut sampler =
            GamestateSamplerForBridgePlayerInfo::new(info_for_turn_player, contract, played_cards);
        strategy_generator.refine_strategy(sampler.clone(), 128);

        let mut board = sampler.sample().0;
        println!("{:?}", board);
        println!(
            "{:?}",
            strategy_generator
                .strategy_for_info(board.info_for_turn_player().clone())
                .move_probabilities(),
        );
        println!();

        while let Some(m) = strategy_generator
            .strategy_for_info(board.info_for_turn_player().clone())
            .pick_move()
        {
            board = board.advance(&m);
            println!("board {:?}", board);
            // println!("lead {:?}", board.lead());
            // println!("turn player hand {}", board.turn_player_hand_mut());
            // println!("moves {:?}", board.info_for_turn_player().moves());
            println!();

            // println!("Refining real quick!");
            // strategy_generator.refine_strategy(BridgeGamestateSampler::new(&board.info_for_turn_player()), 128);

            println!(
                "{:?}",
                strategy_generator
                    .strategy_for_info(board.info_for_turn_player().clone())
                    .move_probabilities(),
            );
        }
    }
}
