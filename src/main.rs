use clap::Parser;
use std::{
    collections::HashSet,
    fs,
    io::{self, Read},
    path::PathBuf,
    process::exit,
    sync::{mpsc, Arc, Mutex},
    thread,
    time::Duration,
};

pub struct Fitter<T> {
    weights: Vec<T>,
    bin_capacities: Vec<T>,
    bins: Vec<Vec<T>>,
}

impl<T> Fitter<T>
where
    T: Ord + Copy + std::hash::Hash + std::ops::SubAssign + std::iter::Sum,
{
    pub fn new(mut weights: Vec<T>, bin_capacities: Vec<T>) -> Self {
        weights.sort();

        Self {
            bins: vec![Vec::new(); bin_capacities.len()],
            bin_capacities,
            weights,
        }
    }

    pub fn fit_helper(&mut self) -> bool {
        let current_item = match self.weights.pop() {
            Some(x) => x,
            None => return true,
        };

        let mut searched_sizes = HashSet::new();
        for bag_idx in 0..self.bin_capacities.len() {
            if self.bin_capacities[bag_idx] < current_item {
                continue;
            }

            if !searched_sizes.insert(self.bin_capacities[bag_idx]) {
                continue;
            }

            // try
            let old_size = self.bin_capacities[bag_idx];
            self.bin_capacities[bag_idx] -= current_item;
            self.bins[bag_idx].push(current_item);

            // recurse
            if self.fit_helper() {
                return true;
            }

            // backtrack
            self.bins[bag_idx].pop();
            self.bin_capacities[bag_idx] = old_size;
        }

        self.weights.push(current_item);
        false
    }

    pub fn fit(mut self) -> Option<Vec<Vec<T>>> {
        let total_weight: T = self.weights.iter().copied().sum();
        let total_size: T = self.bin_capacities.iter().copied().sum();

        if total_weight <= total_size && self.fit_helper() {
            Some(self.bins)
        } else {
            None
        }
    }
}

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input file to parse (uses stdin by default)
    #[arg(short, long)]
    input_file: Option<PathBuf>,

    /// Timeout for the computation
    #[arg(short, long)]
    timeout: Option<humantime::Duration>,

    /// Don't show the values
    #[arg(long)]
    no_values: bool,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    use text_io::read;

    let mut stream = if let Some(path) = args.input_file {
        let file = fs::File::open(path)?;
        let reader = io::BufReader::new(file);
        let stream = reader.bytes().map(Result::unwrap);
        Box::new(stream) as Box<dyn Iterator<Item = u8>>
    } else {
        let stream = io::stdin().bytes().map(Result::unwrap);
        Box::new(stream) as Box<dyn Iterator<Item = u8>>
    };

    let bin_capacity: u32 = read!("{}", &mut stream);
    let mut weights = Vec::new();
    while let nonzero @ 1.. = read!("{}", &mut stream) {
        weights.push(nonzero)
    }

    let mutex = Arc::new(Mutex::new(None));
    let other_fit = mutex.clone();
    let (tx, rx) = mpsc::channel();

    let timeout_thread = thread::spawn(move || {
        if let Some(timeout) = args.timeout {
            if rx.recv_timeout(timeout.into()).is_err() {
                println!("c Timed out after {timeout}");
            }
        } else {
            // wait for the main thread to finish computation
            let _ = rx.recv();
        }

        let best_fit: Option<Vec<Vec<_>>> = other_fit.lock().unwrap().take();
        match best_fit {
            Some(best_fit) => {
                println!("s SAT");

                if !args.no_values {
                    for row in best_fit {
                        println!(
                            "v {}",
                            row.iter()
                                .map(ToString::to_string)
                                .collect::<Vec<_>>()
                                .join(" ")
                        );
                    }
                }
            }
            None => {
                println!("s UNSAT");
            }
        }

        exit(0);
    });

    let mut max_bins = weights.len();

    loop {
        println!("c Fitting to {} bins", max_bins);
        let current_fit: Option<Vec<Vec<_>>> =
            Fitter::new(weights.clone(), vec![bin_capacity; max_bins])
                .fit()
                .map(|fit| fit.into_iter().filter(|row| !row.is_empty()).collect());

        if let Some(current_fit) = current_fit {
            max_bins = current_fit.len().saturating_sub(1);
            let _ = mutex.lock().unwrap().insert(current_fit);
        } else {
            break;
        }

        if max_bins == 0 {
            break;
        }
    }

    let _ = tx.send(());
    let _ = timeout_thread.join();

    anyhow::Ok(())
}
