use std::{
    collections::HashSet,
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

fn main() {
    use text_io::read;

    let bag_size = read!();
    let mut weights = Vec::new();
    while let nonzero @ 1.. = read!() {
        weights.push(nonzero)
    }

    let mutex = Arc::new(Mutex::new(None));
    let other_fit = mutex.clone();
    let (tx, rx) = mpsc::channel();
    let timeout = Duration::from_micros(100);

    let timeout_thread = thread::spawn(move || {
        if rx.recv_timeout(timeout).is_err() {
            println!("c Timed out");
        }

        let best_fit: Option<Vec<Vec<_>>> = other_fit.lock().unwrap().take();
        match best_fit {
            Some(best_fit) => {
                println!("s SAT");

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
            Fitter::new(weights.clone(), vec![bag_size; max_bins])
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
}
