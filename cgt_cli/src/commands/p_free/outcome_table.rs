use crate::commands::p_free::order;
use anyhow::Result;
use cgt::misere::p_free::{GameForm, Outcome};
use clap::Parser;
use quickcheck::{Arbitrary, Gen};
use std::{
    collections::BTreeMap,
    sync::{Arc, atomic::AtomicBool},
};

/// Perform frobination (WIP, DO NOT USE)
#[derive(Parser, Debug, Clone)]
pub struct Args {
    #[arg(long)]
    size: u64,

    #[arg(long)]
    variant: order::Variant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct TippingPoints {
    gl: u32,
    gr: u32,
    hl: u32,
    hr: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct PossibleOutcome {
    outcomes: [bool; 4],
}

impl PossibleOutcome {
    #[inline(always)]
    const fn none() -> PossibleOutcome {
        PossibleOutcome {
            outcomes: [false; 4],
        }
    }

    #[inline(always)]
    const fn mark_as_possible(&mut self, outcome: Outcome) -> bool {
        let was_not_possible = self.outcomes[outcome as usize];
        self.outcomes[outcome as usize] = true;
        !was_not_possible
    }

    #[inline(always)]
    const fn has_outcome(self, outcome: Outcome) -> bool {
        self.outcomes[outcome as usize]
    }
}

struct Latex<T>(T);

impl<T> std::fmt::Display for Latex<T>
where
    T: TexDisplay,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

trait TexDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
}

impl std::fmt::Display for PossibleOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = f.debug_set();
        [Outcome::L, Outcome::R, Outcome::P, Outcome::N]
            .into_iter()
            .filter(|o| self.has_outcome(*o))
            .for_each(|o| {
                s.entry(&o);
            });
        s.finish()
    }
}

impl TexDisplay for PossibleOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\\{{")?;
        let mut first = true;
        [Outcome::L, Outcome::R, Outcome::P, Outcome::N]
            .into_iter()
            .filter(|o| self.has_outcome(*o))
            .try_for_each(|o| {
                if !first {
                    write!(f, ", ")?;
                }
                write!(f, "\\mathcal{{{o}}}")?;
                first = false;
                Ok(())
            })?;
        write!(f, "\\}}")
    }
}

#[allow(clippy::needless_pass_by_value)]
pub fn run(args: Args) -> Result<()> {
    let finished = Arc::new(AtomicBool::new(false));
    ctrlc::set_handler({
        let finished = Arc::clone(&finished);
        move || {
            finished.store(true, std::sync::atomic::Ordering::Relaxed);
        }
    })
    .unwrap();

    let mut known = BTreeMap::<TippingPoints, PossibleOutcome>::new();

    let mut rnd = Gen::new(args.size as usize);
    eprintln!("l(g), r(g), l(h), r(h), o(g + h), g, h");
    while !finished.load(std::sync::atomic::Ordering::Relaxed) {
        let g = GameForm::arbitrary(&mut rnd);
        let h = GameForm::arbitrary(&mut rnd);
        if g.is_p_free()
            && args.variant.matches(&g)
            && g.outcome() == Outcome::N
            && h.is_p_free()
            && args.variant.matches(&h)
            && h.outcome() == Outcome::N
        {
            let tipping_points = TippingPoints {
                gl: g.left_tipping_point(),
                gr: g.right_tipping_point(),
                hl: h.left_tipping_point(),
                hr: h.right_tipping_point(),
            };

            let sum = GameForm::sum(&g, &h);
            let outcome = sum.outcome();
            if known
                .entry(tipping_points)
                .or_insert(PossibleOutcome::none())
                .mark_as_possible(outcome)
            {
                eprintln!(
                    "{},    {},    {},    {},    {},        {}, {}",
                    tipping_points.gl,
                    tipping_points.gr,
                    tipping_points.hl,
                    tipping_points.hr,
                    outcome,
                    h,
                    g
                );
                dbg!(
                    tipping_points.gl < tipping_points.hr && tipping_points.gr > tipping_points.hl
                );
            }
        }
    }

    eprintln!();
    println!("l(g), r(g), l(h), r(h), o(g + h)");
    for (tipping_points, outcomes) in known {
        println!(
            "${}$ & ${}$ & ${}$ & ${}$ & ${}$ \\\\",
            tipping_points.gl,
            tipping_points.gr,
            tipping_points.hl,
            tipping_points.hr,
            Latex(outcomes),
        );
    }

    Ok(())
}
