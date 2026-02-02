use anyhow::Result;
use cgt::{
    misere::game_form::{GameFormContext, StandardFormContext},
    result::UnwrapInfallible,
    short::partizan::Player,
};
use std::convert::Infallible;

#[derive(Debug, clap::Parser)]
pub struct Args {
    /// Day to print
    #[arg(long)]
    day: u32,
    // TODO: Support variant
}

fn reduce<C>(context: &C, game: &C::Form) -> C::Form
where
    C: GameFormContext<DicoticConstructionError = Infallible>,
{
    let lowest_lhs = context
        .moves(game, Player::Left)
        .filter_map(|gl| context.to_integer(gl))
        .min()
        .unwrap_or(i32::MIN);
    let highest_rhs = context
        .moves(game, Player::Right)
        .filter_map(|gr| context.to_integer(gr))
        .max()
        .unwrap_or(i32::MAX);
    context
        .new(
            context
                .moves(game, Player::Left)
                .filter_map(|gl| match context.to_integer(gl) {
                    Some(gl) if gl > lowest_lhs => None,
                    _ => Some(gl),
                })
                .cloned(),
            context
                .moves(game, Player::Right)
                .filter_map(|gr| match context.to_integer(gr) {
                    Some(gr) if gr < highest_rhs => None,
                    _ => Some(gr),
                })
                .cloned(),
        )
        .unwrap_infallible()
}

fn next_day<C>(context: &C, previous_day: &[C::Form]) -> Vec<C::Form>
where
    C: GameFormContext<DicoticConstructionError = Infallible>,
{
    let mut this_day = context
        .next_day(previous_day)
        .filter(|g| context.is_p_free(g) && context.is_dead_ending(g))
        .map(|g| reduce(context, &g))
        .collect::<Vec<_>>();
    this_day.sort_unstable_by(|lhs, rhs| context.total_cmp(lhs, rhs));
    this_day.dedup_by(|lhs, rhs| context.total_eq(lhs, rhs));
    this_day
}

#[allow(clippy::needless_pass_by_value, clippy::unnecessary_wraps)]
pub fn run(args: Args) -> Result<()> {
    let context = StandardFormContext;
    let mut day = vec![context.new_integer(0).unwrap_infallible()];
    for _ in 0..args.day {
        day = next_day(&context, &day);
    }

    for g in day {
        println!("{g}");
    }

    Ok(())
}
