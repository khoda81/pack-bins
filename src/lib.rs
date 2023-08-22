use std::collections;

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

        let mut searched_sizes = collections::HashSet::new();
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
