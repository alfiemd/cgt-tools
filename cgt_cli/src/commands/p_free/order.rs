use cgt::{
    misere::{
        game_form::{GameFormContext, StandardFormContext},
        p_free::{PFreeForm, PFreeFormContext},
    },
    parsing::Parser,
    result::UnwrapInfallible,
};
use clap::ValueEnum;
use quickcheck::Gen;
use std::{convert::Infallible, fmt::Display};

#[derive(ValueEnum, Debug, Clone, Copy)]
pub enum Variant {
    DeadEnding,
    Blocking,
}

impl Variant {
    pub fn matches<C>(self, context: &C, g: &C::Form) -> bool
    where
        C: GameFormContext,
    {
        match self {
            Variant::DeadEnding => context.is_dead_ending(g),
            Variant::Blocking => context.is_blocking(g),
        }
    }
}

#[derive(clap::Parser, Debug)]
pub struct Args {
    #[arg(long, allow_hyphen_values = true)]
    pub lhs: String,

    #[arg(long, allow_hyphen_values = true)]
    pub rhs: String,

    /// Generator size
    #[arg(long)]
    pub size: u64,

    #[arg(long)]
    pub max_attempts: u64,

    #[arg(long, value_enum)]
    pub variant: Variant,
}

// clap cannot set generic bounds so here we go
pub struct RichArgs<G> {
    pub lhs: G,
    pub rhs: G,
    pub size: u64,
    pub max_attempts: u64,
    pub variant: Variant,
}

pub struct SearchStatistics<T> {
    /// Number of total generated potential witnesses
    pub attempted: u64,

    /// Number of non-tested witnesses due to not being P-free or in the specified variant
    pub skipped: u64,

    pub result: T,
}

#[derive(Debug, Clone)]
pub enum Relation<G> {
    /// Possibly equal
    PossiblyEqual,

    /// Possibly less than, guaranteed not to be greater or equal
    PossiblyLessThan { not_ge_witness: G },

    /// Possibly greater than, guaranteed not to be less or equal
    PossiblyGreaterThan { not_le_witness: G },

    /// Guaranteed to be incomparable
    Incomparable {
        not_ge_witness: G,
        not_le_witness: G,
    },
}

#[derive(Debug, Clone, Copy)]
enum CheckedOrder {
    LessThanOrEqual,
    GreaterThanOrEqual,
}

fn check_order<C>(
    context: &PFreeFormContext<C>,
    x: &PFreeForm<C::Form>,
    lhs: &PFreeForm<C::Form>,
    rhs: &PFreeForm<C::Form>,
    order: CheckedOrder,
) -> bool
where
    C: GameFormContext<SumConstructionError = Infallible>,
{
    let ol = context.underlying().outcome(
        &context
            .underlying()
            .sum(lhs.underlying(), x.underlying())
            .unwrap_infallible(),
    );
    let or = context.underlying().outcome(
        &context
            .underlying()
            .sum(rhs.underlying(), x.underlying())
            .unwrap_infallible(),
    );

    match order {
        CheckedOrder::GreaterThanOrEqual => ol < or,
        CheckedOrder::LessThanOrEqual => or < ol,
    }
}

fn shrink_witness<C>(
    context: &PFreeFormContext<C>,
    distinguisher: &PFreeForm<C::Form>,
    lhs: &PFreeForm<C::Form>,
    rhs: &PFreeForm<C::Form>,
    variant: Variant,
    order: CheckedOrder,
) -> Option<PFreeForm<C::Form>>
where
    C: GameFormContext<IntegerConstructionError = Infallible, SumConstructionError = Infallible>,
{
    for shrunken in context.shrink(distinguisher) {
        if variant.matches(context, &shrunken) // TODO: Pass DeadEndingFormContext instead
            && check_order(context, &shrunken, lhs, rhs, order)
        {
            let more_shrunken = shrink_witness(context, &shrunken, lhs, rhs, variant, order);
            return Some(more_shrunken.unwrap_or(shrunken));
        }
    }
    None
}

/// Check if games satisfy specified `order` and return the distinguishing `x` if not
fn find_witness<C>(
    context: &PFreeFormContext<C>,
    args: &RichArgs<PFreeForm<C::Form>>,
    order: CheckedOrder,
) -> SearchStatistics<Option<PFreeForm<C::Form>>>
where
    C: GameFormContext<IntegerConstructionError = Infallible, SumConstructionError = Infallible>,
{
    let mut statistics = SearchStatistics {
        attempted: 0,
        skipped: 0,
        result: None,
    };

    let mut rnd = Gen::new(args.size as usize);
    for _ in 0..args.max_attempts {
        statistics.attempted += 1;
        let Ok(x) = context.arbitrary(&mut rnd) else {
            statistics.skipped += 1;
            continue;
        };
        if args.variant.matches(context, &x) {
            if check_order(context, &x, &args.lhs, &args.rhs, order) {
                statistics.result = Some(
                    shrink_witness(context, &x, &args.lhs, &args.rhs, args.variant, order)
                        .unwrap_or(x),
                );
                break;
            }
        } else {
            statistics.skipped += 1;
        }
    }

    statistics
}

pub fn check_relation_possibility<C>(
    context: &PFreeFormContext<C>,
    args: &RichArgs<PFreeForm<C::Form>>,
) -> SearchStatistics<Relation<<PFreeFormContext<C> as GameFormContext>::Form>>
where
    C: GameFormContext<IntegerConstructionError = Infallible, SumConstructionError = Infallible>,
{
    let le = find_witness(context, args, CheckedOrder::LessThanOrEqual);
    let ge = find_witness(context, args, CheckedOrder::GreaterThanOrEqual);

    SearchStatistics {
        attempted: le.attempted + ge.attempted,
        skipped: le.attempted + ge.skipped,
        result: match (le.result, ge.result) {
            // No counterexample for either >= or <= found
            (None, None) => Relation::PossiblyEqual,
            (Some(not_le_witness), None) => Relation::PossiblyGreaterThan { not_le_witness },
            (None, Some(not_ge_witness)) => Relation::PossiblyLessThan { not_ge_witness },
            // Counterexample found for both >= and <= so the games are incomparable
            (Some(not_le_witness), Some(not_ge_witness)) => Relation::Incomparable {
                not_le_witness,
                not_ge_witness,
            },
        },
    }
}

fn show_results<C>(context: &PFreeFormContext<C>, args: &RichArgs<PFreeForm<C::Form>>)
where
    C: GameFormContext<IntegerConstructionError = Infallible, SumConstructionError = Infallible>,
    PFreeForm<C::Form>: Display,
{
    let stats = check_relation_possibility(context, args);
    eprintln!("Attempted: {}", stats.attempted);
    eprintln!(
        "Ignored: {} ({:.2}%)",
        stats.skipped,
        stats.skipped as f32 / stats.attempted as f32 * 100.0
    );

    let context = context.underlying();
    match stats.result {
        Relation::PossiblyEqual => {
            println!("Possibly: {} = {}", args.lhs, args.rhs);
        }
        Relation::PossiblyLessThan { not_ge_witness } => {
            println!("Witness for not (>=): {}", not_ge_witness);
            println!(
                "o({} + {}) = {}",
                args.lhs,
                not_ge_witness,
                context.outcome(
                    &context
                        .sum(args.lhs.underlying(), not_ge_witness.underlying())
                        .unwrap_infallible()
                )
            );
            println!(
                "o({} + {}) = {}",
                args.rhs,
                not_ge_witness,
                context.outcome(
                    &context
                        .sum(args.lhs.underlying(), not_ge_witness.underlying())
                        .unwrap_infallible()
                )
            );
            println!("Guaranteed: {} != {}", args.rhs, args.lhs);
            println!("Possibly: {} > {}", args.rhs, args.lhs);
        }
        Relation::PossiblyGreaterThan { not_le_witness } => {
            println!("Witness for not (<=): {}", not_le_witness);
            println!(
                "o({} + {}) = {} > {} = o({} + {})",
                args.lhs,
                not_le_witness,
                context.outcome(
                    &context
                        .sum(args.lhs.underlying(), not_le_witness.underlying())
                        .unwrap_infallible()
                ),
                context.outcome(
                    &context
                        .sum(args.rhs.underlying(), not_le_witness.underlying())
                        .unwrap_infallible()
                ),
                args.rhs,
                not_le_witness,
            );
            println!("Guaranteed: {} != {}", args.lhs, args.rhs);
            println!("Possibly: {} > {}", args.lhs, args.rhs);
        }
        Relation::Incomparable {
            not_ge_witness,
            not_le_witness,
        } => {
            println!("Witness for not (>=): {}", not_ge_witness);
            println!(
                "o({} + {}) = {}",
                args.lhs,
                not_ge_witness,
                context.outcome(
                    &context
                        .sum(args.lhs.underlying(), not_ge_witness.underlying())
                        .unwrap_infallible()
                )
            );
            println!(
                "o({} + {}) = {}",
                args.rhs,
                not_ge_witness,
                context.outcome(
                    &context
                        .sum(args.rhs.underlying(), not_ge_witness.underlying())
                        .unwrap_infallible()
                )
            );
            println!("Witness for not (<=): {}", not_le_witness);
            println!(
                "o({} + {}) = {}",
                args.lhs,
                not_le_witness,
                context.outcome(
                    &context
                        .sum(args.lhs.underlying(), not_le_witness.underlying())
                        .unwrap_infallible()
                )
            );
            println!(
                "o({} + {}) = {}",
                args.rhs,
                not_le_witness,
                context.outcome(
                    &context
                        .sum(args.rhs.underlying(), not_le_witness.underlying())
                        .unwrap_infallible()
                )
            );
            println!("Guaranteed: {} || {}", args.lhs, args.rhs);
        }
    }
}

#[allow(clippy::needless_pass_by_value, clippy::unnecessary_wraps)]
pub fn run(args: Args) -> anyhow::Result<()> {
    let context = PFreeFormContext::new(&StandardFormContext);
    let args = RichArgs {
        lhs: context
            .parse(Parser::new(&args.lhs))
            .ok_or_else(|| anyhow::anyhow!("Could not parse `lhs`"))?
            .1,
        rhs: context
            .parse(Parser::new(&args.rhs))
            .ok_or_else(|| anyhow::anyhow!("Could not parse `rhs`"))?
            .1,
        size: args.size,
        max_attempts: args.max_attempts,
        variant: args.variant,
    };
    show_results(context, &args);
    Ok(())
}
