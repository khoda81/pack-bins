use std::collections;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Bin<T> {
    capacity: T,
    items: Vec<T>,
}

impl<T> Bin<T>
where
    T: Clone + std::cmp::PartialOrd + std::ops::SubAssign + std::ops::AddAssign,
{
    pub fn new(capacity: T) -> Self {
        Self {
            capacity,
            items: Vec::new(),
        }
    }

    pub fn try_push(&mut self, item: T) -> Option<T> {
        if self.capacity >= item {
            self.capacity -= item.clone();
            self.items.push(item);

            None
        } else {
            Some(item)
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        self.items.pop().map(|item| {
            self.capacity += item.clone();
            item
        })
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn items(&self) -> &Vec<T> {
        &self.items
    }
}

pub struct Fitter<T> {
    items: collections::LinkedList<T>,
    bins: Vec<Bin<T>>,
}

impl<T> Fitter<T>
where
    T: Ord
        + Clone
        + std::hash::Hash
        + std::ops::SubAssign
        + std::ops::AddAssign
        + for<'a> std::iter::Sum<&'a T>,
{
    pub fn new(mut items: Vec<T>, bin_capacities: Vec<T>) -> Self {
        items.sort();

        Self {
            bins: bin_capacities.into_iter().map(Bin::new).collect(),
            items: collections::LinkedList::from_iter(items),
        }
    }

    pub fn fit_recurse(&mut self) -> bool {
        let mut current_item = match self.items.pop_back() {
            Some(x) => x,
            None => return true,
        };

        let mut searched_sizes = collections::HashSet::new();

        for bag_idx in 0..self.bins.len() {
            // try
            current_item = match self.bins[bag_idx].try_push(current_item) {
                None => {
                    if searched_sizes.insert(self.bins[bag_idx].capacity.clone()) {
                        // recurse
                        if self.fit_recurse() {
                            return true;
                        }
                    }

                    // backtrack
                    self.bins[bag_idx].pop().unwrap()
                }

                Some(item) => item,
            }
        }

        self.items.push_back(current_item);
        false
    }

    pub fn fit(mut self) -> Option<Vec<Bin<T>>> {
        let total_weight: T = self.items.iter().sum();
        let total_size: T = self.bins.iter().map(|bin| &bin.capacity).sum();

        if total_weight <= total_size && self.fit_recurse() {
            Some(self.bins)
        } else {
            None
        }
    }
}
