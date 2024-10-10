use std::fmt::{Debug, Display, Formatter, Write};
use std::sync::LazyLock;

pub const SUITS: u8 = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Suit {
    Clubs,
    Diamonds,
    Hearts,
    Spades,
}

pub static ALL_SUITS: [Suit; 4] = [Suit::Clubs, Suit::Diamonds, Suit::Hearts, Suit::Spades];

impl Suit {
    fn n(&self) -> u8 {
        match self {
            Suit::Clubs => 0,
            Suit::Diamonds => 1,
            Suit::Hearts => 2,
            Suit::Spades => 3,
        }
    }
}

const RANKS: u8 = 13;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Rank {
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
    Ace,
}

pub static ALL_RANKS: [Rank; 13] = [
    Rank::Two,
    Rank::Three,
    Rank::Four,
    Rank::Five,
    Rank::Six,
    Rank::Seven,
    Rank::Eight,
    Rank::Nine,
    Rank::Ten,
    Rank::Jack,
    Rank::Queen,
    Rank::King,
    Rank::Ace,
];

impl Rank {
    pub fn n(&self) -> u8 {
        match self {
            Rank::Two => 0,
            Rank::Three => 1,
            Rank::Four => 2,
            Rank::Five => 3,
            Rank::Six => 4,
            Rank::Seven => 5,
            Rank::Eight => 6,
            Rank::Nine => 7,
            Rank::Ten => 8,
            Rank::Jack => 9,
            Rank::Queen => 10,
            Rank::King => 11,
            Rank::Ace => 12,
        }
    }
}

#[derive(Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Card {
    pub(crate) n: u8,
}

pub static ALL_CARDS: LazyLock<Vec<Card>> = LazyLock::new(|| {
    let mut res = Vec::new();

    for s in ALL_SUITS {
        for r in ALL_RANKS {
            res.push(Card::new(s, r));
        }
    }

    res
});

impl Card {
    pub fn new(suit: Suit, rank: Rank) -> Card {
        Self {
            n: suit.n() * RANKS + rank.n(),
        }
    }

    pub fn suit(&self) -> Suit {
        match self.n {
            0..13 => Suit::Clubs,
            13..26 => Suit::Diamonds,
            26..39 => Suit::Hearts,
            39..52 => Suit::Spades,
            _ => unreachable!(),
        }
    }

    pub fn rank(&self) -> Rank {
        match self.n % 13 {
            0 => Rank::Two,
            1 => Rank::Three,
            2 => Rank::Four,
            3 => Rank::Five,
            4 => Rank::Six,
            5 => Rank::Seven,
            6 => Rank::Eight,
            7 => Rank::Nine,
            8 => Rank::Ten,
            9 => Rank::Jack,
            10 => Rank::Queen,
            11 => Rank::King,
            12 => Rank::Ace,
            _ => unreachable!(),
        }
    }
}

impl Debug for Card {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)?;
        Ok(())
    }
}

impl Display for Card {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let rank = match self.rank() {
            Rank::Two => "2",
            Rank::Three => "3",
            Rank::Four => "4",
            Rank::Five => "5",
            Rank::Six => "6",
            Rank::Seven => "7",
            Rank::Eight => "8",
            Rank::Nine => "9",
            Rank::Ten => "T",
            Rank::Jack => "J",
            Rank::Queen => "Q",
            Rank::King => "K",
            Rank::Ace => "A",
        };

        let suit = match self.suit() {
            Suit::Clubs => "C",
            Suit::Diamonds => "D",
            Suit::Hearts => "H",
            Suit::Spades => "S"
        };

        f.write_fmt(format_args!("{}{}", rank, suit))?;

        Ok(())
    }
}
