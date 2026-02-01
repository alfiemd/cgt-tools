#![allow(missing_docs)]

use crate::{
    misere::game_form::GameFormContext,
    short::partizan::Player,
    total::{TotalWrappable, impl_total_wrapper},
};
use std::{cmp::Ordering, convert::Infallible, str::FromStr};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct StandardFormInner {
    left: Vec<StandardFormInner>,
    right: Vec<StandardFormInner>,
}

impl_total_wrapper! {
    #[derive(Debug, Clone)]
    pub struct StandardForm {
        inner: StandardFormInner
    }
}

impl StandardForm {
    fn new(left: impl IntoIterator<Item = Self>, right: impl IntoIterator<Item = Self>) -> Self {
        let mut left = StandardForm::into_inner_vec(left.into_iter().collect());
        left.sort();
        left.dedup();

        let mut right = StandardForm::into_inner_vec(right.into_iter().collect());
        right.sort();
        right.dedup();

        StandardForm {
            inner: StandardFormInner { left, right },
        }
    }

    fn moves(&self, player: Player) -> impl Iterator<Item = &Self> {
        match player {
            Player::Left => StandardForm::from_inner_slice(self.inner.left.as_slice()).iter(),
            Player::Right => StandardForm::from_inner_slice(self.inner.right.as_slice()).iter(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StandardFormContext;

impl GameFormContext for StandardFormContext {
    type Form = StandardForm;

    type ConstructionError = Infallible;
    type SumConstructionError = Infallible;
    type ConjugateConstructionError = Infallible;
    type IntegerConstructionError = Infallible;

    fn new(
        &self,
        left: impl IntoIterator<Item = Self::Form>,
        right: impl IntoIterator<Item = Self::Form>,
    ) -> Result<Self::Form, Self::ConstructionError> {
        Ok(StandardForm::new(left, right))
    }

    fn moves<'a>(
        &self,
        game: &'a Self::Form,
        player: Player,
    ) -> impl Iterator<Item = &'a Self::Form> {
        game.moves(player)
    }

    fn total_cmp(&self, lhs: &Self::Form, rhs: &Self::Form) -> Ordering {
        TotalWrappable::total_cmp(lhs, rhs)
    }

    fn total_eq(&self, lhs: &Self::Form, rhs: &Self::Form) -> bool {
        TotalWrappable::total_eq(lhs, rhs)
    }
}

impl FromStr for StandardForm {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match StandardFormContext.parse(crate::parsing::Parser::new(s)) {
            Some((p, result)) if p.input.is_empty() => Ok(result),
            Some(_) => Err("Parse error: leftover input"),
            None => Err("Parse error: parser failed"),
        }
    }
}

impl std::fmt::Display for StandardForm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        StandardFormContext.fmt(self, f)
    }
}
