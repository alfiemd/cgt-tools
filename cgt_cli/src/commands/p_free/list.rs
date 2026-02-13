use std::{
    collections::HashSet,
    io::{self, BufWriter, Stdout},
};

use crate::io::FilePathOr;
use anyhow::Result;
use cgt::{
    misere::game_form::{
        DeadEndingFormContext, GameFormContext, PFreeDeadEndingContext, PFreeDeadEndingFormContext,
        PFreeFormContext, StandardFormContext,
    },
    result::{UnwrapInfallible, Void},
    total::{TotalWrappable, TotalWrapper},
};
use itertools::Itertools;

#[derive(Debug, clap::Parser)]
pub struct Args {
    /// Day to print
    #[arg(long)]
    day: u32,

    #[arg(long, default_value = "-")]
    output: FilePathOr<Stdout>,

    #[arg(long, default_value_t = false)]
    print_equal: bool,
    // TODO: Support variant
}

fn next_day<C>(context: &C, previous_day: &[C::Form]) -> Vec<C::Form>
where
    C: PFreeDeadEndingContext,
    C::IntegerConstructionError: Void,
    C::Form: TotalWrappable,
{
    let mut this_day = context
        .next_day(previous_day)
        .filter(|g| context.is_p_free(g) && context.is_dead_ending(g))
        .map(|g| context.reduced(&g))
        .dedup_by(|lhs, rhs| context.total_eq(lhs, rhs))
        .map(TotalWrapper::new)
        .collect::<HashSet<_>>()
        .into_iter()
        .map(TotalWrapper::get)
        .collect::<Vec<_>>();
    this_day.sort_unstable_by(|lhs, rhs| context.total_cmp(lhs, rhs));
    this_day
}

fn print_equal<C>(context: &C, games: &[C::Form])
where
    C: PFreeDeadEndingContext,
    C::IntegerConstructionError: Void,
    C::Form: TotalWrappable,
{
    let mut seen = vec![false; games.len()];

    for i in 0..games.len() {
        if seen[i] {
            continue;
        }

        eprint!("{}", context.display(&games[i]),);
        seen[i] = true;

        for j in (i + 1)..games.len() {
            if !seen[j] && context.eq_mod_p_free_dead_ending(&games[i], &games[j]) {
                seen[j] = true;
                eprint!(" = {}", context.display(&games[j]));
            }
        }

        eprintln!();
    }
}

fn deduplicate_equal<C>(context: &C, games: &mut Vec<C::Form>)
where
    C: PFreeDeadEndingContext,
    C::IntegerConstructionError: Void,
    C::Form: TotalWrappable,
{
    let mut seen: Vec<C::Form> = Vec::new();
    games.retain(|g| {
        if seen.iter().any(|h| context.eq_mod_p_free_dead_ending(g, h)) {
            false
        } else {
            seen.push(g.clone());
            true
        }
    });
}

fn generate_hasse<C, W>(context: &C, mut w: W, day: &[C::Form]) -> io::Result<()>
where
    C: PFreeDeadEndingContext,
    C::IntegerConstructionError: Void,
    W: io::Write,
{
    writeln!(w, "graph Hasse {{")?;
    writeln!(w, "  rankdir=BT;")?;

    for i in 0..day.len() {
        writeln!(
            w,
            "  {} [label = \"{}\", texlbl = \"${}$\"]",
            i,
            context.display(&day[i]),
            context.display_tex(&day[i]),
        )?;

        'inner: for j in 0..day.len() {
            if i == j || !context.ge_mod_p_free_dead_ending(&day[j], &day[i]) {
                continue;
            }

            for k in 0..day.len() {
                if k == i || k == j {
                    continue;
                }

                if context.ge_mod_p_free_dead_ending(&day[j], &day[k])
                    && context.ge_mod_p_free_dead_ending(&day[k], &day[i])
                {
                    continue 'inner;
                }
            }

            writeln!(w, "  {} -- {};", i, j)?;
        }
    }

    writeln!(w, "}}")
}

#[allow(clippy::needless_pass_by_value, clippy::unnecessary_wraps)]
pub fn run(args: Args) -> Result<()> {
    let mut output = BufWriter::new(args.output.create()?);

    let context = PFreeDeadEndingFormContext::new(PFreeFormContext::new(
        DeadEndingFormContext::new(StandardFormContext),
    ));
    let mut day = vec![context.new_integer(0).unwrap_infallible()];
    for _ in 0..args.day {
        day = next_day(&context, &day);
    }

    if args.print_equal {
        print_equal(&context, &day);
    }
    deduplicate_equal(&context, &mut day);
    generate_hasse(&context, &mut output, &day)?;

    Ok(())
}
