use crate::bridge::card::Card;
use crate::bridge::contract::Contract;
use crate::bridge::hand::Hand;
use crate::bridge::player_info::{InfoForTurnPlayer, VisibleInfoForBridgePlayer};
use crate::bridge::seat::Seat;
use crate::cfr::game_model::{OracleGamestate, PlayerNumber};
use tinyvec::{array_vec, ArrayVec};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct BridgeGamestate {
    pub(crate) contract: Contract,

    pub(crate) declarer_tricks: u8,

    pub(crate) turn: Seat,
    pub(crate) cards_played: ArrayVec<[Card; 52]>,
    pub(crate) hand_of_cards_played: Hand,

    pub(crate) before_declarer_hand: Hand,
    pub(crate) declarer_hand: Hand,
    pub(crate) after_declarer_hand: Hand,
    pub(crate) dummy_hand: Hand,
}

impl OracleGamestate<VisibleInfoForBridgePlayer> for BridgeGamestate {
    fn info_for_turn_player(&self) -> VisibleInfoForBridgePlayer {
        if self.before_declarer_hand.is_empty()
            && self.declarer_hand.is_empty()
            && self.after_declarer_hand.is_empty()
            && self.dummy_hand.is_empty()
        {
            return VisibleInfoForBridgePlayer::Terminal {
                contract: self.contract.clone(),
                declarer_tricks: self.declarer_tricks,
            };
        }

        let mut mia_cards = self.declarer_hand
            | self.after_declarer_hand
            | self.before_declarer_hand
            | self.dummy_hand;
        let (player_hand, other_hand) = match self.turn {
            Seat::Declarer => (self.declarer_hand, self.dummy_hand),
            Seat::AfterDeclarer => (self.after_declarer_hand, self.dummy_hand),
            Seat::Dummy => (self.dummy_hand, self.declarer_hand),
            Seat::BeforeDeclarer => (self.before_declarer_hand, self.dummy_hand),
        };
        mia_cards = mia_cards - player_hand - other_hand;

        // FIXME: Reduce the cards in the current trick
        let mut current_trick = array_vec![];
        let trick_size = self.cards_played.len() % 4;
        let starting_idx = self.cards_played.len() - trick_size;
        for i in 1..=(self.cards_played.len() % 4) {
            current_trick.push(self.cards_played[starting_idx + i - 1]);
        }

        VisibleInfoForBridgePlayer::InPlay(InfoForTurnPlayer {
            player: self.turn,
            declarer_tricks: self.declarer_tricks,
            my_hand: player_hand.reduce(self.hand_of_cards_played),
            other_visible_hand: other_hand.reduce(self.hand_of_cards_played),
            cards_in_other_hands: mia_cards.reduce(self.hand_of_cards_played),
            current_trick,
        })
    }

    fn players_playing(&self) -> PlayerNumber {
        4
    }

    fn turn(&self) -> PlayerNumber {
        match self.turn {
            Seat::Declarer => 0,
            Seat::AfterDeclarer => 1,
            Seat::Dummy => 2,
            Seat::BeforeDeclarer => 3,
        }
    }

    fn advance(&self, m: &Card) -> Self {
        let unreduced_card = Hand::new(&[*m]).unreduce(self.hand_of_cards_played).first();

        let mut new_cards_played = self.cards_played.clone();
        new_cards_played.push(unreduced_card);

        let mut declarer_hand = self.declarer_hand;
        let mut after_declarer_hand = self.after_declarer_hand;
        let mut dummy_hand = self.dummy_hand;
        let mut before_declarer_hand = self.before_declarer_hand;

        let mut turn_player_hand = match self.turn {
            Seat::Declarer => &mut declarer_hand,
            Seat::AfterDeclarer => &mut after_declarer_hand,
            Seat::Dummy => &mut dummy_hand,
            Seat::BeforeDeclarer => &mut before_declarer_hand,
        };

        debug_assert!(turn_player_hand.contains(unreduced_card));
        *turn_player_hand -= unreduced_card;

        let mut result = Self {
            contract: self.contract,
            declarer_tricks: self.declarer_tricks,

            turn: self.turn.next(),
            cards_played: new_cards_played,
            hand_of_cards_played: self.hand_of_cards_played + unreduced_card,

            declarer_hand,
            after_declarer_hand,
            dummy_hand,
            before_declarer_hand,
        };

        if result.cards_played.len() % 4 == 0 {
            let mut winning_card = result.cards_played[result.cards_played.len() - 4];
            let mut winning_player = result.turn;

            let mut player = winning_player;
            for i in 1..=3 {
                player = player.next();
                let card = result.cards_played[result.cards_played.len() - 4 + i];

                if card.suit() == winning_card.suit() {
                    if card.rank().n() > winning_card.rank().n() {
                        winning_card = card;
                        winning_player = player;
                    }
                } else if Some(card.suit()) == self.contract.trump {
                    winning_card = card;
                    winning_player = player;
                }
            }

            if winning_player == Seat::Declarer || winning_player == Seat::Dummy {
                result.declarer_tricks += 1;
            }

            result.turn = winning_player;
        }

        result
    }
}
