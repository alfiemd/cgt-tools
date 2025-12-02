//! Transposition tables for game values

use crate::short::partizan::canonical_form::CanonicalForm;
use std::{fmt::Debug, hash::Hash, marker::PhantomData};

mod dashmap;

pub use dashmap::ParallelTranspositionTable;

/// Interface of a transposition table
pub trait TranspositionTable<G> {
    /// Lookup a position value if exists
    fn lookup_position(&self, position: &G) -> Option<CanonicalForm>;

    /// Save position and its game value
    fn insert_position(&self, position: G, value: CanonicalForm);
}

/// Dummy transposition table that does not store anythning
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NoTranspositionTable<G>(PhantomData<G>);

impl<G> NoTranspositionTable<G> {
    #[inline]
    /// Create new dummy transposition table
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<G> Default for NoTranspositionTable<G> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<G> TranspositionTable<G> for NoTranspositionTable<G> {
    #[inline]
    fn lookup_position(&self, _position: &G) -> Option<CanonicalForm> {
        None
    }

    #[inline]
    fn insert_position(&self, _position: G, _value: CanonicalForm) {}
}
