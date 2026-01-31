use cgt::misere::p_free::GameForm;
use clap::{Parser, ValueEnum};
use quickcheck::{Arbitrary, Gen};

#[derive(ValueEnum, Debug, Clone, Copy)]
pub enum Variant {
    DeadEnding,
    Blocking,
}

impl Variant {
    pub fn matches(self, g: &GameForm) -> bool {
        match self {
            Variant::DeadEnding => g.is_dead_ending(),
            Variant::Blocking => g.is_blocking(),
        }
    }
}

#[derive(Parser, Debug)]
pub struct Args {
    #[arg(long, allow_hyphen_values = true)]
    pub lhs: GameForm,

    #[arg(long, allow_hyphen_values = true)]
    pub rhs: GameForm,

    /// Generator size
    #[arg(long)]
    pub size: u64,

    #[arg(long)]
    pub max_attempts: u64,

    #[arg(long, value_enum)]
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
pub enum Relation {
    /// Possibly equal
    PossiblyEqual,

    /// Possibly less than, guaranteed not to be greater or equal
    PossiblyLessThan { not_ge_witness: GameForm },

    /// Possibly greater than, guaranteed not to be less or equal
    PossiblyGreaterThan { not_le_witness: GameForm },

    /// Guaranteed to be incomparable
    Incomparable {
        not_ge_witness: GameForm,
        not_le_witness: GameForm,
    },
}

#[derive(Debug, Clone, Copy)]
enum CheckedOrder {
    LessThanOrEqual,
    GreaterThanOrEqual,
}

fn check_order(x: &GameForm, lhs: &GameForm, rhs: &GameForm, order: CheckedOrder) -> bool {
    let ol = GameForm::sum(lhs, x).outcome();
    let or = GameForm::sum(rhs, x).outcome();

    match order {
        CheckedOrder::GreaterThanOrEqual => ol < or,
        CheckedOrder::LessThanOrEqual => or < ol,
    }
}

fn shrink_witness(
    distinguisher: &GameForm,
    lhs: &GameForm,
    rhs: &GameForm,
    variant: Variant,
    order: CheckedOrder,
) -> Option<GameForm> {
    for shrunken in distinguisher.shrink() {
        if shrunken.is_p_free()
            && variant.matches(&shrunken)
            && check_order(&shrunken, lhs, rhs, order)
        {
            let more_shrunken = shrink_witness(&shrunken, lhs, rhs, variant, order);
            return Some(more_shrunken.unwrap_or(shrunken));
        }
    }
    None
}

/// Check if games satisfy specified `order` and return the distinguishing `x` if not
fn find_witness(args: &Args, order: CheckedOrder) -> SearchStatistics<Option<GameForm>> {
    let mut statistics = SearchStatistics {
        attempted: 0,
        skipped: 0,
        result: None,
    };

    let mut rnd = Gen::new(args.size as usize);
    for _ in 0..args.max_attempts {
        statistics.attempted += 1;
        let x = GameForm::arbitrary(&mut rnd);
        if x.is_p_free() && args.variant.matches(&x) {
            if check_order(&x, &args.lhs, &args.rhs, order) {
                statistics.result = Some(
                    shrink_witness(&x, &args.lhs, &args.rhs, args.variant, order).unwrap_or(x),
                );
                break;
            }
        } else {
            statistics.skipped += 1;
        }
    }

    statistics
}

pub fn check_relation_possibility(args: &Args) -> SearchStatistics<Relation> {
    let le = find_witness(args, CheckedOrder::LessThanOrEqual);
    let ge = find_witness(args, CheckedOrder::GreaterThanOrEqual);

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

#[allow(clippy::needless_pass_by_value, clippy::unnecessary_wraps)]
pub fn run(args: Args) -> anyhow::Result<()> {
    let stats = check_relation_possibility(&args);
    eprintln!("Attempted: {}", stats.attempted);
    eprintln!(
        "Ignored: {} ({:.2}%)",
        stats.skipped,
        stats.skipped as f32 / stats.attempted as f32 * 100.0
    );

    match stats.result {
        Relation::PossiblyEqual => {
            println!("Possibly: {} = {}", args.lhs, args.rhs);
        }
        Relation::PossiblyLessThan { not_ge_witness } => {
            println!("Witness for not (>=): {not_ge_witness}");
            println!(
                "o({} + {}) = {}",
                args.lhs,
                not_ge_witness,
                GameForm::sum(&args.lhs, &not_ge_witness).outcome()
            );
            println!(
                "o({} + {}) = {}",
                args.rhs,
                not_ge_witness,
                GameForm::sum(&args.rhs, &not_ge_witness).outcome()
            );
            println!("Guaranteed: {} != {}", args.rhs, args.lhs);
            println!("Possibly: {} > {}", args.rhs, args.lhs);
        }
        Relation::PossiblyGreaterThan { not_le_witness } => {
            println!("Witness for not (<=): {not_le_witness}");
            println!(
                "o({} + {}) = {} > {} = o({} + {})",
                args.lhs,
                not_le_witness,
                GameForm::sum(&args.lhs, &not_le_witness).outcome(),
                GameForm::sum(&args.rhs, &not_le_witness).outcome(),
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
            println!("Witness for not (>=): {not_ge_witness}");
            println!(
                "o({} + {}) = {}",
                args.lhs,
                not_ge_witness,
                GameForm::sum(&args.lhs, &not_ge_witness).outcome()
            );
            println!(
                "o({} + {}) = {}",
                args.rhs,
                not_ge_witness,
                GameForm::sum(&args.rhs, &not_ge_witness).outcome()
            );
            println!("Witness for not (<=): {not_le_witness}");
            println!(
                "o({} + {}) = {}",
                args.lhs,
                not_le_witness,
                GameForm::sum(&args.lhs, &not_le_witness).outcome()
            );
            println!(
                "o({} + {}) = {}",
                args.rhs,
                not_le_witness,
                GameForm::sum(&args.rhs, &not_le_witness).outcome()
            );
            println!("Guaranteed: {} || {}", args.lhs, args.rhs);
        }
    }

    Ok(())
}
