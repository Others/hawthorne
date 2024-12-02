use crate::bridge::card::Card;
use crate::bridge::contract::Contract;
use crate::bridge::gamestate::BridgeGamestate;
use crate::bridge::hand::{Hand, FULL_HAND};
use crate::bridge::player_info::{InfoForTurnPlayer, VisibleInfoForBridgePlayer};
use crate::bridge::seat::Seat;
use crate::cfr::game_model::{GamestateSampler, Probability, VisibleInfo};
use std::mem;
use tinyvec::ArrayVec;

#[derive(Clone)]
pub struct GamestateSamplerForBridgePlayerInfo {
    v: InfoForTurnPlayer,
    played_cards_vec: ArrayVec<[Card; 52]>,
    played_cards: Hand,
    contract: Contract,
}

impl GamestateSamplerForBridgePlayerInfo {
    pub fn new(
        v: InfoForTurnPlayer,
        contract: Contract,
        played_cards_vec: ArrayVec<[Card; 52]>,
    ) -> Self {
        let played_cards = Hand::new(&played_cards_vec);
        assert_eq!(
            v.my_hand.len()
                + v.other_visible_hand.len()
                + v.cards_in_other_hands.len()
                + played_cards.len(),
            52
        );
        assert_eq!(
            v.my_hand.unreduce(played_cards)
                | v.other_visible_hand.unreduce(played_cards)
                | v.cards_in_other_hands.unreduce(played_cards)
                | played_cards,
            *FULL_HAND
        );

        Self {
            v,
            played_cards,
            played_cards_vec,
            contract,
        }
    }
}

impl GamestateSampler for GamestateSamplerForBridgePlayerInfo {
    type Info = VisibleInfoForBridgePlayer;

    fn sample(&mut self) -> (BridgeGamestate, Probability) {
        let my_hand = self.v.my_hand.unreduce(self.played_cards);
        let other_hand = self.v.other_visible_hand.unreduce(self.played_cards);
        let mut mystery_hand_one = Hand::default();
        let mut mystery_hand_two = Hand::default();

        let pick_from = self
            .v
            .cards_in_other_hands
            .unreduce(self.played_cards)
            .cards()
            .to_vec();
        fill_hands(pick_from, &mut mystery_hand_one, &mut mystery_hand_two);

        let (declarer, after_declarer, dummy, before_declarer) = match self.v.player {
            // Mystery hand one always has the same number or more cards than hand two
            // Therefore listing it right after the turn player ensures the right number of cards always
            Seat::Declarer => (my_hand, mystery_hand_one, other_hand, mystery_hand_two),
            Seat::AfterDeclarer => (mystery_hand_two, my_hand, other_hand, mystery_hand_one),
            Seat::Dummy => (other_hand, mystery_hand_two, my_hand, mystery_hand_one),
            Seat::BeforeDeclarer => (mystery_hand_one, mystery_hand_two, other_hand, my_hand),
        };

        let gamestate = BridgeGamestate {
            contract: self.contract,
            declarer_tricks: self.v.declarer_tricks,
            turn: self.v.player,
            dummy_hand: dummy,
            declarer_hand: declarer,
            after_declarer_hand: after_declarer,
            before_declarer_hand: before_declarer,

            cards_played: self.played_cards_vec,
            hand_of_cards_played: self.played_cards,
        };

        // FIXME: Probability is wrong
        (gamestate, 0.001)
    }
}

fn fill_hands(mut cards_to_pick_from: Vec<Card>, first: &mut Hand, second: &mut Hand) {
    let mut current_hand = first;
    let mut other_hand = second;

    while !cards_to_pick_from.is_empty() {
        let i = fastrand::usize(0..cards_to_pick_from.len());
        let card = cards_to_pick_from.swap_remove(i);
        *current_hand += card;

        mem::swap(&mut current_hand, &mut other_hand);
    }
}
