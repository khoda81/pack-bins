use std::collections::HashSet;

use rand::Rng;

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
            bins: (0..bin_capacities.len()).map(|_| Vec::new()).collect(),
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

        (total_weight <= total_size && self.fit_helper()).then_some(self.bins)
    }

    pub fn find_min_fit(weights: Vec<T>, capacity: T) -> Vec<Vec<T>> {
        let fitter = Self::new(weights.clone(), vec![capacity; weights.len()]);
        let mut best_fit: Vec<_> = fitter
            .fit()
            .unwrap()
            .into_iter()
            .filter(|row| !row.is_empty())
            .collect();

        loop {
            let num_bins = best_fit.len();
            if num_bins == 0 {
                return best_fit;
            }

            let fitter = Self::new(weights.clone(), vec![capacity; num_bins - 1]);

            best_fit = match fitter.fit() {
                Some(fit) => fit.into_iter().filter(|row| !row.is_empty()).collect(),
                None => return best_fit,
            }
        }
    }
}

fn main() {
    use text_io::read;
    let mut weights = vec![];
    // while let nonzero @ 1.. = read!() {
    //     items.push(nonzero)
    // }

    let mut thread_rng = rand::thread_rng();
    for _ in 0..1000 {
        let random_item = thread_rng.gen_range(1..=500);
        weights.push(random_item)
    }

    let bag_size = 1000;

    let fitter = Fitter::new(weights.clone(), vec![bag_size; weights.len()]);
    let mut best_fit: Vec<_> = fitter
        .fit()
        .unwrap()
        .into_iter()
        .filter(|row| !row.is_empty())
        .collect();

    let fit = {
        loop {
            let bin_count = best_fit.len();

            show_fit(&best_fit);

            if bin_count == 0 {
                break best_fit;
            }

            let fitter = Fitter::new(weights.clone(), vec![bag_size; bin_count - 1]);

            best_fit = match fitter.fit() {
                Some(fit) => fit.into_iter().filter(|row| !row.is_empty()).collect(),
                None => break best_fit,
            }
        }
    };

    let fit = Some(fit);

    if let Some(fit) = fit {
        println!("s SAT");
        // show_fit(&fit);
    } else {
        println!("s UNSAT");
    }
}

fn show_fit<T: std::fmt::Display>(fit: &Vec<Vec<T>>) {
    println!("c Found with {}", fit.len());
    for row in fit {
        print!("c ");
        for cell in row.iter() {
            print!("{cell} ");
        }

        println!();
    }
}
