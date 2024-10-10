#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Seat {
    Declarer,
    AfterDeclarer,
    Dummy,
    BeforeDeclarer,
}

impl Default for Seat {
    fn default() -> Self {
        Self::Declarer
    }
}

impl Seat {
    pub fn next(&self) -> Self {
        match self {
            Seat::Declarer => Self::AfterDeclarer,
            Seat::AfterDeclarer => Self::Dummy,
            Seat::Dummy => Self::BeforeDeclarer,
            Seat::BeforeDeclarer => Self::Declarer,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            Seat::Declarer => Self::BeforeDeclarer,
            Seat::AfterDeclarer => Self::Declarer,
            Seat::Dummy => Self::AfterDeclarer,
            Seat::BeforeDeclarer => Self::Dummy,
        }
    }
}
