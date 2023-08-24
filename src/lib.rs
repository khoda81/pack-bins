use std::{collections, time};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Bin<T> {
    capacity: T,
    items: collections::LinkedList<T>,
}

impl<T> Bin<T>
where
    T: Clone + std::cmp::PartialOrd + std::ops::SubAssign + std::ops::AddAssign,
{
    pub fn new(capacity: T) -> Self {
        Self {
            capacity,
            items: collections::LinkedList::new(),
        }
    }

    pub fn try_push(&mut self, item: T) -> Option<T> {
        if self.capacity >= item {
            self.capacity -= item.clone();
            self.items.push_back(item);

            None
        } else {
            Some(item)
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        self.items.pop_back().map(|item| {
            self.capacity += item.clone();
            item
        })
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn items(&self) -> &collections::LinkedList<T> {
        &self.items
    }
}

enum State {
    Try,
    Backtrack,
}
struct StackItem<T> {
    searched_bins: collections::HashSet<T>,
    bin_idx: usize,
    state: State,
}

pub struct Fitter<T> {
    items: collections::LinkedList<T>,
    bins: Vec<Bin<T>>,

    stack: Vec<StackItem<T>>,
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
    pub fn new(mut items: Vec<T>, bin_capacities: impl IntoIterator<Item = T>) -> Self {
        items.sort();

        Self {
            bins: bin_capacities.into_iter().map(Bin::new).collect(),
            items: collections::LinkedList::from_iter(items),
            stack: Vec::new(),
        }
    }

    pub fn fit_recurse(&mut self) -> Option<()> {
        let mut current_item = self.items.pop_back()?;
        let mut searched_sizes = collections::HashSet::new();

        // try
        for bag_idx in 0..self.bins.len() {
            current_item = match self.bins[bag_idx].try_push(current_item) {
                Some(item) => item,

                None => {
                    if searched_sizes.insert(self.bins[bag_idx].capacity.clone()) {
                        // recurse
                        self.fit_recurse()?;
                    }

                    // backtrack
                    self.bins[bag_idx].pop().unwrap()
                }
            }
        }

        self.items.push_back(current_item);

        Some(())
    }

    pub fn fit(mut self) -> Option<Vec<Bin<T>>> {
        let total_weight: T = self.items.iter().sum();
        let total_size: T = self.bins.iter().map(|bin| &bin.capacity).sum();

        self.stack = vec![StackItem {
            searched_bins: collections::HashSet::new(),
            bin_idx: 0,
            state: State::Try,
        }];

        if total_weight <= total_size && {
            let start = time::Instant::now();
            // self.fit_recurse();
            // let num_iters = 1;
            let num_iters = self.count();
            let dur = start.elapsed();
            println!(
                "c {num_iters} iterations in {dur:?} ({:?} per iteration)",
                dur / num_iters.try_into().unwrap()
            );

            self.items.is_empty()
        } {
            Some(self.bins.into_iter().collect())
        } else {
            None
        }
    }
}

impl<T> Iterator for &mut Fitter<T>
where
    T: Clone + Ord + std::hash::Hash + std::ops::SubAssign + std::ops::AddAssign,
{
    type Item = ();

    fn next(&mut self) -> Option<Self::Item> {
        let mut current = self.stack.pop()?;

        match current.state {
            State::Backtrack => {
                let current_item = self.bins[current.bin_idx - 1].pop().unwrap();

                self.items.push_back(current_item);
                current.state = State::Try;

                self.stack.push(current);
            }

            State::Try => {
                if let Some(bin) = self.bins.get_mut(current.bin_idx) {
                    let item = self.items.pop_back()?;
                    current.bin_idx += 1;

                    match bin.try_push(item) {
                        Some(item) => {
                            self.items.push_back(item);
                            self.stack.push(current);
                        }

                        None => {
                            if current.searched_bins.insert(bin.capacity.clone()) {
                                current.state = State::Backtrack;
                                self.stack.push(current);

                                current = StackItem {
                                    searched_bins: collections::HashSet::new(),
                                    bin_idx: 0,
                                    state: State::Try,
                                };
                                self.stack.push(current)
                            } else {
                                self.items.push_back(bin.pop().unwrap());
                                self.stack.push(current);
                            }
                        }
                    };
                };
            }
        };

        Some(())
    }
}
