use std::fmt::{Display, Formatter};
use crate::bridge::card::{Card, Suit, ALL_RANKS, ALL_SUITS};
use std::ops::{Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, Sub, SubAssign};
use std::sync::LazyLock;
use tinyvec::{array_vec, ArrayVec};

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct Hand {
    bitset: u64,
}

pub static FULL_HAND: LazyLock<Hand> = LazyLock::new(|| {
    let mut res = Hand::default();

    for s in ALL_SUITS {
        for r in ALL_RANKS {
            res += Card::new(s, r);
        }
    }

    res
});

impl Hand {
    pub fn new(cards: &[Card]) -> Self {
        let mut res = Self::default();
        for card in cards {
            res += *card;
        }
        res
    }

    pub fn len(&self) -> u32 {
        self.bitset.count_ones()
    }

    pub fn is_empty(&self) -> bool {
        self.bitset == 0
    }

    pub fn contains(&self, card: Card) -> bool {
        (self.bitset & (1 << (card.n as u64))) > 0
    }

    pub fn cards(&self) -> ArrayVec<[Card; 52]> {
        let mut res = array_vec![];

        for i in 0u8..52 {
            let card = Card { n: i };

            if self.contains(card) {
                res.push(card)
            }
        }

        res
    }

    pub fn cards_for_suit(&self, suit: Suit) -> ArrayVec<[Card; 13]> {
        let mut res = array_vec![];

        for i in 0u8..52 {
            let card = Card { n: i };

            if self.contains(card) && card.suit() == suit {
                res.push(card)
            }
        }

        res
    }
}

impl Display for Hand {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.cards()))
    }
}


impl AddAssign<Card> for Hand {
    fn add_assign(&mut self, rhs: Card) {
        self.bitset |= 1 << (rhs.n as u64)
    }
}

impl Add<Card> for Hand {
    type Output = Self;

    fn add(self, rhs: Card) -> Self::Output {
        let mut res = self.clone();
        res += rhs;

        res
    }
}

impl SubAssign<Card> for Hand {
    fn sub_assign(&mut self, rhs: Card) {
        self.bitset &= !(1 << (rhs.n as u64));
    }
}

impl Sub<Card> for Hand {
    type Output = Self;

    fn sub(self, rhs: Card) -> Self::Output {
        let mut res = self.clone();
        res -= rhs;

        res
    }
}

impl Sub<Hand> for Hand {
    type Output = Self;

    fn sub(self, rhs: Hand) -> Self::Output {
        let bitset = self.bitset & !rhs.bitset;

        Hand { bitset }
    }
}

impl Extend<Card> for Hand {
    fn extend<T: IntoIterator<Item = Card>>(&mut self, iter: T) {
        for c in iter {
            *self += c
        }
    }
}

impl BitAndAssign<Hand> for Hand {
    fn bitand_assign(&mut self, rhs: Hand) {
        self.bitset &= rhs.bitset
    }
}

impl BitAnd<Hand> for Hand {
    type Output = Self;

    fn bitand(self, rhs: Hand) -> Self::Output {
        let mut res = self.clone();
        res &= rhs;

        res
    }
}

impl BitOrAssign<Hand> for Hand {
    fn bitor_assign(&mut self, rhs: Hand) {
        self.bitset |= rhs.bitset
    }
}

impl BitOr<Hand> for Hand {
    type Output = Self;

    fn bitor(self, rhs: Hand) -> Self::Output {
        let mut res = self.clone();
        res &= rhs;

        res
    }
}

impl Default for Hand {
    fn default() -> Self {
        Self { bitset: 0 }
    }
}
