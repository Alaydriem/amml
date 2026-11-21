use amml_abi::AmmlHand;

/// A hand of the local player.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Hand {
    Main,
    Off,
}

impl From<Hand> for AmmlHand {
    fn from(hand: Hand) -> Self {
        match hand {
            Hand::Main => AmmlHand::MAIN,
            Hand::Off => AmmlHand::OFF,
        }
    }
}
