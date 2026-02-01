#![allow(missing_docs)]

use crate::{
    misere::game_form::{GameFormContext, Outcome, StandardForm, StandardFormContext},
    result::UnwrapInfallible,
    short::partizan::Player,
};
use std::{convert::Infallible, fmt};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct PFreeFormContext<C> {
    context: C,
}

impl<C> PFreeFormContext<C> {
    pub const fn new(context: &C) -> &Self {
        // SAFETY: We are #[repr(transparent)] so reference cast is safe
        unsafe { &*(::std::ptr::from_ref(context).cast::<Self>()) }
    }

    pub const fn underlying(&self) -> &C {
        &self.context
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct PFreeForm<G> {
    underlying: G,
}

impl<G> PFreeForm<G> {
    pub(crate) const fn new_unchecked(underlying: G) -> PFreeForm<G> {
        PFreeForm { underlying }
    }

    pub(crate) const fn new_ref_unchecked(underlying: &G) -> &PFreeForm<G> {
        // SAFETY: We are #[repr(transparent)] so reference cast is safe
        unsafe { &*(::std::ptr::from_ref(underlying).cast::<Self>()) }
    }

    pub const fn underlying(&self) -> &G {
        &self.underlying
    }
}

pub trait PFreeContext: GameFormContext<IntegerConstructionError = Infallible> {
    fn tipping_point(&self, game: &Self::Form, player: Player) -> u32 {
        match player {
            Player::Left => {
                let mut n = 0;
                loop {
                    if self.outcome(
                        &self
                            .sum(game, &self.new_integer(-(n as i32)).unwrap_infallible())
                            .unwrap(),
                    ) == Outcome::L
                    {
                        break n;
                    }
                    n += 1;
                }
            }
            Player::Right => {
                let mut n = 0;
                loop {
                    if self.outcome(
                        &self
                            .sum(game, &self.new_integer(n as i32).unwrap_infallible())
                            .unwrap(),
                    ) == Outcome::R
                    {
                        break n;
                    }
                    n += 1;
                }
            }
        }
    }

    fn left_tipping_point(&self, game: &Self::Form) -> u32 {
        for n in 0.. {
            let g = self
                .sum(game, &self.new_integer(-(n as i32)).unwrap_infallible())
                .unwrap();
            if self.outcome(&g) == Outcome::L {
                return n;
            }
        }

        unreachable!()
    }

    fn right_tipping_point(&self, game: &Self::Form) -> u32 {
        for n in 0.. {
            let g = self
                .sum(game, &self.new_integer(n as i32).unwrap_infallible())
                .unwrap();
            if self.outcome(&g) == Outcome::R {
                return n;
            }
        }

        unreachable!()
    }

    fn next_tipping_point(&self, game: &Self::Form) -> u32 {
        for n in 0.. {
            let g = self
                .sum(game, &self.new_integer(n as i32).unwrap_infallible())
                .unwrap();
            if self.outcome(&g) == Outcome::N {
                return n;
            }

            let g = self
                .sum(game, &self.new_integer(-(n as i32)).unwrap_infallible())
                .unwrap();
            if self.outcome(&g) == Outcome::N {
                return n;
            }
        }

        unreachable!()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PFreeConstructionError<E> {
    NotPFree,
    Underlying(E),
}

impl<C> GameFormContext for PFreeFormContext<C>
where
    C: GameFormContext<IntegerConstructionError = Infallible>,
{
    type Form = PFreeForm<C::Form>;

    type ConstructionError = PFreeConstructionError<C::ConstructionError>;

    type IntegerConstructionError = Infallible;

    type ConjugateConstructionError = C::ConjugateConstructionError;

    type SumConstructionError = PFreeConstructionError<C::SumConstructionError>;

    fn new(
        &self,
        left: impl IntoIterator<Item = Self::Form>,
        right: impl IntoIterator<Item = Self::Form>,
    ) -> Result<Self::Form, Self::ConstructionError> {
        self.context
            .new(
                left.into_iter().map(|g| g.underlying),
                right.into_iter().map(|g| g.underlying),
            )
            .map_err(PFreeConstructionError::Underlying)
            .and_then(|g| {
                self.context
                    .to_p_free(g)
                    .map_err(|_| PFreeConstructionError::NotPFree)
            })
    }

    fn moves<'a>(
        &self,
        game: &'a Self::Form,
        player: Player,
    ) -> impl Iterator<Item = &'a Self::Form> {
        self.context
            .moves(&game.underlying, player)
            .map(PFreeForm::new_ref_unchecked)
    }

    fn total_cmp(&self, lhs: &Self::Form, rhs: &Self::Form) -> std::cmp::Ordering {
        self.context.total_cmp(&lhs.underlying, &rhs.underlying)
    }

    fn total_eq(&self, lhs: &Self::Form, rhs: &Self::Form) -> bool {
        self.context.total_eq(&lhs.underlying, &rhs.underlying)
    }

    fn is_p_free(&self, _game: &Self::Form) -> bool {
        true
    }

    fn to_p_free(&self, game: Self::Form) -> Result<PFreeForm<Self::Form>, Self::Form> {
        Ok(PFreeForm::new_unchecked(game))
    }

    fn sum(
        &self,
        g: &Self::Form,
        h: &Self::Form,
    ) -> Result<Self::Form, Self::SumConstructionError> {
        self.context
            .sum(&g.underlying, &h.underlying)
            .map_err(PFreeConstructionError::Underlying)
            .and_then(|g| {
                self.context
                    .to_p_free(g)
                    .map_err(|_| PFreeConstructionError::NotPFree)
            })
    }
}

impl<C> PFreeContext for PFreeFormContext<C> where
    C: GameFormContext<IntegerConstructionError = Infallible>
{
}

impl std::fmt::Display for PFreeForm<StandardForm> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        StandardFormContext.fmt(&self.underlying, f)
    }
}
