use crate::bridge::card::{Card, Rank, Suit, ALL_RANKS, ALL_SUITS};
use std::fmt::{Display, Formatter};
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

pub static ACES: LazyLock<Hand> = LazyLock::new(|| {
    let mut res = Hand::default();

    for s in ALL_SUITS {
        res += Card::new(s, Rank::Ace);
    }

    res
});

pub static TWOS: LazyLock<Hand> = LazyLock::new(|| {
    let mut res = Hand::default();

    for s in ALL_SUITS {
        res += Card::new(s, Rank::Two);
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

    #[inline]
    pub fn contains(&self, card: Card) -> bool {
        (self.bitset & (1 << (card.n as u64))) > 0
    }

    #[inline]
    pub fn run_for_cards(&self, mut f: impl FnMut(Card)) {
        for i in 0u8..52 {
            let card = Card { n: i };

            if self.contains(card) {
                f(card)
            }
        }
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

    pub fn reduce(&self, played_cards: Hand) -> Hand {
        debug_assert!(self.bitset & played_cards.bitset == 0);

        let mut res = self.bitset;

        // Removing a two doesn't allow any up-shifting
        let mask = (played_cards - *TWOS).bitset >> 1;

        while res & mask != 0 {
            res += res & mask;
        }

        debug_assert!(res.count_ones() == self.bitset.count_ones());

        Self { bitset: res }
    }

    pub fn unreduce(&self, played_cards: Hand) -> Hand {
        let mut res = self.bitset;

        // Just like how twos can't cause upshifting, we can't downshift a two
        let mask = (played_cards - *TWOS).bitset;
        while res & mask != 0 {
            // Put a 1 every place a card needs to be downshifted
            let downshift_candidates = res & mask;
            // Move those 1s down, so we can subtract to move the numbers
            let subtraction_mask = downshift_candidates >> 1;
            // Filter out any slot that is occupied
            let filter_subtraction = subtraction_mask & !res;
            res -= filter_subtraction;
        }

        debug_assert!(res.count_ones() == self.bitset.count_ones());

        Self { bitset: res }
    }

    pub fn first(&self) -> Card {
        for i in 0u8..52 {
            let card = Card { n: i };

            if self.contains(card) {
                return card;
            }
        }

        panic!()
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
        res |= rhs;

        res
    }
}

impl Default for Hand {
    fn default() -> Self {
        Self { bitset: 0 }
    }
}

#[cfg(test)]
mod test {
    use crate::bridge::card::{Card, Rank, Suit};
    use crate::bridge::hand::Hand;

    #[test]
    fn reduce_unreduce_rountrip() {
        let original = Hand::new(&[
            Card::new(Suit::Hearts, Rank::King),
            Card::new(Suit::Diamonds, Rank::Two),
            Card::new(Suit::Clubs, Rank::Three),
        ]);

        let reduce_by = Hand::new(&[
            Card::new(Suit::Hearts, Rank::Ace),
            Card::new(Suit::Diamonds, Rank::Three),
            Card::new(Suit::Diamonds, Rank::Four),
            Card::new(Suit::Diamonds, Rank::Five),
            Card::new(Suit::Clubs, Rank::Two),
        ]);

        let reduced = original.reduce(reduce_by);
        eprintln!("{:?}", reduced.cards());

        let restored = original.unreduce(reduce_by);
        eprintln!("{:?}", restored.cards());

        assert_eq!(original, restored);
    }
}
