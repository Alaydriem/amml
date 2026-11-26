use amml_abi::AmmlHand;

/// A hand of the local player.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Hand {
    Main,
    Off,
}

impl From<AmmlHand> for Hand {
    /// The off-hand slot is `Off`; every other slot reads as `Main`.
    fn from(hand: AmmlHand) -> Self {
        if hand == AmmlHand::OFF { Hand::Off } else { Hand::Main }
    }
}

impl From<Hand> for AmmlHand {
    fn from(hand: Hand) -> Self {
        match hand {
            Hand::Main => AmmlHand::MAIN,
            Hand::Off => AmmlHand::OFF,
        }
    }
}
