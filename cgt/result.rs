//! Utilities for working with the `Result` type

#![allow(missing_docs)]

use std::convert::Infallible;

pub trait Void {
    fn absurd<T>(self) -> T;
}

impl Void for Infallible {
    fn absurd<T>(self) -> T {
        match self {}
    }
}

pub trait UnwrapInfallible {
    type R;
    fn unwrap_infallible(self) -> Self::R;
}

impl<T, E> UnwrapInfallible for Result<T, E>
where
    E: Void,
{
    type R = T;
    fn unwrap_infallible(self) -> Self::R {
        match self {
            Ok(res) => res,
            Err(err) => err.absurd(),
        }
    }
}
