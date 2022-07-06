use std::fmt;

#[derive(PartialEq, Eq)]
pub enum State {
    Playing,
    WhiteIsMated,
    BlackIsMated,
    Stalemate,
    Repitition,
    FiftyMoveRule,
    InsufficientMaterial
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let string = match self {
            Self::Playing => "Still playing",
            Self::WhiteIsMated => "Black won by checkmate",
            Self::BlackIsMated => "White won by checkmate",
            Self::Stalemate => "Draw by stalemate",
            Self::Repitition => "Draw by repition",
            Self::FiftyMoveRule => "Draw due to fifty move rule",
            Self::InsufficientMaterial => "Draw due to insufficient material"
        };
        write!(f, "{}", string)
    }
}