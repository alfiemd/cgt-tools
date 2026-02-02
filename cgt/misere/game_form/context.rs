#![allow(missing_docs)]

use crate::{
    misere::game_form::Outcome,
    parsing::{Parser, lexeme, try_option},
    short::partizan::Player,
};
use std::{cmp::Ordering, convert::Infallible};

#[cfg(any(test, feature = "quickcheck"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ArbitraryError<Construction, Integer> {
    Construction(Construction),
    Integer(Integer),
}

#[cfg(any(test, feature = "quickcheck"))]
impl<G, Construction, Integer> ConstructionError<G> for ArbitraryError<Construction, Integer>
where
    Construction: ConstructionError<G>,
    Integer: ConstructionError<G>,
{
    fn recover(self) -> G {
        match self {
            ArbitraryError::Construction(err) => err.recover(),
            ArbitraryError::Integer(err) => err.recover(),
        }
    }
}

#[cfg(any(test, feature = "quickcheck"))]
impl<Construction, Integer> crate::result::Void for ArbitraryError<Construction, Integer>
where
    Construction: crate::result::Void,
    Integer: crate::result::Void,
{
    fn absurd<T>(self) -> T {
        match self {
            ArbitraryError::Construction(err) => crate::result::Void::absurd(err),
            ArbitraryError::Integer(err) => crate::result::Void::absurd(err),
        }
    }
}

pub trait ConstructionError<G>: std::fmt::Debug {
    fn recover(self) -> G;
}

impl<G> ConstructionError<G> for Infallible {
    fn recover(self) -> G {
        match self {}
    }
}

#[allow(clippy::missing_errors_doc)]
pub trait GameFormContext {
    type Form: Clone;
    type BaseForm: Clone + std::fmt::Debug;

    type DicoticConstructionError: ConstructionError<Self::BaseForm>;
    type IntegerConstructionError: ConstructionError<Self::BaseForm>;
    type ConjugateConstructionError: ConstructionError<Self::BaseForm>;
    type SumConstructionError: ConstructionError<Self::BaseForm>;

    #[allow(clippy::wrong_self_convention)]
    fn new(
        &self,
        left: impl IntoIterator<Item = Self::Form>,
        right: impl IntoIterator<Item = Self::Form>,
    ) -> Result<Self::Form, Self::DicoticConstructionError>;

    fn moves<'a>(
        &self,
        game: &'a Self::Form,
        player: Player,
    ) -> impl Iterator<Item = &'a Self::Form>;

    fn new_integer(&self, n: i32) -> Result<Self::Form, Self::IntegerConstructionError> {
        use std::cmp::Ordering;

        match n.cmp(&0) {
            Ordering::Less => {
                let previous = self.new_integer(n + 1)?;
                Ok(self.new([], [previous]).unwrap())
            }
            Ordering::Equal => Ok(self.new([], []).unwrap()),
            Ordering::Greater => {
                let previous = self.new_integer(n - 1)?;
                Ok(self.new([previous], []).unwrap())
            }
        }
    }

    fn to_integer(&self, game: &Self::Form) -> Option<i32> {
        let mut left = self.moves(game, Player::Left);
        let l1 = left.next();
        let l2 = left.next();

        let mut right = self.moves(game, Player::Right);
        let r1 = right.next();
        let r2 = right.next();

        match (l1, l2, r1, r2) {
            (None, _, None, _) => Some(0),
            (Some(gl), None, None, None) => {
                let prev = self.to_integer(gl)?;
                (prev >= 0).then_some(prev + 1)
            }
            (None, None, Some(gr), None) => {
                let prev = self.to_integer(gr)?;
                (prev <= 0).then_some(prev - 1)
            }
            _ => None,
        }
    }

    fn player_outcome(&self, game: &Self::Form, player: Player) -> Player {
        if self.wins_going_first(game, player) {
            player
        } else {
            player.opposite()
        }
    }

    fn wins_going_first(&self, game: &Self::Form, player: Player) -> bool {
        self.moves(game, player).count() == 0
            || self
                .moves(game, player)
                .any(|g| !self.wins_going_first(g, player.opposite()))
    }

    fn outcome(&self, game: &Self::Form) -> Outcome {
        match (
            self.wins_going_first(game, Player::Left),
            self.wins_going_first(game, Player::Right),
        ) {
            (true, true) => Outcome::N,
            (true, false) => Outcome::L,
            (false, true) => Outcome::R,
            (false, false) => Outcome::P,
        }
    }

    fn is_p_free(&self, game: &Self::Form) -> bool {
        (self.outcome(game) != Outcome::P)
            && Player::forall(|p| self.moves(game, p).all(|g| self.is_p_free(g)))
    }

    fn is_end(&self, game: &Self::Form, player: Player) -> bool {
        self.moves(game, player).count() == 0
    }

    fn is_dead_end(&self, game: &Self::Form, player: Player) -> bool {
        self.is_end(game, player)
            && self
                .moves(game, player.opposite())
                .all(|g| self.is_dead_end(g, player))
    }

    fn is_dead_ending(&self, game: &Self::Form) -> bool {
        Player::forall(|p| !self.is_end(game, p) || self.is_dead_end(game, p))
            && Player::forall(|p| self.moves(game, p).all(|g| self.is_dead_ending(g)))
    }

    fn is_blocked_end(&self, game: &Self::Form, p: Player) -> bool {
        self.is_end(game, p)
            && self.moves(game, p.opposite()).all(|gr| {
                self.is_blocked_end(gr, p)
                    || self.moves(gr, p).any(|grl| self.is_blocked_end(grl, p))
            })
    }

    fn is_blocking(&self, game: &Self::Form) -> bool {
        Player::forall(|p| !self.is_end(game, p) || self.is_blocked_end(game, p))
            && Player::forall(|p| self.moves(game, p).all(|g| self.is_blocking(g)))
    }

    fn total_cmp(&self, lhs: &Self::Form, rhs: &Self::Form) -> Ordering;

    fn total_eq(&self, lhs: &Self::Form, rhs: &Self::Form) -> bool;

    fn next_day(&self, day: &[Self::Form]) -> impl Iterator<Item = Self::Form> {
        use itertools::Itertools;

        day.iter().powerset().flat_map(move |left_moves| {
            day.iter().powerset().filter_map(move |right_moves| {
                self.new(
                    left_moves.clone().into_iter().cloned().collect::<Vec<_>>(),
                    right_moves.into_iter().cloned().collect::<Vec<_>>(),
                )
                .ok()
            })
        })
    }

    fn conjugate(&self, game: &Self::Form) -> Result<Self::Form, Self::ConjugateConstructionError> {
        Ok(self
            .new(
                self.moves(game, Player::Right)
                    .map(|gl| self.conjugate(gl))
                    .collect::<Result<Vec<_>, _>>()?,
                self.moves(game, Player::Left)
                    .map(|gr| self.conjugate(gr))
                    .collect::<Result<Vec<_>, _>>()?,
            )
            .unwrap())
    }

    fn sum(
        &self,
        g: &Self::Form,
        h: &Self::Form,
    ) -> Result<Self::Form, Self::SumConstructionError> {
        let mut left = Vec::new();
        for gl in self.moves(g, Player::Left) {
            left.push(self.sum(gl, h)?);
        }
        for hl in self.moves(h, Player::Left) {
            left.push(self.sum(g, hl)?);
        }

        let mut right = Vec::new();
        for gr in self.moves(g, Player::Right) {
            right.push(self.sum(gr, h)?);
        }
        for hr in self.moves(h, Player::Right) {
            right.push(self.sum(g, hr)?);
        }

        Ok(self.new(left, right).unwrap())
    }

    fn birthday(&self, game: &Self::Form) -> u32 {
        self.moves(game, Player::Left)
            .chain(self.moves(game, Player::Right))
            .map(|g| self.birthday(g) + 1)
            .max()
            .unwrap_or(0)
    }

    fn parse_list<'a>(&'a self, mut p: Parser<'a>) -> Option<(Parser<'a>, Vec<Self::Form>)> {
        let mut acc = Vec::new();
        loop {
            match lexeme!(p, |p| self.parse(p)) {
                Some((cf_p, cf)) => {
                    acc.push(cf);
                    p = cf_p;
                    p = p.trim_whitespace();
                    match p.parse_ascii_char(',') {
                        Some(pp) => {
                            p = pp.trim_whitespace();
                        }
                        None => return Some((p, acc)),
                    }
                }
                None => return Some((p, acc)),
            }
        }
    }

    fn parse<'a>(&'a self, p: Parser<'a>) -> Option<(Parser<'a>, Self::Form)> {
        let p = p.trim_whitespace();
        if let Some(p) = p.parse_ascii_char('{') {
            let (p, left) = try_option!(self.parse_list(p));
            let p = try_option!(p.parse_ascii_char('|'));
            let (p, right) = try_option!(self.parse_list(p));
            let p = try_option!(p.parse_ascii_char('}'));
            let p = p.trim_whitespace();
            Some((p, self.new(left, right).ok()?))
        } else {
            // TODO: Generalize number parsers
            let (p, integer) = try_option!(lexeme!(p, Parser::parse_i64));
            Some((p, self.new_integer(integer as i32).ok()?))
        }
    }

    #[allow(clippy::wrong_self_convention)]
    fn from_str(&self, input: &str) -> Option<Self::Form> {
        let (_, g) = self.parse(Parser::new(input))?;
        Some(g)
    }

    fn to_string(&self, game: &Self::Form) -> String {
        self.display(game).to_string()
    }

    fn display<'a>(&'a self, game: &'a Self::Form) -> impl std::fmt::Display + 'a {
        DisplayGame {
            context: self,
            form: game,
        }
    }

    fn fmt(&self, game: &Self::Form, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.to_integer(game) {
            Some(n) => write!(f, "{n}"),
            None => {
                write!(f, "{{")?;
                for (idx, gl) in self.moves(game, Player::Left).enumerate() {
                    if idx > 0 {
                        write!(f, ",")?;
                    }
                    self.fmt(gl, f)?;
                }
                write!(f, "|")?;
                for (idx, gr) in self.moves(game, Player::Right).enumerate() {
                    if idx > 0 {
                        write!(f, ",")?;
                    }
                    self.fmt(gr, f)?;
                }
                write!(f, "}}")
            }
        }
    }

    fn base(&self, game: Self::Form) -> Self::BaseForm;

    fn base_context(&self) -> &impl GameFormContext<Form = Self::BaseForm>;

    #[cfg(any(test, feature = "quickcheck"))]
    fn arbitrary_sized(
        &self,
        g: &mut quickcheck::Gen,
        size: i64,
    ) -> Result<
        Self::Form,
        ArbitraryError<Self::DicoticConstructionError, Self::IntegerConstructionError>,
    > {
        use quickcheck::Arbitrary;

        if size < 1 {
            return self.new_integer(0).map_err(ArbitraryError::Integer);
        }

        let is_integer = u8::arbitrary(g) < 64;
        if is_integer {
            let n = i64::arbitrary(g).rem_euclid(size);
            if bool::arbitrary(g) {
                self.new_integer(n as i32).map_err(ArbitraryError::Integer)
            } else {
                self.new_integer(-n as i32).map_err(ArbitraryError::Integer)
            }
        } else {
            let mut mk_player = || {
                let size = i64::arbitrary(g).rem_euclid(size);
                (0..size)
                    .filter_map(|_| self.arbitrary_sized(g, size - 1).ok())
                    .collect::<Vec<_>>()
            };
            let left = mk_player();
            let right = mk_player();

            self.new(left, right).map_err(ArbitraryError::Construction)
        }
    }

    #[cfg(any(test, feature = "quickcheck"))]
    fn arbitrary(
        &self,
        g: &mut quickcheck::Gen,
    ) -> Result<
        Self::Form,
        ArbitraryError<Self::DicoticConstructionError, Self::IntegerConstructionError>,
    > {
        self.arbitrary_sized(g, g.size() as i64)
    }

    #[cfg(any(test, feature = "quickcheck"))]
    #[allow(clippy::option_if_let_else)] // Needed for Box<&dyn>
    fn shrink<'a>(&'a self, game: &'a Self::Form) -> Box<dyn Iterator<Item = Self::Form> + 'a> {
        use itertools::Itertools;
        use std::cmp::Ordering;

        match self.to_integer(game) {
            Some(n) => match n.cmp(&0) {
                Ordering::Less => Box::new(((n + 1)..=0).filter_map(|n| self.new_integer(n).ok())),
                Ordering::Equal => Box::new(std::iter::empty()),
                Ordering::Greater => {
                    Box::new((0..n).rev().filter_map(|n| self.new_integer(n).ok()))
                }
            },
            None => {
                let left = (0..self.moves(game, Player::Left).count()).flat_map(|n| {
                    self.moves(game, Player::Left)
                        .collect::<Vec<_>>()
                        .into_iter()
                        .combinations(n)
                });
                let right = (0..self.moves(game, Player::Right).count()).flat_map(|n| {
                    self.moves(game, Player::Right)
                        .collect::<Vec<_>>()
                        .into_iter()
                        .combinations(n)
                });
                Box::new(left.cartesian_product(right).filter_map(|(left, right)| {
                    self.new(left.into_iter().cloned(), right.into_iter().cloned())
                        .ok()
                }))
            }
        }
    }
}

struct DisplayGame<'a, C>
where
    C: GameFormContext + ?Sized,
{
    context: &'a C,
    form: &'a C::Form,
}

impl<C> std::fmt::Display for DisplayGame<'_, C>
where
    C: GameFormContext + ?Sized,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.context.fmt(self.form, f)
    }
}
