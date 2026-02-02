//! Partizan games

pub mod canonical_form;
pub mod games;
pub mod partizan_game;
pub mod thermograph;
pub mod trajectory;
pub mod transposition_table;

/// Player
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Player {
    /// Right (red) player
    Right,

    /// Left (bLue) player
    Left,
}

impl Player {
    /// Opposite player
    #[inline(always)]
    #[must_use]
    pub const fn opposite(self) -> Player {
        match self {
            Player::Left => Player::Right,
            Player::Right => Player::Left,
        }
    }

    /// Run a predicate for both players
    #[inline(always)]
    pub fn forall<P>(mut predicate: P) -> bool
    where
        P: FnMut(Player) -> bool,
    {
        predicate(Player::Left) && predicate(Player::Right)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::{Arbitrary, Gen};
    use std::cmp::Ordering;

    impl Arbitrary for Player {
        fn arbitrary(g: &mut Gen) -> Self {
            if Arbitrary::arbitrary(g) {
                Player::Left
            } else {
                Player::Right
            }
        }
    }

    #[test]
    fn player_order() {
        // Just so no one changes order of enum variants
        assert_eq!(Ord::cmp(&Player::Left, &Player::Right), Ordering::Greater);
        assert_eq!(Ord::cmp(&Player::Right, &Player::Left), Ordering::Less);
        assert_eq!(Ord::cmp(&Player::Left, &Player::Left), Ordering::Equal);
        assert_eq!(Ord::cmp(&Player::Right, &Player::Right), Ordering::Equal);
    }
}
