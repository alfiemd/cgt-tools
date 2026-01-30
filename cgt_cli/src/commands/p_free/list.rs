use anyhow::Result;
use cgt::{misere::p_free::GameForm, short::partizan::Player, total::TotalWrappable};

#[derive(Debug, clap::Parser)]
pub struct Args {
    /// Day to print
    #[arg(long)]
    day: u32,
    // TODO: Support variant
}

fn reduce(game: &GameForm) -> GameForm {
    let lowest_lhs = game
        .moves(Player::Left)
        .iter()
        .filter_map(|gl| gl.to_integer())
        .min()
        .unwrap_or(i32::MIN);
    let highest_rhs = game
        .moves(Player::Right)
        .iter()
        .filter_map(|gl| gl.to_integer())
        .max()
        .unwrap_or(i32::MAX);
    GameForm::new(
        game.moves(Player::Left)
            .iter()
            .filter_map(|gl| match gl.to_integer() {
                Some(gl) if gl > lowest_lhs => None,
                _ => Some(gl),
            })
            .cloned()
            .collect(),
        game.moves(Player::Right)
            .iter()
            .filter_map(|gr| match gr.to_integer() {
                Some(gr) if gr < highest_rhs => None,
                _ => Some(gr),
            })
            .cloned()
            .collect(),
    )
}

fn next_day(previous_day: &[GameForm]) -> Vec<GameForm> {
    let mut this_day = GameForm::next_day(previous_day)
        .filter(|g| g.is_p_free() && g.is_dead_ending())
        .map(|g| reduce(&g))
        .collect::<Vec<_>>();
    this_day.sort_unstable_by(|g, h| g.total_cmp(h));
    this_day.dedup_by(|g, h| g.total_eq(&h));
    this_day
}

pub fn run(args: Args) -> Result<()> {
    let mut day = vec![GameForm::new_integer(0)];
    for _ in 0..args.day {
        day = next_day(&day);
    }

    for g in day {
        println!("{g}");
    }

    Ok(())
}
