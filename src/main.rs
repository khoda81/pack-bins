#![feature(deadline_api)]
#![feature(is_sorted)]

use clap::Parser;
use std::io::Read;

use std::{cmp, fs, io, path, time};

/// A backtracking solution to bin packing problem
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input file to parse (uses stdin by default)
    #[arg(short, long)]
    input_file: Option<path::PathBuf>,

    /// Timeout for the computation
    #[arg(short, long)]
    timeout: Option<humantime::Duration>,

    /// Show the values
    #[arg(long)]
    values: bool,

    /// Try to minimize the number of bins to use
    #[arg(short, long)]
    minimize_bins: bool,

    #[command(flatten)]
    verbose: clap_verbosity_flag::Verbosity<clap_verbosity_flag::WarnLevel>,
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum SolutionState<S> {
    #[default]
    Unknown,
    Unsolvable,
    Solved(S),
}

impl<S> SolutionState<S> {
    pub fn insert(&mut self, status: Self) {
        if let Self::Unknown = self {
            *self = status
        };
    }
}

fn parse_input(stream: impl Read) -> anyhow::Result<(u32, Vec<u32>)> {
    use text_io::try_read;

    let mut stream = stream.bytes().map(Result::unwrap);
    let bin_capacity = try_read!("{}", &mut stream)?;
    let mut weights = Vec::new();
    while let nonzero @ 1.. = try_read!("{}", &mut stream)? {
        weights.push(nonzero)
    }

    Ok((bin_capacity, weights))
}

fn print_solution(best_fit: &[fitter::Bin<u32>]) {
    best_fit
        .iter()
        // .filter(|bin| !bin.is_empty())
        .for_each(|bin| {
            let line = bin
                .items()
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(" ");

            println!("v {}", line);
        });

    let is_sorted = best_fit.is_sorted_by_key(cmp::Reverse);
    log::debug!("c Is sorted: {}", is_sorted);
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let mut builder = env_logger::Builder::from_default_env();
    builder.filter_level(args.verbose.log_level_filter());

    // Custom formatting for log
    builder.format(|buf, record| {
        use std::io::Write;

        let mut subtle = buf.style();
        subtle.set_color(env_logger::fmt::Color::Black);
        subtle.set_intense(true);

        write!(buf, "c ")?;

        write!(buf, "{}", subtle.value("["))?;
        write!(buf, "{:<5}", buf.default_styled_level(record.level()))?;
        write!(buf, " {}", buf.timestamp())?;
        write!(buf, "{}", subtle.value("]"))?;

        writeln!(buf, " {}", record.args())?;

        Ok(())
    });

    // Initialize the logger
    builder.init();

    let stream: Box<dyn Read> = if let Some(path) = args.input_file {
        let file = fs::File::open(path)?;
        let reader = io::BufReader::new(file);

        Box::new(reader)
    } else {
        Box::new(io::stdin())
    };

    let (bin_capacity, weights) = parse_input(stream)?;
    let solve_start = time::Instant::now();
    let deadline = args.timeout.map(|timeout| solve_start + timeout.into());

    let mut solution = SolutionState::Unknown;
    let mut max_bins = weights.len();

    'optimize: loop {
        log::info!("Trying to fit in {max_bins} bins");

        let total_weight: u32 = weights.iter().sum();
        let total_size = bin_capacity * max_bins as u32;
        if total_weight > total_size {
            solution.insert(SolutionState::Unsolvable);
            break 'optimize;
        }

        let bin_capacities = vec![bin_capacity; max_bins];
        let mut solver = fitter::Fitter::new(weights.clone(), bin_capacities);

        let time_out = if let Some(deadline) = deadline {
            !solver.solve_until(|| time::Instant::now() < deadline)
        } else {
            !solver.solve_until(|| true)
        };

        if time_out {
            break 'optimize;
        }

        if solver.is_solved() {
            let bins = solver
                .bins
                .into_iter()
                .filter(|bin| !bin.is_empty())
                .collect::<Vec<_>>();

            max_bins = bins.len().saturating_sub(1);
            solution = SolutionState::Solved(bins);
            if max_bins > 0 && args.minimize_bins {
                continue 'optimize;
            }
        }

        // unsolved
        solution.insert(SolutionState::Unsolvable);
        break;
    }

    match solution {
        SolutionState::Unknown => println!("s UNKNOWN"),
        SolutionState::Unsolvable => println!("s UNSAT"),
        SolutionState::Solved(solution) => {
            println!("s SAT");

            if args.values {
                print_solution(&solution);
            }
        }
    }

    anyhow::Ok(())
}
