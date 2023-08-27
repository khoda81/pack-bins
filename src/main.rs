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
    #[arg(short, long)]
    values: bool,

    /// Try to minimize the number of bins to use
    #[arg(short, long)]
    minimize_bins: bool,
}

const PRINT_INTERVAL: &str = "200ms";

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Solvability<S> {
    Unsolvable,
    Solvable(S),
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum Solution<S> {
    #[default]
    Unknown,
    Known(S),
}

impl<S> Solution<S> {
    pub fn insert(&mut self, status: S) {
        match self {
            Self::Unknown => *self = Self::Known(status),
            Self::Known(_prev_result) => {}
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
            let row = bin
                .items()
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(" ");

            println!("v {}", row);
        });

    let is_sorted = best_fit.is_sorted_by_key(cmp::Reverse);
    println!("c sorted: {}", is_sorted);
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

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

    let mut solution = Solution::Unknown;
    let mut max_bins = weights.len();

    'find_solution: loop {
        println!("c Trying to fit in {max_bins} bins");

        let total_weight: u32 = weights.iter().sum();
        let total_size = bin_capacity * max_bins as u32;
        if total_weight > total_size {
            // unsolved
            solution.insert(Solvability::Unsolvable);
            break 'find_solution;
        }

        let bin_capacities = vec![bin_capacity; max_bins];
        let mut solver = fitter::Fitter::new(weights.clone(), bin_capacities);

        let initial_len = solver.items.len();
        let print_interval = humantime::parse_duration(PRINT_INTERVAL).unwrap();

        let mut solved_amount = 0.;
        let mut prev_print_amount = solved_amount;
        let mut min_items = initial_len;
        let mut next_print_time = time::Instant::now() + print_interval;

        let mut num_iters = 1;
        let start = time::Instant::now();

        while solver.step() {
            // print!("\x1b[2J\x1b[H"); // clear screen
            // println!(
            //     "c {}",
            //     solver
            //         .items
            //         .iter()
            //         .map(ToString::to_string)
            //         .collect::<Vec<_>>()
            //         .join(" ")
            // );

            // print_solution(&solver.bins);
            // io::stdin().read_line(&mut String::new())?;

            if solver.items.len() < min_items {
                min_items = solver.items.len();
                let m = min_items as f64;
                let l = initial_len as f64;
                solved_amount = 1. - (m * (l - 1.) / l + 1.).log(l);
            };

            if time::Instant::now() > next_print_time && prev_print_amount != solved_amount {
                // print
                next_print_time = time::Instant::now() + print_interval;
                prev_print_amount = solved_amount;
                println!("c {:.2}% solved (items={min_items})", solved_amount * 100.);
            }

            num_iters += 1;

            if let Some(deadline) = deadline {
                if time::Instant::now() > deadline {
                    // unknown
                    break 'find_solution;
                }
            }
        }

        let dur = start.elapsed();
        if num_iters > 0 {
            let time_per_iteration = dur / num_iters;
            println!("c {num_iters} iterations in {dur:?} ({time_per_iteration:?} per iteration)",);
        } else {
            println!("c warning: no iterations");
        }

        if solver.is_solved() {
            let bins = solver
                .bins
                .into_iter()
                .filter(|bin| !bin.is_empty())
                .collect::<Vec<_>>();

            max_bins = bins.len().saturating_sub(1);
            solution = Solution::Known(Solvability::Solvable(bins));
        } else {
            // unsolved
            solution.insert(Solvability::Unsolvable);
            break;
        }

        if max_bins == 0 || !args.minimize_bins {
            break;
        }
    }

    match solution {
        Solution::Unknown => println!("s UNKNOWN"),
        Solution::Known(Solvability::Unsolvable) => println!("s UNSAT"),

        Solution::Known(Solvability::Solvable(best_fit)) => {
            println!("s SAT");

            if args.values {
                print_solution(&best_fit);
            }
        }
    }

    anyhow::Ok(())
}
